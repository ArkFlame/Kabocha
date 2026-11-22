use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, AtomicI32, AtomicUsize, Ordering},
};

use pumpkin_data::BlockState;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos, vector3::Vector3};
use rand::RngExt;

use crate::entity::ai::control::flying_move_control::FlyingMoveControl;
use crate::entity::ai::control::ghast_move_control::GhastMoveControl;
use crate::entity::ai::control::happy_ghast_look_control::HappyGhastLookControl;
use crate::entity::ai::control::look_control::LookControl;
use crate::entity::ai::control::{LookControlTrait, MoveControlTrait};
use crate::entity::ai::goal::{Controls, Goal, GoalReach};
use crate::entity::{
    Entity, EntityBase,
    ageable::{AgeableData, AgeableMob},
    ai::goal::{goal_selector::GoalSelector, swim::SwimGoal, tempt::TemptGoal},
    custom_sound::CustomSound,
    mob::{Mob, MobEntity, equipment::equip_on_target},
    passive::animal::Animal,
    player::Player,
};

/// Vanilla `HappyGhast.MAX_STILL_TIMEOUT`, applied while a player stands above the ghast.
const MAX_STILL_TIMEOUT: i32 = 10;
/// Vanilla `HappyGhast.STILL_TIMEOUT_ON_LOAD_GRACE_PERIOD`: no decrement before this age.
const STILL_TIMEOUT_GRACE: i32 = 60;
/// Vanilla `HappyGhast.continuousHeal` interval while precipitating on the ghast.
const FAST_HEAL_TICKS: i32 = 20;
/// Per-nudge velocity toward a random float target (matches `Ghast`'s float goal).
const FLOAT_ACCELERATION: f64 = 0.1;
/// Vanilla Happy Ghast leash snap distance.
const LEASH_SNAP_DISTANCE: f64 = 16.0;
/// Vanilla Happy Ghast leash elastic distance.
const LEASH_ELASTIC_DISTANCE: f64 = 10.0;

/// Represents a Happy Ghast, a passive flying mob.
///
/// Wiki: <https://minecraft.wiki/w/Happy_Ghast>
pub struct HappyGhastEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: AgeableData,
    pub server_still_timeout: AtomicI32,
    pub leash_holder_time: AtomicI32,
    pub is_leash_holder: AtomicBool,
    pub stays_still: AtomicBool,
    /// Last observed passenger count, used to replay vanilla `addPassenger`/`removePassenger` side effects.
    pub observed_passenger_count: AtomicUsize,
    /// Weak self handle, so `age_boundary_reached` can re-register goals that need it.
    self_weak: Weak<Self>,
}

/// Vanilla `HappyGhast.registerGoals` predicate: babies and harnessed ghasts temp on
/// `HAPPY_GHAST_FOOD`, other adults on `HAPPY_GHAST_TEMPT_ITEMS`.
fn tempt_item_predicate(mob: &dyn Mob, stack: &ItemStack) -> bool {
    let baby = mob
        .as_ageable()
        .is_some_and(crate::entity::ageable::AgeableMob::is_baby);
    let harnessed = !mob
        .get_mob_entity()
        .is_slot_empty(&pumpkin_data::data_component_impl::EquipmentSlot::BODY);
    if !baby && !harnessed {
        stack
            .item
            .has_tag(&tag::Item::MINECRAFT_HAPPY_GHAST_TEMPT_ITEMS)
    } else {
        stack.item.has_tag(&tag::Item::MINECRAFT_HAPPY_GHAST_FOOD)
    }
}

impl HappyGhastEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let mob_arc = Arc::new_cyclic(|self_weak| Self {
            mob_entity,
            ageable_data: AgeableData::default(),
            server_still_timeout: AtomicI32::new(0),
            leash_holder_time: AtomicI32::new(0),
            is_leash_holder: AtomicBool::new(false),
            stays_still: AtomicBool::new(false),
            observed_passenger_count: AtomicUsize::new(0),
            self_weak: self_weak.clone(),
        });
        // Vanilla `HappyGhast` constructor installs the adult controls before any age change.
        mob_arc.install_controls(false);
        Self::register_goals(
            &mob_arc.mob_entity.goals_selector,
            mob_arc.self_weak.clone(),
        );
        mob_arc
    }

    /// Vanilla `HappyGhast.registerGoals`.
    fn register_goals(goal_selector: &GoalSelector, ghast: Weak<Self>) {
        goal_selector.add_goal(3, Box::new(HappyGhastFloatGoal::new(ghast.clone())));
        goal_selector.add_goal(
            4,
            Box::new(TemptGoal::with_item_predicate(
                1.0,
                tempt_item_predicate,
                false,
                7.0,
            )),
        );
        // Vanilla uses a ghast-style random float goal instead of ground wandering.
        goal_selector.add_goal(5, Box::new(HappyGhastFloatAroundGoal::new(ghast)));
    }

    pub fn set_server_still_timeout(&self, timeout: i32) {
        self.server_still_timeout.store(timeout, Ordering::Relaxed);
        self.sync_stay_still_flag();
    }

    #[must_use]
    pub fn is_on_still_timeout(&self) -> bool {
        self.stays_still.load(Ordering::Relaxed)
            || self.server_still_timeout.load(Ordering::Relaxed) > 0
    }

    /// `StillCheck` for the ghast controls: downcasts the mob back to a happy ghast.
    fn is_still_check(mob: &dyn Mob) -> bool {
        mob.cast_any()
            .downcast_ref::<Self>()
            .is_some_and(Self::is_on_still_timeout)
    }

    /// Vanilla `HappyGhast.adultGhastSetup` and `babyGhastSetup` control swap.
    fn install_controls(&self, baby: bool) {
        let move_control: Box<dyn MoveControlTrait> = if baby {
            Box::new(FlyingMoveControl::new(180, true))
        } else {
            Box::new(GhastMoveControl::new(true, Some(Self::is_still_check)))
        };
        let look_control: Box<dyn LookControlTrait> = if baby {
            Box::new(LookControl::default())
        } else {
            Box::new(HappyGhastLookControl::new(Self::is_still_check))
        };
        *self
            .mob_entity
            .move_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = move_control;
        *self
            .mob_entity
            .look_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = look_control;
    }

    /// Vanilla HappyGhast.isWearingBodyArmor: a non-empty body slot is a harness.
    fn is_wearing_harness(&self) -> bool {
        !self
            .mob_entity
            .is_slot_empty(&pumpkin_data::data_component_impl::EquipmentSlot::BODY)
    }

    /// Vanilla HappyGhast.canAddPassenger caps riders at four; mobInteract also
    /// requires a harness and no sneak key before a player mounts.
    const fn can_player_mount(
        wearing_harness: bool,
        sneaking: bool,
        passenger_count: usize,
    ) -> bool {
        const MAX_PASSENGERS: usize = 4;
        wearing_harness && !sneaking && passenger_count < MAX_PASSENGERS
    }

    /// Vanilla `HappyGhast.addPassenger`/`removePassenger` play the goggles sound only
    /// when the ghast becomes or stops being a vehicle.
    const fn goggles_sound_on_change(previous: usize, current: usize) -> Option<Sound> {
        if previous == 0 && current > 0 {
            Some(Sound::EntityHappyGhastHarnessGogglesDown)
        } else if previous > 0 && current == 0 {
            Some(Sound::EntityHappyGhastHarnessGogglesUp)
        } else {
            None
        }
    }

    fn set_leash_holder(&self, is_leash_holder: bool) {
        self.is_leash_holder
            .store(is_leash_holder, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::happy_ghast::IS_LEASH_HOLDER,
            is_leash_holder,
        );
    }

    fn sync_stay_still_flag(&self) {
        let stays_still = self.server_still_timeout.load(Ordering::Relaxed) > 0;
        self.stays_still.store(stays_still, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::happy_ghast::STAYS_STILL,
            stays_still,
        );
    }

    /// Vanilla `HappyGhast.scanPlayerAboveGhast` builds this box just above the ghast.
    fn still_detection_box(bb: BoundingBox) -> BoundingBox {
        let height = bb.max.y - bb.min.y;
        BoundingBox {
            min: Vector3::new(bb.min.x - 1.0, bb.max.y - 1.0e-5, bb.min.z - 1.0),
            max: Vector3::new(bb.max.x + 1.0, bb.max.y + height / 2.0, bb.max.z + 1.0),
        }
    }

    /// Vanilla `HappyGhast.notifyLeashHolder`: `Some(5)` only when the leashee supports a quad
    /// leash; otherwise the timer is left as-is (`None`).
    fn leash_holder_notify_ticks(quad_leash: bool) -> Option<i32> {
        quad_leash.then_some(5)
    }

    /// Vanilla `HappyGhast.tick`: decrement the lead-holder timer before evaluating the synced
    /// `IS_LEASH_HOLDER` flag, so the flag clears on the tick the timer reaches zero.
    const fn tick_leash_holder_time(current: i32) -> (i32, bool) {
        let next = if current > 0 { current - 1 } else { 0 };
        (next, next > 0)
    }

    fn scan_player_above_ghast(&self) -> bool {
        let entity = self.get_entity();
        let detection_box = Self::still_detection_box(entity.bounding_box.load());
        let world = entity.world.load();
        world.players.load().iter().any(|player| {
            if player.is_spectator() {
                return false;
            }
            let player_base: &dyn EntityBase = player.as_ref();
            let root = player.get_entity().get_root_vehicle();
            let root_ref: &dyn EntityBase = root.as_deref().unwrap_or(player_base);
            // Vanilla ignores a player already riding a happy ghast.
            if root_ref.cast_any().is::<Self>() {
                return false;
            }
            detection_box.contains(root_ref.get_entity().pos.load())
        })
    }
}

impl AgeableMob for HappyGhastEntity {
    fn get_ageable_data(&self) -> &AgeableData {
        &self.ageable_data
    }

    /// Vanilla `HappyGhast.ageBoundaryReached`: babies drop every goal and clear the still
    /// timeout; adults rebuild their goals.
    fn age_boundary_reached(&self) {
        let goal_selector = &self.mob_entity.goals_selector;
        if self.is_baby() {
            self.install_controls(true);
            self.set_server_still_timeout(0);
            goal_selector.clear(self);
        } else {
            self.install_controls(false);
            goal_selector.clear(self);
            Self::register_goals(goal_selector, self.self_weak.clone());
        }
    }
}

impl CustomSound for HappyGhastEntity {
    fn hurt_sound(&self) -> Option<Sound> {
        Some(if self.is_baby() {
            Sound::EntityGhastlingHurt
        } else {
            Sound::EntityHappyGhastHurt
        })
    }

    fn death_sound(&self) -> Option<Sound> {
        Some(if self.is_baby() {
            Sound::EntityGhastlingDeath
        } else {
            Sound::EntityHappyGhastDeath
        })
    }

    /// Vanilla `HappyGhast#getSoundVolume`: babies are quiet, adults are loud.
    fn sound_volume(&self) -> f32 {
        if self.is_baby() { 1.0 } else { 4.0 }
    }
}

impl Animal for HappyGhastEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack
            .item
            .has_tag(&tag::Item::MINECRAFT_HAPPY_GHAST_FOOD)
    }
}

impl Mob for HappyGhastEntity {
    fn as_custom_sound(&self) -> Option<&dyn CustomSound> {
        Some(self)
    }

    /// Hovers like `Ghast`: cancel the generated gravity and damp vertical velocity.
    fn get_mob_gravity(&self) -> f64 {
        0.0
    }

    fn get_mob_y_velocity_drag(&self) -> Option<f64> {
        Some(0.95)
    }

    fn mob_support_quad_leash_as_holder(&self) -> bool {
        true
    }

    fn mob_leash_snap_distance(&self) -> f64 {
        LEASH_SNAP_DISTANCE
    }

    fn mob_leash_elastic_distance(&self) -> f64 {
        LEASH_ELASTIC_DISTANCE
    }

    /// Vanilla HappyGhast.getControllingPassenger: the first player steers only
    /// while harnessed and not on a still timeout.
    fn mob_controlling_passenger(&self) -> Option<Arc<dyn EntityBase>> {
        if !self.is_wearing_harness() || self.is_on_still_timeout() {
            return None;
        }
        let first = self.get_mob_entity().first_passenger()?;
        if first.get_player().is_some() {
            Some(first)
        } else {
            self.mob_rider_controller(first)
        }
    }

    /// Vanilla `HappyGhast.notifyLeashHolder`: a leashee with a quad leash makes the ghast
    /// render as a lead holder for 5 ticks. A non-quad leashee leaves the timer untouched.
    fn mob_notify_leash_holder(&self, leashee: &dyn EntityBase) {
        if let Some(ticks) = Self::leash_holder_notify_ticks(leashee.support_quad_leash()) {
            self.leash_holder_time.store(ticks, Ordering::Relaxed);
        }
    }

    fn can_dispenser_equip_into_slot(
        &self,
        slot: &pumpkin_data::data_component_impl::EquipmentSlot,
    ) -> bool {
        *slot == pumpkin_data::data_component_impl::EquipmentSlot::BODY
    }

    fn should_stay_close_to_leash_holder(&self) -> bool {
        false
    }

    /// Vanilla `HappyGhast#causeFallDamage` always returns false.
    fn should_check_fall_damage(&self) -> bool {
        false
    }

    /// Vanilla `HappyGhast` steps silently.
    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {}

    fn as_ageable(&self) -> Option<&dyn AgeableMob> {
        Some(self)
    }

    fn as_animal(&self) -> Option<&dyn Animal> {
        Some(self)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_int(
            "still_timeout",
            self.server_still_timeout.load(Ordering::Relaxed),
        );
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if let Some(timeout) = nbt.get_int("still_timeout") {
            self.set_server_still_timeout(timeout);
        }
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn get_ambient_sound(&self) -> Option<Sound> {
        Some(if self.is_baby() {
            Sound::EntityGhastlingAmbient
        } else {
            Sound::EntityHappyGhastAmbient
        })
    }

    /// Vanilla ambient interval is 120 ticks, stretched 6x while carrying passengers.
    fn get_ambient_sound_interval(&self) -> i32 {
        if self.get_entity().has_passengers() {
            720
        } else {
            120
        }
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        const TICKS_PER_HEAL: i32 = 600;
        let (next_leash_time, is_leash_holder) =
            Self::tick_leash_holder_time(self.leash_holder_time.load(Ordering::Relaxed));
        self.leash_holder_time
            .store(next_leash_time, Ordering::Relaxed);
        self.set_leash_holder(is_leash_holder);

        let still_timeout = self.server_still_timeout.load(Ordering::Relaxed);
        if still_timeout > 0 {
            let entity = self.get_entity();
            if entity.tick_count.load(Ordering::Relaxed) > STILL_TIMEOUT_GRACE {
                self.server_still_timeout.fetch_sub(1, Ordering::Relaxed);
            }
            self.sync_stay_still_flag();
        }

        // Vanilla `HappyGhast.tick`: a player above keeps the ghast collidably still.
        if self.scan_player_above_ghast() {
            self.set_server_still_timeout(MAX_STILL_TIMEOUT);
        }

        // Vanilla `HappyGhast.addPassenger`/`removePassenger`: mounting clears the still
        // timeout unless another player stands above; dismounting resets it to 10 and
        // toggles the goggles sound and home when the last rider leaves.
        let ghast_entity = self.get_entity();
        let current_passengers = ghast_entity
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len();
        let previous_passengers = self
            .observed_passenger_count
            .swap(current_passengers, Ordering::Relaxed);
        if current_passengers != previous_passengers {
            if let Some(sound) =
                Self::goggles_sound_on_change(previous_passengers, current_passengers)
            {
                ghast_entity.play_sound(sound);
            }
            if current_passengers > previous_passengers {
                if !self.scan_player_above_ghast() {
                    self.set_server_still_timeout(0);
                } else if self.server_still_timeout.load(Ordering::Relaxed) > MAX_STILL_TIMEOUT {
                    self.set_server_still_timeout(MAX_STILL_TIMEOUT);
                }
            } else {
                self.set_server_still_timeout(MAX_STILL_TIMEOUT);
                if current_passengers == 0 {
                    self.mob_entity.clear_home();
                }
            }
        }

        // Vanilla `HappyGhast.continuousHeal`: one health every 20 ticks while it rains
        // on the ghast, otherwise every 600, keyed on the entity's own age. The
        // in-clouds fast heal needs the cloud environment attribute, which is not modelled.
        let entity = self.get_entity();
        let living = &self.mob_entity.living_entity;
        let world = entity.world.load();
        let interval = if world.is_raining_at(&entity.block_pos.load()) {
            FAST_HEAL_TICKS
        } else {
            TICKS_PER_HEAL
        };
        if entity.is_alive()
            && entity.tick_count.load(Ordering::Relaxed) % interval == 0
            && living.health.load() < living.get_max_health()
        {
            living.heal(1.0);
        }
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        let is_baby = entity.age.load(Ordering::Relaxed) < 0;
        entity.set_synced_data(pumpkin_data::tracked_data::happy_ghast::BABY_ID, is_baby);
        entity.set_synced_data(
            pumpkin_data::tracked_data::happy_ghast::IS_LEASH_HOLDER,
            self.is_leash_holder.load(Ordering::Relaxed),
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::happy_ghast::STAYS_STILL,
            self.stays_still.load(Ordering::Relaxed),
        );
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        // Vanilla HappyGhast.mobInteract: babies only feed; adults let the held
        // item interact (harness equip) before an un-sneaking player mounts.
        if self.is_baby() {
            return self.animal_interact(player, item_stack);
        }
        if self.animal_interact(player, item_stack) || equip_on_target(self, player, item_stack) {
            return true;
        }
        let entity = self.get_entity();
        let passenger_count = entity
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len();
        if !Self::can_player_mount(
            self.is_wearing_harness(),
            player.get_entity().is_sneaking(),
            passenger_count,
        ) {
            return false;
        }
        let world = player.world();
        if let Some(vehicle) = world.get_entity_by_id(entity.entity_id)
            && let Some(passenger) = world.get_player_by_id(player.entity_id())
        {
            entity.add_passenger(vehicle, passenger as Arc<dyn EntityBase>);
        }
        true
    }
}

/// Vanilla `HappyGhast.HappyGhastFloatGoal`: `FloatGoal` (`SwimGoal` here) that also
/// requires the ghast to not be on a still timeout.
pub struct HappyGhastFloatGoal {
    ghast: Weak<HappyGhastEntity>,
    swim: SwimGoal,
}

impl HappyGhastFloatGoal {
    #[must_use]
    pub fn new(ghast: Weak<HappyGhastEntity>) -> Self {
        Self {
            ghast,
            swim: SwimGoal::default(),
        }
    }
}

impl Goal for HappyGhastFloatGoal {
    fn tick_reach(&self, mob: &dyn Mob, running: bool) -> Option<GoalReach> {
        self.swim.tick_reach(mob, running)
    }

    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(ghast) = self.ghast.upgrade() else {
            return false;
        };
        !ghast.is_on_still_timeout() && self.swim.can_start(mob)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        self.swim.should_continue(mob)
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.swim.start(mob);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.swim.stop(mob);
    }

    fn tick(&mut self, mob: &dyn Mob) {
        self.swim.tick(mob);
    }

    fn should_run_every_tick(&self) -> bool {
        self.swim.should_run_every_tick()
    }

    fn can_stop(&self) -> bool {
        self.swim.can_stop()
    }

    fn get_tick_count(&self, ticks: i32) -> i32 {
        self.swim.get_tick_count(ticks)
    }

    fn controls(&self) -> Controls {
        self.swim.controls()
    }
}

/// Vanilla `Ghast.RandomFloatAroundGoal`: picks a point within 16 blocks and nudges
/// the ghast toward it, gated on not being on a still timeout.
pub struct HappyGhastFloatAroundGoal {
    ghast: Weak<HappyGhastEntity>,
    wanted_target: Option<Vector3<f64>>,
    float_duration: i32,
}

impl HappyGhastFloatAroundGoal {
    #[must_use]
    pub const fn new(ghast: Weak<HappyGhastEntity>) -> Self {
        Self {
            ghast,
            wanted_target: None,
            float_duration: 0,
        }
    }
}

impl Goal for HappyGhastFloatAroundGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        let Some(ghast) = self.ghast.upgrade() else {
            return false;
        };
        if ghast.is_on_still_timeout() {
            return false;
        }
        let pos = ghast.get_entity().pos.load();
        self.wanted_target.is_none_or(|target| {
            let dist_sq = pos.squared_distance_to_vec(&target);
            !dist_sq.is_nan() && !(1.0..=3600.0).contains(&dist_sq)
        })
    }

    fn start(&mut self, _mob: &dyn Mob) {
        let Some(ghast) = self.ghast.upgrade() else {
            return;
        };
        let pos = ghast.get_entity().pos.load();
        let mut rng = rand::rng();
        self.wanted_target = Some(Vector3::new(
            pos.x + (rng.random::<f64>() * 2.0 - 1.0) * 16.0,
            pos.y + (rng.random::<f64>() * 2.0 - 1.0) * 16.0,
            pos.z + (rng.random::<f64>() * 2.0 - 1.0) * 16.0,
        ));
    }

    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        let Some(ghast) = self.ghast.upgrade() else {
            return false;
        };
        if ghast.is_on_still_timeout() {
            return false;
        }
        let pos = ghast.get_entity().pos.load();
        self.wanted_target.is_some_and(|target| {
            let dist_sq = pos.squared_distance_to_vec(&target);
            (1.0..=3600.0).contains(&dist_sq)
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn tick(&mut self, _mob: &dyn Mob) {
        let Some(ghast) = self.ghast.upgrade() else {
            return;
        };
        let Some(target) = self.wanted_target else {
            return;
        };
        let entity = ghast.get_entity();
        let pos = entity.pos.load();
        self.float_duration -= 1;
        if self.float_duration <= 0 {
            self.float_duration = rand::random_range(2..=6);
            let travel = Vector3::new(target.x - pos.x, target.y - pos.y, target.z - pos.z);
            if travel.length() > 0.001 {
                let norm = travel.normalize();
                let velocity = entity.velocity.load();
                entity.velocity.store(Vector3::new(
                    velocity.x + norm.x * FLOAT_ACCELERATION,
                    velocity.y + norm.y * FLOAT_ACCELERATION,
                    velocity.z + norm.z * FLOAT_ACCELERATION,
                ));
            }
        }
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn still_detection_box_matches_vanilla_extents() {
        let bb = BoundingBox {
            min: Vector3::new(-1.5, 10.0, -2.0),
            max: Vector3::new(1.5, 13.0, 2.0),
        };
        let detection = HappyGhastEntity::still_detection_box(bb);
        assert_eq!(detection.min.x, -2.5);
        assert_eq!(detection.min.y, 13.0 - 1.0e-5);
        assert_eq!(detection.min.z, -3.0);
        assert_eq!(detection.max.x, 2.5);
        assert_eq!(detection.max.y, 14.5);
        assert_eq!(detection.max.z, 3.0);
    }

    #[test]
    fn leash_holder_notify_only_quad_leashee_extends_timer() {
        assert_eq!(HappyGhastEntity::leash_holder_notify_ticks(true), Some(5));
        assert_eq!(HappyGhastEntity::leash_holder_notify_ticks(false), None);
    }

    #[test]
    fn leash_holder_flag_uses_decremented_time() {
        assert_eq!(HappyGhastEntity::tick_leash_holder_time(5), (4, true));
        assert_eq!(HappyGhastEntity::tick_leash_holder_time(1), (0, false));
        assert_eq!(HappyGhastEntity::tick_leash_holder_time(0), (0, false));
    }

    #[test]
    fn player_mount_requires_harness_and_a_free_seat() {
        assert!(HappyGhastEntity::can_player_mount(true, false, 0));
        assert!(HappyGhastEntity::can_player_mount(true, false, 3));
        assert!(!HappyGhastEntity::can_player_mount(true, false, 4));
        assert!(!HappyGhastEntity::can_player_mount(false, false, 0));
        assert!(!HappyGhastEntity::can_player_mount(true, true, 0));
    }

    #[test]
    fn goggles_sound_only_on_vehicle_transition() {
        assert!(HappyGhastEntity::goggles_sound_on_change(0, 1).is_some());
        assert!(HappyGhastEntity::goggles_sound_on_change(1, 2).is_none());
        assert!(HappyGhastEntity::goggles_sound_on_change(2, 1).is_none());
        assert!(HappyGhastEntity::goggles_sound_on_change(1, 0).is_some());
        assert!(HappyGhastEntity::goggles_sound_on_change(0, 0).is_none());
    }
}
