use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, AtomicI32, AtomicU64, Ordering},
};

use pumpkin_data::BlockState;
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::{EntityStatus, EntityType};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::bedrock::server::actor_event::ActorEventID;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::math::position::BlockPos;

use crate::entity::{
    Entity, EntityBase,
    ageable::{AgeableData, AgeableMob},
    ai::goal::{
        breed::BreedGoal, escape_danger::EscapeDangerGoal, follow_parent::FollowParentGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal, swim::SwimGoal,
        tempt::TemptGoal, wander_around::WanderAroundGoal,
    },
    custom_sound::CustomSound,
    item::ItemEntity,
    mob::{Mob, MobEntity, drop_from_gift_loot_table},
    passive::animal::Animal,
    player::Player,
};

const ARMADILLO_BRUSH_LOOT_TABLE: &str = "minecraft:brush/armadillo";
const ARMADILLO_SHED_LOOT_TABLE: &str = "minecraft:gameplay/armadillo_shed";
/// Vanilla `armadillo_scare_detected` `MobSensor` scan rate, in ticks.
pub const SCARE_SCAN_INTERVAL: i32 = 5;
/// Vanilla `DANGER_DETECTED_RECENTLY` memory lifetime after a threat is seen.
pub const SCARE_CHECK_INTERVAL: i32 = 80;
pub const SCARE_DISTANCE_HORIZONTAL: f64 = 7.0;
pub const SCARE_DISTANCE_VERTICAL: f64 = 2.0;
/// Vanilla `ArmadilloBallUp.DANGER_DETECTED_RECENTLY_DANGER_THRESHOLD`: a peek needs more danger ticks left than this.
const PEEK_DANGER_THRESHOLD_TICKS: i32 = 75;
/// Vanilla `ArmadilloState.UNROLLING.animationDuration()`: the unroll gate in `ArmadilloBallUp`.
const BALL_UP_UNROLLING_TICKS: i32 = 30;

/// Vanilla `ArmadilloBallUp.pickNextPeekTimer`: scared animation plus 100..=400 ticks.
fn pick_next_peek_timer() -> i32 {
    i32::try_from(ArmadilloState::Scared.animation_duration()).unwrap_or_default()
        + rand::random_range(100..=400)
}

/// Vanilla `ArmadilloBallUp.tick` unroll gates: a scared ball unrolls once the danger timer drops
/// below the unroll animation, and an unrolling ball rolls back up when danger returns above it.
const fn ball_up_transition(state: ArmadilloState, danger_ticks: i32) -> Option<ArmadilloState> {
    match state {
        ArmadilloState::Scared if danger_ticks < BALL_UP_UNROLLING_TICKS => {
            Some(ArmadilloState::Unrolling)
        }
        ArmadilloState::Unrolling if danger_ticks > BALL_UP_UNROLLING_TICKS => {
            Some(ArmadilloState::Scared)
        }
        _ => None,
    }
}

fn pick_next_scute_drop_time() -> i32 {
    let rand_ticks = (rand::random::<u32>() % 6000) as i32;
    rand_ticks + 6000
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum ArmadilloState {
    #[default]
    Idle = 0,
    Rolling = 1,
    Scared = 2,
    Unrolling = 3,
}

impl ArmadilloState {
    #[must_use]
    pub const fn from_id(id: i32) -> Self {
        match id {
            1 => Self::Rolling,
            2 => Self::Scared,
            3 => Self::Unrolling,
            _ => Self::Idle,
        }
    }

    #[must_use]
    pub const fn id(self) -> i32 {
        self as i32
    }

    #[must_use]
    pub const fn is_threatened(self) -> bool {
        !matches!(self, Self::Idle)
    }

    #[must_use]
    pub const fn animation_duration(self) -> u64 {
        match self {
            Self::Idle => 0,
            Self::Rolling => 10,
            Self::Scared => 50,
            Self::Unrolling => 30,
        }
    }

    #[must_use]
    pub const fn should_hide_in_shell(self, ticks_in_state: u64) -> bool {
        match self {
            Self::Idle => false,
            Self::Rolling => ticks_in_state > 5,
            Self::Scared => true,
            Self::Unrolling => ticks_in_state < 26,
        }
    }

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Rolling => "rolling",
            Self::Scared => "scared",
            Self::Unrolling => "unrolling",
        }
    }

    #[must_use]
    pub fn from_name(name: &str) -> Self {
        match name {
            "rolling" => Self::Rolling,
            "scared" => Self::Scared,
            "unrolling" => Self::Unrolling,
            _ => Self::Idle,
        }
    }
}

/// Represents an Armadillo, a passive entity that can roll into a ball when threatened.
///
/// Wiki: <https://minecraft.wiki/w/Armadillo>
pub struct ArmadilloEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: AgeableData,
    pub state: AtomicI32,
    pub in_state_ticks: AtomicU64,
    pub scute_time: AtomicI32,
    pub danger_detected_recently_ticks: AtomicI32,
    /// Vanilla `ArmadilloBallUp.nextPeekTimer`.
    pub next_peek_timer: AtomicI32,
    /// Vanilla `ArmadilloBallUp.dangerWasAround`.
    pub danger_was_around: AtomicBool,
}

impl ArmadilloEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let armadillo = Self {
            mob_entity,
            ageable_data: AgeableData::default(),
            state: AtomicI32::new(ArmadilloState::Idle.id()),
            in_state_ticks: AtomicU64::new(0),
            scute_time: AtomicI32::new(pick_next_scute_drop_time()),
            danger_detected_recently_ticks: AtomicI32::new(0),
            next_peek_timer: AtomicI32::new(0),
            danger_was_around: AtomicBool::new(false),
        };
        let mob_arc = Arc::new(armadillo);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, EscapeDangerGoal::new_environmental(2.0));
            goal_selector.add_goal(2, BreedGoal::new(1.0));
            goal_selector.add_goal(
                3,
                Box::new(TemptGoal::with_tag(
                    1.25,
                    &tag::Item::MINECRAFT_ARMADILLO_FOOD,
                    false,
                )),
            );
            goal_selector.add_goal(4, Box::new(FollowParentGoal::new(1.1)));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(7, Box::new(RandomLookAroundGoal::default()));
        };

        mob_arc
    }

    #[must_use]
    pub fn get_state(&self) -> ArmadilloState {
        ArmadilloState::from_id(self.state.load(Ordering::Relaxed))
    }

    pub fn switch_to_state(&self, state: ArmadilloState) {
        self.state.store(state.id(), Ordering::Relaxed);
        self.in_state_ticks.store(0, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::armadillo::ARMADILLO_STATE,
            VarInt(state.id()),
        );
    }

    #[must_use]
    pub fn is_scared(&self) -> bool {
        self.get_state() != ArmadilloState::Idle
    }

    #[must_use]
    pub fn should_hide_in_shell(&self) -> bool {
        self.get_state()
            .should_hide_in_shell(self.in_state_ticks.load(Ordering::Relaxed))
    }

    #[must_use]
    pub fn should_switch_to_scared_state(&self) -> bool {
        self.get_state() == ArmadilloState::Rolling
            && self.in_state_ticks.load(Ordering::Relaxed)
                > ArmadilloState::Rolling.animation_duration()
    }

    pub fn can_stay_rolled_up(&self) -> bool {
        let entity = self.get_entity();
        // Vanilla `Armadillo.canStayRolledUp` uses `isInLiquid()`, which includes lava.
        !self.is_panicking()
            && !entity.is_in_liquid()
            && !entity.is_leashed()
            && !entity.is_passenger()
            && !entity.is_vehicle()
    }

    pub fn roll_up(&self) {
        if !self.is_scared() {
            // Vanilla Armadillo.rollUp stops in place and emits ENTITY_ACTION.
            self.mob_entity.reset_love_ticks();
            self.mob_entity.stop_navigation();
            let entity = self.get_entity();
            let world = entity.world.load();
            let pos = entity.pos.load();
            world.play_sound(Sound::EntityArmadilloRoll, SoundCategory::Neutral, &pos);
            world.emit_game_event(
                pumpkin_data::game_event::GameEvent::EntityAction.name(),
                pos,
            );
            self.switch_to_state(ArmadilloState::Rolling);
        }
    }

    pub fn roll_out(&self) {
        if self.is_scared() {
            let entity = self.get_entity();
            let world = entity.world.load();
            let pos = entity.pos.load();
            world.play_sound(
                Sound::EntityArmadilloUnrollFinish,
                SoundCategory::Neutral,
                &pos,
            );
            world.emit_game_event(
                pumpkin_data::game_event::GameEvent::EntityAction.name(),
                pos,
            );
            self.switch_to_state(ArmadilloState::Idle);
        }
    }

    /// Vanilla `ArmadilloBallUp.tick` for a threatened ball: the peek timer, then the unroll gates.
    fn tick_ball_up(&self, state: ArmadilloState, danger_ticks: i32) {
        let entity = self.get_entity();
        let world = entity.world.load();
        let mut next_peek = self.next_peek_timer.load(Ordering::Relaxed);
        if next_peek > 0 {
            next_peek -= 1;
        }
        let danger_is_around = danger_ticks > PEEK_DANGER_THRESHOLD_TICKS;
        if danger_is_around
            != self
                .danger_was_around
                .swap(danger_is_around, Ordering::Relaxed)
        {
            next_peek = pick_next_peek_timer();
        }
        if state == ArmadilloState::Scared
            && next_peek == 0
            && danger_is_around
            && entity.on_ground.load(Ordering::Relaxed)
        {
            world.broadcast_entity_event(entity, EntityStatus::ArmadilloPeek, ActorEventID::None);
            next_peek = pick_next_peek_timer();
        }
        self.next_peek_timer.store(next_peek, Ordering::Relaxed);
        if let Some(next) = ball_up_transition(state, danger_ticks) {
            if next == ArmadilloState::Unrolling {
                world.play_sound(
                    Sound::EntityArmadilloUnrollStart,
                    SoundCategory::Neutral,
                    &entity.pos.load(),
                );
            }
            self.switch_to_state(next);
        }
    }

    pub fn brush_off_scute(
        &self,
        _interacting_entity: Option<&dyn EntityBase>,
        tool: &ItemStack,
    ) -> bool {
        if self.is_baby() || !self.get_entity().is_alive() {
            return false;
        }
        let entity = self.get_entity();
        let world = entity.world.load();
        let pos = entity.pos.load();
        if let Some(table) = world.get_loot_table(ARMADILLO_BRUSH_LOOT_TABLE) {
            let params = crate::world::loot::LootContextParameters {
                world: Some(&world),
                position: Some(pos),
                this_entity: Some(self),
                tool: Some(tool.clone()),
                ..Default::default()
            };
            for drop_stack in
                crate::world::loot::generate_loot_from_handle(&table, rand::random(), &params)
            {
                let item_entity = Arc::new(ItemEntity::new(
                    Entity::new(world.clone(), pos, &EntityType::ITEM),
                    drop_stack,
                ));
                world.spawn_entity(item_entity);
            }
        }
        world.play_sound(Sound::EntityArmadilloBrush, SoundCategory::Neutral, &pos);
        world.emit_game_event(
            pumpkin_data::game_event::GameEvent::EntityInteract.name(),
            pos,
        );
        true
    }

    pub fn is_scared_by(&self, living_entity: &dyn EntityBase) -> bool {
        // Vanilla `getBoundingBox().inflate(7, 2, 7).intersects(target.getBoundingBox())`.
        let scan_box = self.get_entity().bounding_box.load().expand(
            SCARE_DISTANCE_HORIZONTAL,
            SCARE_DISTANCE_VERTICAL,
            SCARE_DISTANCE_HORIZONTAL,
        );
        let target_ent = living_entity.get_entity();
        if !scan_box.intersects(&target_ent.bounding_box.load()) {
            return false;
        }

        let target_type = target_ent.entity_type;
        if target_type.has_tag(&tag::EntityType::MINECRAFT_UNDEAD) {
            return true;
        }

        // Vanilla isScaredBy also accepts the mob that last hurt this armadillo.
        if self
            .mob_entity
            .living_entity
            .get_last_hurt_by_mob()
            .is_some_and(|attacker| attacker.get_entity().entity_uuid == target_ent.entity_uuid)
        {
            return true;
        }

        if target_type == &EntityType::PLAYER {
            // Vanilla isScaredBy skips spectators; sprinting or riding scares.
            if !living_entity.is_spectator()
                && (target_ent.is_sprinting() || target_ent.has_vehicle())
            {
                return true;
            }
        }

        false
    }
}

impl AgeableMob for ArmadilloEntity {
    fn get_ageable_data(&self) -> &AgeableData {
        &self.ageable_data
    }

    fn get_baby_start_age(&self) -> i32 {
        // Vanilla Armadillo does not override getBabyStartAge, so AgeableMob's -24000 applies.
        crate::entity::ageable::BABY_START_AGE
    }
}

impl Animal for ArmadilloEntity {
    fn play_eating_sound(&self) {
        self.make_sound(Sound::EntityArmadilloEat);
    }

    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack
            .item
            .has_tag(&tag::Item::MINECRAFT_ARMADILLO_FOOD)
    }

    fn can_fall_in_love(&self) -> bool {
        // Vanilla canFallInLove: a scared armadillo never enters love mode.
        !self.is_scared() && !self.mob_entity.is_in_love()
    }
}

impl CustomSound for ArmadilloEntity {
    fn death_sound(&self) -> Option<Sound> {
        Some(Sound::EntityArmadilloDeath)
    }

    /// Vanilla `getHurtSound` uses the muted variant for the whole scared state (`isScared`).
    fn hurt_sound(&self) -> Option<Sound> {
        self.is_scared()
            .then_some(Sound::EntityArmadilloHurtReduced)
    }
}

impl Mob for ArmadilloEntity {
    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {
        self.play_entity_sound(Sound::EntityArmadilloStep, 0.15, 1.0);
    }

    /// Vanilla `getMaxHeadYRot`: the head locks to the body while scared.
    fn get_max_head_rotation(&self) -> f32 {
        if self.is_scared() { 0.0 } else { 32.0 }
    }

    fn get_ambient_sound(&self) -> Option<Sound> {
        // Vanilla getAmbientSound returns null for the whole scared state.
        (!self.is_scared()).then_some(Sound::EntityArmadilloAmbient)
    }

    fn as_custom_sound(&self) -> Option<&dyn CustomSound> {
        Some(self)
    }

    fn as_ageable(&self) -> Option<&dyn AgeableMob> {
        Some(self)
    }

    fn as_animal(&self) -> Option<&dyn Animal> {
        Some(self)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_string("state", self.get_state().name().to_string());
        nbt.put_int("scute_time", self.scute_time.load(Ordering::Relaxed));
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if let Some(state_name) = nbt.get_string("state") {
            self.switch_to_state(ArmadilloState::from_name(state_name));
        } else if let Some(state_id) = nbt.get_int("state") {
            self.switch_to_state(ArmadilloState::from_id(state_id));
        }
        if let Some(scute_time) = nbt.get_int("scute_time") {
            self.scute_time.store(scute_time, Ordering::Relaxed);
        }
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn modify_incoming_damage(
        &self,
        amount: f32,
        _damage_type: DamageType,
        _source: Option<&dyn EntityBase>,
        _cause: Option<&dyn EntityBase>,
    ) -> f32 {
        // Vanilla `hurtServer` halves damage for the whole scared state, not just while hidden.
        if self.is_scared() {
            (amount - 1.0) / 2.0
        } else {
            amount
        }
    }

    /// Vanilla `Armadillo#actuallyHurt`: a living attacker is remembered for 80 ticks and
    /// triggers `rollUp`; environmental panic damage without a living attacker rolls out.
    fn on_damage(&self, damage_type: DamageType, source: Option<&dyn EntityBase>) {
        // Vanilla `actuallyHurt` skips the reaction entirely for NoAI or dying mobs.
        if self.mob_entity.is_no_ai() || self.mob_entity.living_entity.is_dead_or_dying() {
            return;
        }
        if source.is_some_and(|attacker| attacker.get_living_entity().is_some()) {
            self.danger_detected_recently_ticks
                .store(SCARE_CHECK_INTERVAL, Ordering::Relaxed);
            if self.can_stay_rolled_up() {
                self.roll_up();
            }
        } else if damage_type.has_tag(&tag::DamageType::MINECRAFT_PANIC_ENVIRONMENTAL_CAUSES) {
            self.roll_out();
        }
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        self.in_state_ticks.fetch_add(1, Ordering::Relaxed);
    }

    fn custom_server_ai_step(&self, _caller: &dyn EntityBase) {
        let danger_ticks = self.danger_detected_recently_ticks.load(Ordering::Relaxed);
        if danger_ticks > 0 {
            self.danger_detected_recently_ticks
                .store(danger_ticks - 1, Ordering::Relaxed);
        }

        let entity = self.get_entity();
        let world = entity.world.load();

        // Vanilla keeps counting while shedding is blocked, so a held-back scute drops once allowed.
        if entity.is_alive() {
            let scute_time = self.scute_time.fetch_sub(1, Ordering::Relaxed) - 1;
            if scute_time <= 0 && Mob::should_drop_loot(self) {
                let pos = entity.pos.load();
                if drop_from_gift_loot_table(self, ARMADILLO_SHED_LOOT_TABLE, |drop_stack| {
                    let item_entity = Arc::new(ItemEntity::new(
                        Entity::new(world.clone(), pos, &EntityType::ITEM),
                        drop_stack,
                    ));
                    // Vanilla `spawnAtLocation`: the shed scute is saved with the chunk, like the brush drop.
                    world.spawn_entity(item_entity);
                }) {
                    // Vanilla `Armadillo.java:145-146`: volume 1.0, pitch `(nextFloat - nextFloat) * 0.2 + 1.0`.
                    world.play_sound_fine(
                        Sound::EntityArmadilloScuteDrop,
                        SoundCategory::Neutral,
                        &pos,
                        1.0,
                        (rand::random::<f32>() - rand::random::<f32>()).mul_add(0.2, 1.0),
                    );
                    world.emit_game_event(
                        pumpkin_data::game_event::GameEvent::EntityPlace.name(),
                        pos,
                    );
                }
                self.scute_time
                    .store(pick_next_scute_drop_time(), Ordering::Relaxed);
            }
        }

        // Vanilla `armadillo_scare_detected` (MobSensor, scan rate 5) rescans while the armadillo
        // can stay rolled up, even when already rolled, and refreshes DANGER_DETECTED_RECENTLY
        // for 80 ticks on a hit. Rolled-up armadillos therefore stay curled while threatened.
        if self.can_stay_rolled_up()
            && entity.age.load(Ordering::Relaxed) % SCARE_SCAN_INTERVAL == 0
        {
            // Vanilla scans `getBoundingBox().inflate(7, 2, 7)`, so corners count too.
            let scan_box = entity.bounding_box.load().expand(
                SCARE_DISTANCE_HORIZONTAL,
                SCARE_DISTANCE_VERTICAL,
                SCARE_DISTANCE_HORIZONTAL,
            );
            let threatened = world.get_all_at_box(&scan_box).iter().any(|candidate| {
                candidate.get_entity().is_alive() && self.is_scared_by(candidate.as_ref())
            });
            if threatened {
                self.danger_detected_recently_ticks
                    .store(SCARE_CHECK_INTERVAL, Ordering::Relaxed);
                self.roll_up();
            }
        }

        let state = self.get_state();
        let ticks_in_state = self.in_state_ticks.load(Ordering::Relaxed);

        match state {
            ArmadilloState::Rolling => {
                if ticks_in_state > ArmadilloState::Rolling.animation_duration() {
                    self.switch_to_state(ArmadilloState::Scared);
                }
            }
            ArmadilloState::Scared | ArmadilloState::Unrolling => {
                let danger_ticks = self.danger_detected_recently_ticks.load(Ordering::Relaxed);
                // Vanilla ARMADILLO_ROLLING_OUT: rolls out once DANGER_DETECTED_RECENTLY expires.
                if !self.can_stay_rolled_up() || danger_ticks == 0 {
                    self.roll_out();
                } else {
                    self.tick_ball_up(state, danger_ticks);
                }
            }
            ArmadilloState::Idle => {}
        }
    }

    fn mob_init_data_tracker(&self) {
        // The baby flag is synced by AgeableMob::set_age on every age boundary crossing, so
        // only the armadillo-specific state needs initializing here.
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::armadillo::ARMADILLO_STATE,
            VarInt(self.get_state().id()),
        );
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        if item_stack.item == &Item::BRUSH
            && self.brush_off_scute(Some(player.as_ref()), item_stack)
        {
            if player.gamemode.load() != pumpkin_util::GameMode::Creative {
                let _ = item_stack.damage_item(16);
            }
            return true;
        }
        if self.is_scared() {
            return false;
        }
        self.animal_interact(player, item_stack)
    }
}

// Generated tracked-data aliases must stay in sync: the armadillo alias and the ageable
// `set_age` path refer to the same synced-data index.
const _: () = assert!(
    pumpkin_data::tracked_data::armadillo::BABY_ID.id.0
        == pumpkin_data::tracked_data::ageable_mob::DATA_BABY_ID.id.0,
    "armadillo BABY_ID diverged from ageable DATA_BABY_ID"
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::ai::test_support::world;
    use pumpkin_util::math::vector3::Vector3;

    #[tokio::test]
    async fn lava_contact_prevents_staying_rolled_up() -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let armadillo = ArmadilloEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.0, 64.0, 0.0),
            &EntityType::ARMADILLO,
        ));
        assert!(armadillo.can_stay_rolled_up());
        armadillo
            .mob_entity
            .living_entity
            .entity
            .touching_lava
            .store(true, Ordering::SeqCst);
        let rolled_up_in_lava = armadillo.can_stay_rolled_up();
        world.level.shutdown().await;
        assert!(!rolled_up_in_lava);
        Ok(())
    }

    #[test]
    fn ball_up_unroll_gates_match_vanilla_threshold() {
        use ArmadilloState::{Scared, Unrolling};
        assert_eq!(ball_up_transition(Scared, BALL_UP_UNROLLING_TICKS), None);
        assert_eq!(
            ball_up_transition(Scared, BALL_UP_UNROLLING_TICKS - 1),
            Some(Unrolling)
        );
        assert_eq!(ball_up_transition(Unrolling, BALL_UP_UNROLLING_TICKS), None);
        assert_eq!(
            ball_up_transition(Unrolling, BALL_UP_UNROLLING_TICKS + 1),
            Some(Scared)
        );
    }

    #[test]
    fn state_ids_and_names_match_vanilla_order() {
        let cases = [
            (ArmadilloState::Idle, 0, "idle"),
            (ArmadilloState::Rolling, 1, "rolling"),
            (ArmadilloState::Scared, 2, "scared"),
            (ArmadilloState::Unrolling, 3, "unrolling"),
        ];
        for (state, id, name) in cases {
            assert_eq!(state.id(), id);
            assert_eq!(ArmadilloState::from_id(id), state);
            assert_eq!(state.name(), name);
            assert_eq!(ArmadilloState::from_name(name), state);
        }
    }

    #[test]
    fn shell_thresholds_match_vanilla() {
        use ArmadilloState::*;
        assert!(!Idle.should_hide_in_shell(100));
        assert!(!Rolling.should_hide_in_shell(5));
        assert!(Rolling.should_hide_in_shell(6));
        assert!(Scared.should_hide_in_shell(0));
        assert!(Unrolling.should_hide_in_shell(25));
        assert!(!Unrolling.should_hide_in_shell(26));
    }
}
