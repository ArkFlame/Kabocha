use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicI32, AtomicU8, Ordering},
};

use pumpkin_data::damage::DamageType;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::potion::Effect;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::{Block, BlockState};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use rand::RngExt;

use crate::block::entities::beehive::BeehiveBlockEntity;
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
        go_to_known_flower::GoToKnownFlowerGoal, grow_crop::GrowCropGoal,
        melee_attack::MeleeAttackGoal, pollinate_flower::PollinateFlowerGoal,
        reset_universal_anger::ResetUniversalAngerGoal, revenge::RevengeGoal, swim::SwimGoal,
        tempt::TemptGoal,
    },
    ai::pathfinder::{Navigator, node::PathType},
    mob::{
        Mob, MobEntity,
        neutral::{NeutralData, NeutralMob},
    },
    passive::animal::Animal,
    player::Player,
};

use crate::entity::ai::goal::bee_wander::BeeWanderGoal;
use crate::entity::ai::goal::hive::{EnterHiveGoal, GoToHiveGoal, LocateHiveGoal};

pub const FLAG_ROLL: u8 = 2;
pub const FLAG_HAS_STUNG: u8 = 4;
pub const FLAG_HAS_NECTAR: u8 = 8;

/// Vanilla `Bee.TICKS_PER_FLAP`, `Mth.ceil(1.4959966F)`.
const TICKS_PER_FLAP: i32 = 2;

/// Vanilla `Bee.TICKS_WITHOUT_NECTAR_BEFORE_GOING_HOME`.
pub const TICKS_WITHOUT_NECTAR_BEFORE_GOING_HOME: i32 = 3600;

/// Vanilla `Bee.isTiredOfLookingForNectar`. Strict `>`, matching the decompiled check:
/// a bee is only tired once the counter exceeds 3600, not when it equals it.
#[must_use]
pub const fn tired_of_looking_for_nectar(ticks_without_nectar: i32) -> bool {
    ticks_without_nectar > TICKS_WITHOUT_NECTAR_BEFORE_GOING_HOME
}

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

/// Vanilla `Bee.wantsToEnterHive` early-outs: while still staying out, pollinating, having
/// stung or holding a target, a bee never returns to its hive.
#[must_use]
pub const fn wants_to_enter_hive_gate(
    stay_out_of_hive_ticks: i32,
    is_pollinating: bool,
    has_stung: bool,
    has_target: bool,
) -> bool {
    stay_out_of_hive_ticks <= 0 && !is_pollinating && !has_stung && !has_target
}

/// Vanilla `Bee.isHiveNearFire`: a fire block in the 3x3x3 around the hive keeps the bee out.
/// The predicate keeps the neighbourhood check pure and unit-testable.
#[must_use]
fn hive_near_fire(hive_pos: BlockPos, is_fire: impl Fn(BlockPos) -> bool) -> bool {
    for dx in -1..=1 {
        for dy in -1..=1 {
            for dz in -1..=1 {
                let pos = BlockPos::new(hive_pos.0.x + dx, hive_pos.0.y + dy, hive_pos.0.z + dz);
                if is_fire(pos) {
                    return true;
                }
            }
        }
    }
    false
}

/// Vanilla `BeePollinateGoal.canBeeUse`: no nectar and no flower-search cooldown.
fn can_pollinate(mob: &dyn Mob) -> bool {
    mob.cast_any()
        .downcast_ref::<BeeEntity>()
        .is_some_and(|bee| !bee.has_nectar() && bee.flower_search_cooldown() == 0)
}

/// Vanilla `BeePollinateGoal.canBeeUse` when no flower is reachable: retry in 20-60 ticks.
fn no_flower_found(mob: &dyn Mob) {
    if let Some(bee) = mob.cast_any().downcast_ref::<BeeEntity>() {
        bee.reset_flower_search_cooldown();
    }
}

/// Vanilla `BeePollinateGoal.start` sets the `pollinating` flag and resets the no-nectar timer.
fn pollination_started(mob: &dyn Mob, flower: BlockPos) {
    if let Some(bee) = mob.cast_any().downcast_ref::<BeeEntity>() {
        bee.set_pollinating(true);
        // Vanilla `BeePollinateGoal.canBeeUse` saves the found flower block on the bee.
        bee.set_flower_pos(Some(flower));
        // Vanilla `Bee.resetTicksWithoutNectarSinceExitingHive` at pollination start
        // (`Bee.java:1176`); matters for a bee loaded from NBT with a stale timer.
        bee.ticks_without_nectar.store(0, Ordering::Relaxed);
    }
}

/// Vanilla `BeePollinateGoal.stop`: clear `pollinating` and wait 200 ticks before another flower.
fn pollination_stopped(mob: &dyn Mob) {
    if let Some(bee) = mob.cast_any().downcast_ref::<BeeEntity>() {
        bee.set_pollinating(false);
        bee.set_flower_search_cooldown(200);
    }
}

fn record_nectar(mob: &dyn Mob) {
    if let Some(bee) = mob.cast_any().downcast_ref::<BeeEntity>() {
        // Vanilla `Bee.setHasNectar(true)` resets the no-nectar timer. The flower was
        // already remembered at pollination start, matching vanilla `savedFlowerPos`.
        bee.set_has_nectar(true);
    }
}

/// Vanilla `BlockPos.CODEC` stores a hive or flower position as an `X`/`Y`/`Z` compound.
fn write_block_pos(nbt: &mut NbtCompound, key: &str, pos: BlockPos) {
    let mut tag = NbtCompound::new();
    tag.put_int("X", pos.0.x);
    tag.put_int("Y", pos.0.y);
    tag.put_int("Z", pos.0.z);
    nbt.put_compound(key, tag);
}

/// Reads the vanilla compound or the legacy int-array form of a saved position.
fn read_block_pos(nbt: &NbtCompound, key: &str) -> Option<BlockPos> {
    if let Some(tag) = nbt.get_compound(key) {
        let x = tag.get_int("X").or_else(|| tag.get_int("x"))?;
        let y = tag.get_int("Y").or_else(|| tag.get_int("y"))?;
        let z = tag.get_int("Z").or_else(|| tag.get_int("z"))?;
        return Some(BlockPos::new(x, y, z));
    }
    let arr = nbt.get_int_array(key)?;
    (arr.len() == 3).then(|| BlockPos::new(arr[0], arr[1], arr[2]))
}

/// Vanilla `Bee.MAX_CROPS_GROWN` per pollination cycle.
const MAX_CROPS_GROWN_SINCE_POLLINATION: i32 = 10;

/// Vanilla `BeeGrowCropGoal.canBeeUse` stops growing once the cap is reached.
#[must_use]
pub const fn has_crop_growth_budget(crops_grown: i32) -> bool {
    crops_grown < MAX_CROPS_GROWN_SINCE_POLLINATION
}

/// Vanilla `BeeGrowCropGoal` only runs while the bee carries nectar, has growth budget left,
/// and remembers a live hive (`Bee.isHiveValid`).
fn can_grow_crop(mob: &dyn Mob) -> bool {
    let Some(bee) = mob.cast_any().downcast_ref::<BeeEntity>() else {
        return false;
    };
    if !bee.has_nectar() || !has_crop_growth_budget(bee.crops_grown_since_pollination()) {
        return false;
    }
    bee.is_hive_valid(&mob.get_entity().world.load())
}

/// Counts a fertilized crop.
///
/// Vanilla keeps `HasNectar` after the cap so the bee can deliver it to its hive;
/// `canBeeUse` alone stops further growth at ten crops.
fn record_crop_grown(mob: &dyn Mob) {
    if let Some(bee) = mob.cast_any().downcast_ref::<BeeEntity>() {
        bee.record_crop_grown();
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
    pub hive_search_cooldown: AtomicI32,
    pub crops_grown_since_pollination: AtomicI32,
    pub time_since_sting: AtomicI32,
    pub hive_pos: Mutex<Option<BlockPos>>,
    pub flower_pos: Mutex<Option<BlockPos>>,
    pub flower_search_cooldown: AtomicI32,
    pub is_pollinating: AtomicBool,
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
            hive_search_cooldown: AtomicI32::new(0),
            crops_grown_since_pollination: AtomicI32::new(0),
            time_since_sting: AtomicI32::new(0),
            hive_pos: Mutex::new(None),
            flower_pos: Mutex::new(None),
            flower_search_cooldown: AtomicI32::new(rand::rng().random_range(20..=60)),
            is_pollinating: AtomicBool::new(false),
        };
        let mob_arc = Arc::new(bee);

        // Vanilla `Bee.defineSynchedData` registers the anger end time with the
        // -1 default the client assumes. Define at construction, before NBT read,
        // so a loaded angry value set later becomes non-default and is spawned.
        mob_arc
            .get_entity()
            .synched_data
            .define(pumpkin_data::tracked_data::bee::DATA_ANGER_END_TIME, -1i64);

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;

            goal_selector.add_goal(
                0,
                Box::new(MeleeAttackGoal::new(1.4, true).gated_by(can_sting)),
            );
            // Vanilla Bee.registerGoals: enter hive 1, locate hive / go to hive 5.
            goal_selector.add_goal(1, Box::new(EnterHiveGoal));
            goal_selector.add_goal(5, Box::new(LocateHiveGoal));
            goal_selector.add_goal(5, Box::new(GoToHiveGoal::default()));
            // Vanilla `BeePollinateGoal` priority 4.
            goal_selector.add_goal(
                4,
                Box::new(PollinateFlowerGoal::new(
                    can_pollinate,
                    record_nectar,
                    no_flower_found,
                    pollination_stopped,
                    pollination_started,
                )),
            );
            // Vanilla `BeeGoToKnownFlowerGoal` priority 6.
            goal_selector.add_goal(6, Box::new(GoToKnownFlowerGoal::new()));
            // Vanilla `BeeGrowCropGoal` priority 7 (`Bee.java:201`).
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
            // Vanilla BeeWanderGoal at priority 8: water-avoiding hover flight toward hive.
            goal_selector.add_goal(8, Box::new(BeeWanderGoal::default()));
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
        // Vanilla `Bee.setHasNectar` resets the no-nectar timer on gain.
        if val {
            self.ticks_without_nectar.store(0, Ordering::Relaxed);
        }
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

    #[must_use]
    pub fn hive_pos(&self) -> Option<BlockPos> {
        *self
            .hive_pos
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub fn set_hive_pos(&self, pos: Option<BlockPos>) {
        *self
            .hive_pos
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = pos;
    }

    #[must_use]
    pub fn hive_search_cooldown(&self) -> i32 {
        self.hive_search_cooldown.load(Ordering::Relaxed)
    }

    pub fn set_hive_search_cooldown(&self, ticks: i32) {
        self.hive_search_cooldown.store(ticks, Ordering::Relaxed);
    }

    /// Vanilla Bee.wantsToEnterHive.
    pub fn wants_to_enter_hive(&self, mob: &dyn Mob) -> bool {
        if !wants_to_enter_hive_gate(
            self.cannot_enter_hive_ticks.load(Ordering::Relaxed),
            self.is_pollinating(),
            self.has_stung(),
            mob.get_mob_entity().get_target().is_some(),
        ) {
            return false;
        }
        let world = mob.get_entity().world.load();
        let pos = mob.get_entity().block_pos.load();
        // Vanilla reads the data-driven BEES_STAY_IN_HIVE attribute (keyframed night plus rain).
        let wants = self.has_nectar()
            || self.is_tired_of_looking_for_nectar()
            || world.bees_stay_in_hive(&pos);
        // Vanilla returns `wants && !isHiveNearFire()`; a bee with no hive is never near fire.
        wants
            && !self.hive_pos().is_some_and(|hive| {
                hive_near_fire(hive, |fire_pos| world.get_block(&fire_pos) == &Block::FIRE)
            })
    }

    #[must_use]
    pub fn flower_pos(&self) -> Option<BlockPos> {
        *self
            .flower_pos
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub fn set_flower_pos(&self, pos: Option<BlockPos>) {
        *self
            .flower_pos
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = pos;
    }

    #[must_use]
    pub fn ticks_without_nectar(&self) -> i32 {
        self.ticks_without_nectar.load(Ordering::Relaxed)
    }

    #[must_use]
    pub fn flower_search_cooldown(&self) -> i32 {
        self.flower_search_cooldown.load(Ordering::Relaxed)
    }

    pub fn set_flower_search_cooldown(&self, ticks: i32) {
        self.flower_search_cooldown.store(ticks, Ordering::Relaxed);
    }

    /// Vanilla `Bee.remainingCooldownBeforeLocatingNewFlower = random(20, 60)`.
    pub fn reset_flower_search_cooldown(&self) {
        self.set_flower_search_cooldown(rand::rng().random_range(20..=60));
    }

    /// Vanilla `Bee.dropFlower`: forget the flower and retry a search shortly.
    pub fn drop_flower(&self) {
        self.set_flower_pos(None);
        self.reset_flower_search_cooldown();
    }

    /// Vanilla `Bee.isTiredOfLookingForNectar`, read by the hive goals to trigger the return home.
    #[must_use]
    pub fn is_tired_of_looking_for_nectar(&self) -> bool {
        tired_of_looking_for_nectar(self.ticks_without_nectar.load(Ordering::Relaxed))
    }

    /// Vanilla `BeePollinateGoal.pollinating`, which blocks a return to the hive.
    #[must_use]
    pub fn is_pollinating(&self) -> bool {
        self.is_pollinating.load(Ordering::Relaxed)
    }

    pub fn set_pollinating(&self, val: bool) {
        self.is_pollinating.store(val, Ordering::Relaxed);
    }

    /// Vanilla `Bee.isHiveValid`: the remembered hive must still have a block entity.
    fn is_hive_valid(&self, world: &Arc<crate::world::World>) -> bool {
        self.hive_pos().is_some_and(|pos| {
            world
                .get_block_entity(&pos)
                .is_some_and(|entity| entity.as_any().is::<BeehiveBlockEntity>())
        })
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
        if let Some(pos) = self.hive_pos() {
            write_block_pos(nbt, "hive_pos", pos);
        }
        if let Some(pos) = self.flower_pos() {
            write_block_pos(nbt, "flower_pos", pos);
        }
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
        if let Some(pos) = read_block_pos(nbt, "hive_pos") {
            self.set_hive_pos(Some(pos));
        }
        if let Some(pos) = read_block_pos(nbt, "flower_pos") {
            self.set_flower_pos(Some(pos));
        }
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
        // return reads this at 3600; pollination resets it when nectar is gathered.
        if !self.has_nectar() {
            self.ticks_without_nectar.fetch_add(1, Ordering::Relaxed);
        }
        // Vanilla `Bee.aiStep` decrements `stayOutOfHiveCountdown` every server tick.
        if self.cannot_enter_hive_ticks.load(Ordering::Relaxed) > 0 {
            self.cannot_enter_hive_ticks.fetch_sub(1, Ordering::Relaxed);
        }
        // Vanilla Bee.aiStep decrements remainingCooldownBeforeLocatingNewHive.
        if self.hive_search_cooldown.load(Ordering::Relaxed) > 0 {
            self.hive_search_cooldown.fetch_sub(1, Ordering::Relaxed);
        }
        // Vanilla `Bee.aiStep` decrements the flower-search cooldown every server tick.
        if self.flower_search_cooldown() > 0 {
            self.flower_search_cooldown.fetch_sub(1, Ordering::Relaxed);
        }
        // Vanilla `Bee.aiStep` rolls when angry and close to the current target.
        let should_roll = is_angry(self)
            && !self.has_stung()
            && self.mob_entity.get_target().is_some_and(|target| {
                target
                    .get_entity()
                    .pos
                    .load()
                    .squared_distance_to_vec(&self.get_entity().pos.load())
                    < 4.0
            });
        self.set_rolling(should_roll);
        // Vanilla `Bee.aiStep` forgets an invalid hive every 20 ticks.
        let world = self.get_entity().world.load();
        if self.get_entity().tick_count.load(Ordering::Relaxed) % 20 == 0
            && !self.is_hive_valid(&world)
        {
            self.set_hive_pos(None);
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
    use pumpkin_nbt::compound::NbtCompound;
    use pumpkin_nbt::tag::NbtTag;
    use pumpkin_util::Difficulty;
    use pumpkin_util::math::position::BlockPos;

    use super::{
        has_crop_growth_budget, hive_near_fire, read_block_pos, sting_poison_ticks,
        tired_of_looking_for_nectar, wants_to_enter_hive_gate, write_block_pos,
    };

    #[test]
    fn sting_poison_scales_with_difficulty() {
        assert_eq!(sting_poison_ticks(Difficulty::Peaceful), 0);
        assert_eq!(sting_poison_ticks(Difficulty::Easy), 0);
        assert_eq!(sting_poison_ticks(Difficulty::Normal), 200);
        assert_eq!(sting_poison_ticks(Difficulty::Hard), 360);
    }

    #[test]
    fn saved_positions_round_trip_as_vanilla_compounds() {
        let mut nbt = NbtCompound::new();
        let hive = BlockPos::new(12, 64, -7);
        let flower = BlockPos::new(-3, 70, 22);
        write_block_pos(&mut nbt, "hive_pos", hive);
        write_block_pos(&mut nbt, "flower_pos", flower);
        assert_eq!(read_block_pos(&nbt, "hive_pos"), Some(hive));
        assert_eq!(read_block_pos(&nbt, "flower_pos"), Some(flower));
        assert_eq!(read_block_pos(&nbt, "unset"), None);
    }

    #[test]
    fn saved_positions_accept_legacy_int_arrays() {
        let mut nbt = NbtCompound::new();
        nbt.put("hive_pos", NbtTag::IntArray(vec![1, 2, 3]));
        assert_eq!(
            read_block_pos(&nbt, "hive_pos"),
            Some(BlockPos::new(1, 2, 3))
        );
    }

    #[test]
    fn tired_of_looking_for_nectar_boundary() {
        assert!(!tired_of_looking_for_nectar(0));
        assert!(!tired_of_looking_for_nectar(3599));
        assert!(!tired_of_looking_for_nectar(3600));
        assert!(tired_of_looking_for_nectar(3601));
        assert!(tired_of_looking_for_nectar(7200));
    }

    #[test]
    fn crop_growth_budget_stops_at_ten() {
        assert!(has_crop_growth_budget(0));
        assert!(has_crop_growth_budget(9));
        assert!(!has_crop_growth_budget(10));
        assert!(!has_crop_growth_budget(11));
    }

    #[test]
    fn pollinating_bee_does_not_want_to_enter_hive() {
        assert!(wants_to_enter_hive_gate(0, false, false, false));
        assert!(!wants_to_enter_hive_gate(1, false, false, false));
        assert!(!wants_to_enter_hive_gate(0, true, false, false));
        assert!(!wants_to_enter_hive_gate(0, false, true, false));
        assert!(!wants_to_enter_hive_gate(0, false, false, true));
    }

    #[test]
    fn hive_near_fire_checks_only_the_immediate_neighborhood() {
        let hive = BlockPos::new(10, 64, -7);
        assert!(hive_near_fire(hive, |pos| pos == hive));
        assert!(hive_near_fire(hive, |pos| pos == BlockPos::new(11, 63, -6)));
        assert!(!hive_near_fire(hive, |pos| pos == BlockPos::new(13, 64, -7)));
        assert!(!hive_near_fire(hive, |_| false));
    }
}
