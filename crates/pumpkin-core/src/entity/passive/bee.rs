use std::sync::{
    Arc,
    atomic::{AtomicI32, AtomicU8, Ordering},
};

use pumpkin_data::BlockState;
use pumpkin_data::damage::DamageType;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::potion::Effect;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use rand::RngExt;

use crate::entity::ai::behavior::hunting::{self, Hunt};
use crate::entity::ai::target_match::TargetMatch;
use crate::entity::{
    Entity, EntityBase,
    ageable::{AgeableData, AgeableMob},
    ai::control::{
        Control, LookControlTrait, flying_move_control::FlyingMoveControl,
        look_control::LookControl,
    },
    ai::goal::{
        active_target::TargetCondition, breed::BreedGoal, follow_parent::FollowParentGoal,
        grow_crop::GrowCropGoal, melee_attack::MeleeAttackGoal,
        pollinate_flower::PollinateFlowerGoal, reset_universal_anger::ResetUniversalAngerGoal,
        revenge::RevengeGoal, swim::SwimGoal, tempt::TemptGoal, wander_around::WanderAroundGoal,
    },
    ai::pathfinder::{Navigator, node::PathType},
    mob::{
        Mob, MobEntity,
        neutral::{NeutralData, NeutralMob},
    },
    passive::animal::Animal,
    player::Player,
};

pub const FLAG_ROLL: u8 = 2;
pub const FLAG_HAS_STUNG: u8 = 4;
pub const FLAG_HAS_NECTAR: u8 = 8;

/// Vanilla `Bee.TICKS_PER_FLAP`, `Mth.ceil(1.4959966F)`.
const TICKS_PER_FLAP: i32 = 2;

/// Vanilla `Bee.doHurtTarget` sting poison in ticks: 10 s on Normal, 18 s on Hard, none otherwise.
const fn sting_poison_ticks(difficulty: pumpkin_util::Difficulty) -> i32 {
    match difficulty {
        pumpkin_util::Difficulty::Normal => 10 * 20,
        pumpkin_util::Difficulty::Hard => 18 * 20,
        _ => 0,
    }
}

fn is_angry(mob: &dyn Mob) -> bool {
    mob.as_neutral().is_some_and(NeutralMob::is_angry)
}

/// Vanilla `BeePollinateGoal` only runs while the bee has no nectar.
fn needs_nectar(mob: &dyn Mob) -> bool {
    mob.cast_any()
        .downcast_ref::<BeeEntity>()
        .is_some_and(|bee| !bee.has_nectar())
}

fn record_nectar(mob: &dyn Mob) {
    if let Some(bee) = mob.cast_any().downcast_ref::<BeeEntity>() {
        bee.set_has_nectar(true);
    }
}

/// Vanilla `Bee.MAX_CROPS_GROWN` per pollination cycle.
const MAX_CROPS_GROWN_SINCE_POLLINATION: i32 = 10;

/// Vanilla `BeeGrowCropGoal` only runs while the bee carries nectar and has growth budget left.
fn can_grow_crop(mob: &dyn Mob) -> bool {
    mob.cast_any()
        .downcast_ref::<BeeEntity>()
        .is_some_and(|bee| {
            bee.has_nectar()
                && bee.crops_grown_since_pollination() < MAX_CROPS_GROWN_SINCE_POLLINATION
        })
}

/// Counts a fertilized crop; the bee loses its nectar once it has grown the maximum.
fn record_crop_grown(mob: &dyn Mob) {
    if let Some(bee) = mob.cast_any().downcast_ref::<BeeEntity>() {
        let grown = bee.record_crop_grown();
        if grown >= MAX_CROPS_GROWN_SINCE_POLLINATION {
            bee.set_has_nectar(false);
        }
    }
}

/// Angry and still holding its stinger.
fn can_sting(mob: &dyn Mob) -> bool {
    is_angry(mob)
        && !mob
            .cast_any()
            .downcast_ref::<BeeEntity>()
            .is_some_and(BeeEntity::has_stung)
}

const BEE_HUNTS: &[Hunt] = &[Hunt::new(2, TargetMatch::Type(&EntityType::PLAYER), true)
    .when(TargetCondition::AngryAt)
    .gated_by(can_sting)];

pub struct BeeEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: AgeableData,
    pub neutral_data: NeutralData,
    pub flags: AtomicU8,
    pub ticks_without_nectar: AtomicI32,
    pub cannot_enter_hive_ticks: AtomicI32,
    pub crops_grown_since_pollination: AtomicI32,
    pub time_since_sting: AtomicI32,
}

impl BeeEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mut mob_entity = MobEntity::new(entity);
        *mob_entity
            .move_control
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            Box::new(FlyingMoveControl::new(20, true));
        *mob_entity
            .look_control
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            Box::new(BeeLookControl::default());
        let navigator = mob_entity
            .navigator
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *navigator = Navigator::flying();
        navigator.set_can_open_doors(false);
        navigator.set_can_float(false);
        navigator.set_required_path_length(48.0);
        // Vanilla `Bee` constructor maluses. Java FIRE maps to DamageFire, as in blaze.rs.
        navigator.set_pathfinding_malus(PathType::DamageFire, -1.0);
        navigator.set_pathfinding_malus(PathType::Water, -1.0);
        navigator.set_pathfinding_malus(PathType::WaterBorder, 16.0);
        navigator.set_pathfinding_malus(PathType::Cocoa, -1.0);
        navigator.set_pathfinding_malus(PathType::Fence, -1.0);
        let bee = Self {
            mob_entity,
            ageable_data: AgeableData::default(),
            neutral_data: NeutralData::default(),
            flags: AtomicU8::new(0),
            ticks_without_nectar: AtomicI32::new(0),
            cannot_enter_hive_ticks: AtomicI32::new(0),
            crops_grown_since_pollination: AtomicI32::new(0),
            time_since_sting: AtomicI32::new(0),
        };
        let mob_arc = Arc::new(bee);

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;

            goal_selector.add_goal(
                0,
                Box::new(MeleeAttackGoal::new(1.4, true).gated_by(can_sting)),
            );
            // UNVERIFIED: vanilla `BeePollinateGoal` priority 4.
            goal_selector.add_goal(
                4,
                Box::new(PollinateFlowerGoal::new(needs_nectar, record_nectar)),
            );
            // UNVERIFIED: vanilla `BeeGrowCropGoal` priority 7 (tracker notes say 6).
            goal_selector.add_goal(
                7,
                Box::new(GrowCropGoal::new(can_grow_crop, record_crop_grown)),
            );
            goal_selector.add_goal(2, BreedGoal::new(1.0));
            goal_selector.add_goal(
                3,
                Box::new(TemptGoal::with_tag(
                    1.25,
                    &tag::Item::MINECRAFT_BEE_FOOD,
                    false,
                )),
            );
            goal_selector.add_goal(5, Box::new(FollowParentGoal::new(1.25)));
            goal_selector.add_goal(8, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(9, Box::new(SwimGoal::default()));
        };

        {
            let target_selector = &mob_arc.mob_entity.target_selector;

            // Vanilla BeeHurtByOtherGoal.
            target_selector.add_goal(
                1,
                Box::new(
                    RevengeGoal::new(true)
                        .alerting_others()
                        .alerting_in_sight()
                        .continuing_while(is_angry),
                ),
            );
            hunting::apply(target_selector, &mob_arc.mob_entity, BEE_HUNTS);
            target_selector.add_goal(3, ResetUniversalAngerGoal::new(true));
        };

        mob_arc
    }

    #[must_use]
    pub fn has_flag(&self, flag: u8) -> bool {
        (self.flags.load(Ordering::Relaxed) & flag) != 0
    }

    pub fn set_flag(&self, flag: u8, val: bool) {
        let current = self.flags.load(Ordering::Relaxed);
        let new_flags = if val { current | flag } else { current & !flag };
        self.flags.store(new_flags, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::bee::DATA_FLAGS_ID,
            new_flags as i8,
        );
    }

    #[must_use]
    pub fn has_nectar(&self) -> bool {
        self.has_flag(FLAG_HAS_NECTAR)
    }

    pub fn set_has_nectar(&self, val: bool) {
        self.set_flag(FLAG_HAS_NECTAR, val);
    }

    #[must_use]
    pub fn crops_grown_since_pollination(&self) -> i32 {
        self.crops_grown_since_pollination.load(Ordering::Relaxed)
    }

    /// Adds one fertilized crop and returns the new count.
    pub fn record_crop_grown(&self) -> i32 {
        self.crops_grown_since_pollination
            .fetch_add(1, Ordering::Relaxed)
            + 1
    }

    #[must_use]
    pub fn has_stung(&self) -> bool {
        self.has_flag(FLAG_HAS_STUNG)
    }

    pub fn set_has_stung(&self, val: bool) {
        self.set_flag(FLAG_HAS_STUNG, val);
    }

    #[must_use]
    pub fn is_rolling(&self) -> bool {
        self.has_flag(FLAG_ROLL)
    }

    pub fn set_rolling(&self, val: bool) {
        self.set_flag(FLAG_ROLL, val);
    }
}

impl AgeableMob for BeeEntity {
    fn get_ageable_data(&self) -> &AgeableData {
        &self.ageable_data
    }
}

impl Animal for BeeEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack.item.has_tag(&tag::Item::MINECRAFT_FLOWERS)
            || item_stack.item.has_tag(&tag::Item::MINECRAFT_BEE_FOOD)
    }
}

impl NeutralMob for BeeEntity {
    fn updates_anger_in_ai_step(&self) -> bool {
        true
    }

    fn get_neutral_data(&self) -> &NeutralData {
        &self.neutral_data
    }

    /// Bees calm down once the target is gone, unlike the other neutral mobs.
    fn stays_angry_with_target(&self) -> bool {
        false
    }

    /// The client picks the angry eyes from this value.
    fn anger_end_time_data(&self) -> Option<pumpkin_data::tracked_data::TrackedData> {
        Some(pumpkin_data::tracked_data::bee::DATA_ANGER_END_TIME)
    }
}

#[derive(Default)]
struct BeeLookControl(LookControl);

impl Control for BeeLookControl {}

impl LookControlTrait for BeeLookControl {
    fn tick(&mut self, mob: &dyn Mob) {
        if !is_angry(mob) {
            self.0.tick(mob);
        }
    }

    fn look_at_with_range(
        &mut self,
        x: f64,
        y: f64,
        z: f64,
        max_yaw_change: f32,
        max_pitch_change: f32,
    ) {
        self.0
            .look_at_with_range(x, y, z, max_yaw_change, max_pitch_change);
    }
}

impl Mob for BeeEntity {
    fn mob_is_flapping(&self) -> bool {
        let entity = self.get_entity();
        !entity.on_ground.load(Ordering::Relaxed)
            && entity.tick_count.load(Ordering::Relaxed) % TICKS_PER_FLAP == 0
    }

    /// Vanilla `Bee` steps silently.
    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {}

    fn omnidirectional_air_mover(&self) -> bool {
        true
    }

    fn as_ageable(&self) -> Option<&dyn AgeableMob> {
        Some(self)
    }

    fn as_animal(&self) -> Option<&dyn Animal> {
        Some(self)
    }

    fn as_neutral(&self) -> Option<&dyn NeutralMob> {
        Some(self)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_bool("HasNectar", self.has_nectar());
        nbt.put_bool("HasStung", self.has_stung());
        nbt.put_int(
            "TicksSincePollination",
            self.ticks_without_nectar.load(Ordering::Relaxed),
        );
        nbt.put_int(
            "CannotEnterHiveTicks",
            self.cannot_enter_hive_ticks.load(Ordering::Relaxed),
        );
        nbt.put_int(
            "CropsGrownSincePollination",
            self.crops_grown_since_pollination.load(Ordering::Relaxed),
        );
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if let Some(nectar) = nbt.get_bool("HasNectar") {
            self.set_has_nectar(nectar);
        }
        if let Some(stung) = nbt.get_bool("HasStung") {
            self.set_has_stung(stung);
        }
        if let Some(ticks) = nbt.get_int("TicksSincePollination") {
            self.ticks_without_nectar.store(ticks, Ordering::Relaxed);
        }
        if let Some(cannot) = nbt.get_int("CannotEnterHiveTicks") {
            self.cannot_enter_hive_ticks
                .store(cannot, Ordering::Relaxed);
        }
        if let Some(crops) = nbt.get_int("CropsGrownSincePollination") {
            self.crops_grown_since_pollination
                .store(crops, Ordering::Relaxed);
        }
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn custom_server_ai_step(&self, caller: &dyn EntityBase) {
        // Vanilla `Bee.customServerAiStep`: a bee that has stung keeps counting and
        // may die from its own sting once the countdown nears 1200 ticks.
        if self.has_stung() {
            // Vanilla increments first, then rolls on multiples of 5.
            let time = self.time_since_sting.load(Ordering::Relaxed) + 1;
            self.time_since_sting.store(time, Ordering::Relaxed);
            if time % 5 == 0 {
                let bound = (1200 - time).clamp(1, 1200);
                if self.get_random().random_range(0..bound) == 0 {
                    let living = &self.mob_entity.living_entity;
                    living.damage(caller, living.health.load(), DamageType::GENERIC);
                }
            }
        }
        // Vanilla `Bee.customServerAiStep`: count ticks spent without nectar. The hive
        // return reads this at 3600, and the pollination reset is not ported yet.
        if !self.has_nectar() {
            self.ticks_without_nectar.fetch_add(1, Ordering::Relaxed);
        }
        self.update_persistent_anger();
    }

    /// Vanilla `Bee.doHurtTarget`: a successful sting uses up the stinger and drops the target.
    fn on_attack(&self, target: &dyn EntityBase) {
        if self.has_stung() {
            return;
        }
        if let Some(living) = target.get_living_entity() {
            let world = self.mob_entity.living_entity.entity.world.load();
            let duration = sting_poison_ticks(world.level_info.load().difficulty);
            if duration > 0 {
                living.add_effect(Effect {
                    effect_type: &StatusEffect::POISON,
                    duration,
                    amplifier: 0,
                    ambient: false,
                    show_particles: true,
                    show_icon: true,
                    blend: false,
                });
            }
        }
        self.set_has_stung(true);
        self.mob_entity.set_target(None);
        self.mob_entity
            .living_entity
            .entity
            .play_sound(Sound::EntityBeeSting);
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        let is_baby = entity.age.load(Ordering::Relaxed) < 0;
        if is_baby {
            entity.set_synced_data(pumpkin_data::tracked_data::bee::DATA_BABY_ID, true);
        }
        entity.set_synced_data(
            pumpkin_data::tracked_data::bee::DATA_FLAGS_ID,
            self.flags.load(Ordering::Relaxed) as i8,
        );
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        self.animal_interact(player, item_stack)
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_util::Difficulty;

    use super::sting_poison_ticks;

    #[test]
    fn sting_poison_scales_with_difficulty() {
        assert_eq!(sting_poison_ticks(Difficulty::Peaceful), 0);
        assert_eq!(sting_poison_ticks(Difficulty::Easy), 0);
        assert_eq!(sting_poison_ticks(Difficulty::Normal), 200);
        assert_eq!(sting_poison_ticks(Difficulty::Hard), 360);
    }
}
