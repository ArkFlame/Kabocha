use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, AtomicI32, Ordering},
};

use pumpkin_data::BlockState;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos, vector3::Vector3};
use rand::RngExt;

use crate::entity::ai::goal::{Controls, Goal};
use crate::entity::{
    Entity, EntityBase,
    ageable::{AgeableData, AgeableMob},
    ai::goal::{
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal, swim::SwimGoal,
        tempt::TemptGoal,
    },
    custom_sound::CustomSound,
    mob::{Mob, MobEntity},
    passive::animal::Animal,
    player::Player,
};

/// Vanilla `HappyGhast.MAX_STILL_TIMEOUT`, applied while a player stands above the ghast.
const MAX_STILL_TIMEOUT: i32 = 10;
/// Vanilla `HappyGhast.STILL_TIMEOUT_ON_LOAD_GRACE_PERIOD`: no decrement before this age.
const STILL_TIMEOUT_GRACE: i32 = 60;
/// Per-nudge velocity toward a random float target (matches `Ghast`'s float goal).
const FLOAT_ACCELERATION: f64 = 0.1;

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
}

impl HappyGhastEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let happy_ghast = Self {
            mob_entity,
            ageable_data: AgeableData::default(),
            server_still_timeout: AtomicI32::new(0),
            leash_holder_time: AtomicI32::new(0),
            is_leash_holder: AtomicBool::new(false),
            stays_still: AtomicBool::new(false),
        };
        let mob_arc = Arc::new(happy_ghast);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };
        let happy_ghast_weak: Weak<HappyGhastEntity> = Arc::downgrade(&mob_arc);

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(
                1,
                Box::new(TemptGoal::with_tag_and_stop_distance(
                    1.0,
                    &tag::Item::MINECRAFT_HAPPY_GHAST_TEMPT_ITEMS,
                    false,
                    7.0,
                )),
            );
            // Vanilla uses a ghast-style random float goal instead of ground wandering.
            goal_selector.add_goal(
                2,
                Box::new(HappyGhastFloatAroundGoal::new(happy_ghast_weak)),
            );
            goal_selector.add_goal(
                3,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(4, Box::new(RandomLookAroundGoal::default()));
        };

        mob_arc
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
            if root_ref.cast_any().is::<HappyGhastEntity>() {
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
        let leash_time = self.leash_holder_time.load(Ordering::Relaxed);
        if leash_time > 0 {
            self.leash_holder_time.fetch_sub(1, Ordering::Relaxed);
        }
        self.set_leash_holder(leash_time > 0);

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

        // Vanilla heals one health every 600 ticks, keyed on the entity's own age.
        let entity = self.get_entity();
        let living = &self.mob_entity.living_entity;
        if entity.is_alive()
            && entity.tick_count.load(Ordering::Relaxed) % TICKS_PER_HEAL == 0
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
        if self.is_baby() {
            return self.animal_interact(player, item_stack);
        }

        self.animal_interact(player, item_stack)
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
}
