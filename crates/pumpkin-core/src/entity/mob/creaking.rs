mod ai;
mod home;

use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicI32, Ordering},
};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::damage::DamageType;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::{Block, BlockState};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::java::client::play::CEntityStatus;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use crate::block::entities::creaking_heart::CreakingHeartBlockEntity;
use crate::entity::{
    Entity, EntityBase,
    ai::{
        brain::{
            Brain, BrainTick,
            memory::{
                PackedMemories,
                codec::{block_pos_from_tag, block_pos_to_tag},
            },
        },
        pathfinder::{node::PathType, walk_node_evaluator::HomeRestriction},
    },
    mob::{Mob, MobEntity},
    player::Player,
};
use crate::world::{BlockEntityLookup, World};

pub const ATTACK_ANIMATION_DURATION: i32 = 15;
pub const MAX_HEALTH: f32 = 1.0;
pub const ATTACK_DAMAGE: f32 = 3.0;
pub const FOLLOW_RANGE: f32 = 32.0;
pub const ACTIVATION_RANGE_SQ: f64 = 144.0;
/// Vanilla `Creaking.checkCanMove` passes coneSize 0.5 with `adjustForDistance=false`.
pub const GAZE_CONE_SIZE: f64 = 0.5;
pub const ATTACK_INTERVAL: i32 = 40;
pub const MOVEMENT_SPEED_WHEN_FIGHTING: f32 = 0.4;
pub const SPEED_MULTIPLIER_WHEN_IDLING: f32 = 0.3;
pub const CREAKING_ORANGE: i32 = 16545810;
pub const CREAKING_GRAY: i32 = 6250335;
pub const INVULNERABILITY_ANIMATION_DURATION: i32 = 8;
pub const TWITCH_DEATH_DURATION: i32 = 45;
pub const MAX_PLAYER_STUCK_COUNTER: i32 = 4;
/// Vanilla `LivingEntity.MAX_GAZE_RANGE`: a player farther than this cannot activate a Creaking.
const MAX_GAZE_RANGE: f64 = 128.0;
/// Vanilla `LivingEntity.isLookingAtMe` ignores gaze targets closer than this epsilon.
const GAZE_MIN_DISTANCE: f64 = 1.0e-5;
/// Vanilla `CreakingAi` adds two idle ticks per tick in bright light so it despawns faster.
const BRIGHTNESS_FOR_FASTER_DESPAWN: f32 = 0.5;

/// Which NBT shape the heart home was written in. A legacy compound home keeps its
/// format until the heart chunk loads and the heart can adopt the binding.
#[derive(Clone, Copy, PartialEq, Eq)]
enum HeartHome {
    Current(BlockPos),
    Legacy(BlockPos),
}

impl HeartHome {
    const fn pos(self) -> BlockPos {
        match self {
            Self::Current(pos) | Self::Legacy(pos) => pos,
        }
    }
}

/// A melee hit reserved during the brain tick and resolved on the following tick,
/// after plugin damage callbacks can no longer re-enter the held brain lock.
struct PendingMeleeAttack {
    target_id: i32,
}

const fn gaze_distance_in_range(distance: f64) -> bool {
    distance >= GAZE_MIN_DISTANCE && distance <= MAX_GAZE_RANGE
}

/// Vanilla `Creaking.hurtServer`: a heart-bound creaking only starts its shake for a
/// living or projectile attacker, or for a player who is responsible for the damage.
#[must_use]
pub const fn heart_shake_allowed(has_direct_attacker: bool, has_responsible_player: bool) -> bool {
    has_direct_attacker || has_responsible_player
}

impl crate::entity::custom_sound::CustomSound for CreakingEntity {
    fn death_sound(&self) -> Option<Sound> {
        Some(Sound::EntityCreakingDeath)
    }

    /// Vanilla `Creaking.getHurtSound`: heart-bound hurt is the sway, otherwise the default.
    fn hurt_sound(&self) -> Option<Sound> {
        self.is_heart_bound().then_some(Sound::EntityCreakingSway)
    }
}

// Vanilla gates every Creaking activity (wander, melee, look, navigation) on canMove.
fn creaking_can_move(mob: &dyn Mob) -> bool {
    mob.cast_any()
        .downcast_ref::<CreakingEntity>()
        .is_some_and(CreakingEntity::can_move)
}

// Vanilla `CreakingAi` IDLE `StartAttacking` is gated on `isActive`, so a dormant creaking
// wanders but never acquires an attack target on its own. `activate()` assigns the target
// directly, so only the automatic nearest-player search needs this gate.
fn creaking_is_active(mob: &dyn Mob) -> bool {
    mob.cast_any()
        .downcast_ref::<CreakingEntity>()
        .is_some_and(CreakingEntity::is_active)
}

pub struct CreakingEntity {
    pub mob_entity: MobEntity,
    can_move: AtomicBool,
    is_active: AtomicBool,
    is_tearing_down: AtomicBool,
    home_pos: HomeRestriction,
    home_binding: AtomicCell<Option<HeartHome>>,
    home_chunk_ticket: Mutex<Option<home::HeartChunkTicket>>,
    pending_home_feedback: AtomicCell<Option<(BlockPos, bool)>>,
    pending_melee_attack: Mutex<Option<PendingMeleeAttack>>,
    attack_animation_remaining_ticks: AtomicI32,
    invulnerability_animation_remaining_ticks: AtomicI32,
    player_stuck_counter: AtomicI32,
    death_time: AtomicI32,
}

impl CreakingEntity {
    fn set_navigation_malus(mob_entity: &mut MobEntity, home_pos: &HomeRestriction) {
        let navigator = mob_entity
            .navigator
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        navigator.set_pathfinding_malus(PathType::DamageOther, 8.0);
        navigator.set_pathfinding_malus(PathType::PowderSnow, 8.0);
        navigator.set_pathfinding_malus(PathType::Lava, 8.0);
        navigator.set_pathfinding_malus(PathType::DamageFire, 0.0);
        navigator.set_pathfinding_malus(PathType::DangerFire, 0.0);
        // Vanilla `Creaking.createNavigation`: `HomeNodeEvaluator` reads the live home position.
        navigator.set_home_restriction(Some(home_pos.clone()));
    }

    pub fn new(entity: Entity) -> Arc<Self> {
        let mut mob_entity = MobEntity::new(entity);
        // Vanilla `Creaking` keeps it away from damaging blocks, powder snow and lava, and
        // avoids fire entirely.
        let home_pos: HomeRestriction = Arc::new(AtomicCell::new(None));
        Self::set_navigation_malus(&mut mob_entity, &home_pos);
        let creaking = Self {
            mob_entity,
            can_move: AtomicBool::new(true),
            is_active: AtomicBool::new(false),
            is_tearing_down: AtomicBool::new(false),
            home_pos,
            home_binding: AtomicCell::new(None),
            home_chunk_ticket: Mutex::new(None),
            pending_home_feedback: AtomicCell::new(None),
            pending_melee_attack: Mutex::new(None),
            attack_animation_remaining_ticks: AtomicI32::new(0),
            invulnerability_animation_remaining_ticks: AtomicI32::new(0),
            player_stuck_counter: AtomicI32::new(0),
            death_time: AtomicI32::new(0),
        };

        // Initialize attributes
        {
            let mut attributes = creaking
                .mob_entity
                .living_entity
                .attributes
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            if let Some(health) = attributes.get_mut(&Attributes::MAX_HEALTH.id) {
                health.base_value = f64::from(MAX_HEALTH);
                health.dirty.store(true, Ordering::Relaxed);
            }
            if let Some(speed) = attributes.get_mut(&Attributes::MOVEMENT_SPEED.id) {
                speed.base_value = f64::from(MOVEMENT_SPEED_WHEN_FIGHTING);
                speed.dirty.store(true, Ordering::Relaxed);
            }
            if let Some(damage) = attributes.get_mut(&Attributes::ATTACK_DAMAGE.id) {
                damage.base_value = f64::from(ATTACK_DAMAGE);
                damage.dirty.store(true, Ordering::Relaxed);
            }
            if let Some(follow) = attributes.get_mut(&Attributes::FOLLOW_RANGE.id) {
                follow.base_value = f64::from(FOLLOW_RANGE);
                follow.dirty.store(true, Ordering::Relaxed);
            }
            if let Some(step) = attributes.get_mut(&Attributes::STEP_HEIGHT.id) {
                step.base_value = 1.0625;
                step.dirty.store(true, Ordering::Relaxed);
            }
        }
        creaking.mob_entity.living_entity.health.store(MAX_HEALTH);

        Arc::new(creaking)
    }

    pub fn set_transient(&self, pos: BlockPos) {
        self.set_heart_home(Some(HeartHome::Current(pos)));
    }

    pub fn is_heart_bound(&self) -> bool {
        self.get_home_pos().is_some()
    }

    pub fn can_move(&self) -> bool {
        self.can_move.load(Ordering::Relaxed)
    }

    pub fn set_can_move(&self, can_move: bool) {
        self.can_move.store(can_move, Ordering::Relaxed);
        self.mob_entity
            .living_entity
            .entity
            .set_synced_data(pumpkin_data::tracked_data::creaking::CAN_MOVE, can_move);
    }

    pub fn is_active(&self) -> bool {
        self.is_active.load(Ordering::Relaxed)
    }

    pub fn set_is_active(&self, active: bool) {
        self.is_active.store(active, Ordering::Relaxed);
        self.mob_entity
            .living_entity
            .entity
            .set_synced_data(pumpkin_data::tracked_data::creaking::IS_ACTIVE, active);
    }

    pub fn is_tearing_down(&self) -> bool {
        self.is_tearing_down.load(Ordering::Relaxed)
    }

    pub fn set_tearing_down(&self) {
        self.is_tearing_down.store(true, Ordering::Relaxed);
        self.mob_entity
            .living_entity
            .entity
            .set_synced_data(pumpkin_data::tracked_data::creaking::IS_TEARING_DOWN, true);
    }

    pub fn get_home_pos(&self) -> Option<BlockPos> {
        self.home_pos.load()
    }

    pub fn set_home_pos(&self, pos: Option<BlockPos>) {
        self.set_heart_home(pos.map(HeartHome::Current));
    }

    fn set_heart_home(&self, home: Option<HeartHome>) {
        let pos = home.map(HeartHome::pos);
        if self.home_binding.swap(home).map(HeartHome::pos) != pos {
            self.release_home_ticket();
            self.pending_home_feedback.store(None);
        }
        self.home_pos.store(pos);
        if let Some(p) = pos {
            self.mob_entity.position_target.store(p);
            self.mob_entity
                .position_target_range
                .store(32, Ordering::Relaxed);
        } else {
            self.mob_entity
                .position_target_range
                .store(-1, Ordering::Relaxed);
        }
        self.mob_entity
            .living_entity
            .entity
            .set_synced_data(pumpkin_data::tracked_data::creaking::HOME_POS, pos);
    }

    fn release_home_ticket(&self) {
        self.home_chunk_ticket
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
    }

    fn request_home_chunk(&self, world: &World, home: BlockPos) {
        let chunk = home.chunk_position();
        let mut ticket = self
            .home_chunk_ticket
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if self.get_entity().is_removed() || self.get_home_pos() != Some(home) {
            return;
        }
        if ticket
            .as_ref()
            .is_none_or(|ticket| ticket.chunk != chunk || !Arc::ptr_eq(&ticket.level, &world.level))
        {
            *ticket = Some(home::HeartChunkTicket::new(world.level.clone(), chunk));
        }
    }

    fn migrate_legacy_home(&self, home: BlockPos) {
        if self.home_binding.load() == Some(HeartHome::Legacy(home)) {
            self.set_heart_home(Some(HeartHome::Current(home)));
        }
    }

    fn apply_pending_heart_feedback(&self, world: &Arc<World>, home: BlockPos) {
        let Some((pending_home, hit_by_player)) = self.pending_home_feedback.load() else {
            return;
        };
        if pending_home != home
            || self
                .pending_home_feedback
                .compare_exchange(Some((pending_home, hit_by_player)), None)
                .is_err()
        {
            return;
        }
        let Some(block_entity) = world.get_block_entity(&home) else {
            return;
        };
        let Some(heart) = block_entity
            .as_deref()
            .and_then(|be| be.as_any().downcast_ref::<CreakingHeartBlockEntity>())
        else {
            return;
        };
        if !heart.is_protector(self.get_entity().entity_uuid) {
            return;
        }
        if hit_by_player {
            heart.creaking_hurt(world);
        }
        self.play_sound(Sound::EntityCreakingSway);
    }

    fn apply_pending_melee_attack(&self) {
        let Some(pending) = self
            .pending_melee_attack
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
        else {
            return;
        };
        let entity = self.get_entity();
        if entity.is_removed() {
            return;
        }
        let world = entity.world.load_full();
        let Some(target) = world.get_entity_by_id(pending.target_id) else {
            return;
        };
        if target.get_entity().is_removed() {
            return;
        }
        self.mob_entity.living_entity.swing_hand();
        if self.mob_entity.try_attack(entity, target.as_ref()) {
            self.on_attack(target.as_ref());
        }
    }

    pub fn stop_in_place(&self) {
        let entity = &self.mob_entity.living_entity.entity;
        entity.velocity.store(Vector3::default());
        self.mob_entity.steered().navigator().stop();
    }

    pub fn activate(&self, player: &Arc<Player>) {
        self.mob_entity.set_target(Some(player.clone()));
        self.play_sound(Sound::EntityCreakingActivate);
        self.set_is_active(true);
        // Vanilla `Creaking.activate` emits ENTITY_ACTION so nearby sculk sensors react.
        let entity = &self.mob_entity.living_entity.entity;
        entity
            .world
            .load()
            .emit_game_event(GameEvent::EntityAction.name(), entity.pos.load());
    }

    pub fn deactivate(&self) {
        self.mob_entity.set_target(None);
        self.play_sound(Sound::EntityCreakingDeactivate);
        self.set_is_active(false);
        let entity = &self.mob_entity.living_entity.entity;
        entity
            .world
            .load()
            .emit_game_event(GameEvent::EntityAction.name(), entity.pos.load());
    }

    pub fn player_is_stuck_in_you(&self, players: &[Arc<Player>]) -> bool {
        let own_box = self.mob_entity.living_entity.entity.bounding_box.load();
        for player in players {
            let p = player.get_entity().get_eye_pos();
            if p.x >= own_box.min.x
                && p.x <= own_box.max.x
                && p.y >= own_box.min.y
                && p.y <= own_box.max.y
                && p.z >= own_box.min.z
                && p.z <= own_box.max.z
            {
                let count = self.player_stuck_counter.fetch_add(1, Ordering::Relaxed) + 1;
                return count > MAX_PLAYER_STUCK_COUNTER;
            }
        }
        self.player_stuck_counter.store(0, Ordering::Relaxed);
        false
    }

    pub fn is_looking_at_me(&self, player: &Player) -> bool {
        let creaking_entity = &self.mob_entity.living_entity.entity;
        let creaking_pos = creaking_entity.pos.load();
        let eye_y = creaking_entity.get_eye_y();
        let feet_y = creaking_pos.y + 0.5;
        let mid_y = f64::midpoint(eye_y, creaking_pos.y);

        let player_entity = player.get_entity();
        // Vanilla `LivingEntity.isLookingAtMe` never sees across levels.
        if !Arc::ptr_eq(
            &player_entity.world.load_full(),
            &creaking_entity.world.load_full(),
        ) {
            return false;
        }
        let player_eye_pos = player_entity.get_eye_pos();

        let pitch = player_entity.pitch.load().to_radians();
        let yaw = -player_entity.yaw.load().to_radians();

        let cos_pitch = pitch.cos();
        let look_dir = Vector3::new(
            (yaw.sin() * cos_pitch) as f64,
            (-pitch.sin()) as f64,
            (yaw.cos() * cos_pitch) as f64,
        );

        let targets_y = [eye_y, feet_y, mid_y];
        for target_y in targets_y {
            let target = Vector3::new(creaking_pos.x, target_y, creaking_pos.z);
            let dir = target - player_eye_pos;
            let distance = (dir.x * dir.x + dir.y * dir.y + dir.z * dir.z).sqrt();
            if gaze_distance_in_range(distance) {
                let norm_dir = Vector3::new(dir.x / distance, dir.y / distance, dir.z / distance);
                let dot = look_dir.dot(&norm_dir);
                // Vanilla `isLookingAtMe` divides the cone size by distance only when
                // `adjustForDistance` is true; the creaking passes false, so the
                // threshold stays constant. LOS uses the per-gaze-height endpoint
                // (seeThroughTransparentBlocks=true -> ClipBlock::Visual).
                if dot > 1.0 - GAZE_CONE_SIZE {
                    let world = player_entity.world.load_full();
                    let hit = world.clip(&pumpkin_world::clip::ClipContext::new(
                        player_eye_pos,
                        target,
                        pumpkin_world::clip::ClipBlock::Visual,
                        pumpkin_world::clip::ClipFluid::None,
                    ));
                    if hit.is_miss {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn is_player_disguised(player: &Player) -> bool {
        use pumpkin_data::tag::{self, Taggable};

        let equipment = player.living_entity.entity_equipment.try_lock();
        if let Ok(equipment) = equipment
            && let Some(head_stack) = equipment.equipment.get(&EquipmentSlot::HEAD)
            && !head_stack.is_empty()
        {
            return head_stack
                .item
                .has_tag(&tag::Item::MINECRAFT_GAZE_DISGUISE_EQUIPMENT);
        }
        false
    }

    pub fn check_can_move(&self) -> bool {
        let entity = &self.mob_entity.living_entity.entity;
        let world = entity.world.load();
        let pos = entity.pos.load();
        let active = self.is_active();

        let players = world.get_nearby_players(pos, 32.0);
        if players.is_empty() {
            if active {
                self.deactivate();
            }
            return true;
        }

        let mut has_potential_target = false;
        for player in players {
            if player.living_entity.entity.is_alive()
                && player.gamemode.load() != pumpkin_util::GameMode::Creative
                && player.gamemode.load() != pumpkin_util::GameMode::Spectator
            {
                has_potential_target = true;
                let is_disguised = Self::is_player_disguised(&player);
                // Vanilla `checkCanMove`: `(!active || PLAYER_NOT_WEARING_DISGUISE_ITEM.test(player))`.
                // The disguise only shields against an already-active creaking; it does not
                // stop a disguised player from activating a dormant one by looking at it.
                if (!active || !is_disguised) && self.is_looking_at_me(&player) {
                    if active {
                        return false;
                    }
                    let target_pos = player.get_entity().pos.load();
                    let dist_sq = pos.squared_distance_to(target_pos.x, target_pos.y, target_pos.z);
                    if dist_sq < ACTIVATION_RANGE_SQ {
                        self.activate(&player);
                        return false;
                    }
                }
            }
        }

        if !has_potential_target && active {
            self.deactivate();
        }

        true
    }

    pub fn tear_down(&self) {
        let entity = &self.mob_entity.living_entity.entity;
        let world = entity.world.load();
        let pos = entity.pos.load();

        // `block_crumble` carries a block state, so it must use the block particle
        // helper; an empty option payload makes the client fail to decode
        // `level_particles`.
        world.spawn_block_particle(
            pos,
            Vector3::new(0.3, 0.3, 0.3),
            0.0,
            100,
            Block::CREAKING_HEART.default_state.id.as_u16(),
        );

        self.play_sound(Sound::EntityCreakingDeath);
        entity.remove();
    }

    pub fn creaking_death_effects(&self) {
        self.set_tearing_down();
        self.play_sound(Sound::EntityCreakingTwitch);
    }

    fn play_sound(&self, sound: Sound) {
        let entity = &self.mob_entity.living_entity.entity;
        let world = entity.world.load();
        world.play_sound_fine(sound, SoundCategory::Hostile, &entity.pos.load(), 1.0, 1.0);
    }
}

impl Mob for CreakingEntity {
    /// Vanilla `Creaking.getAmbientSound`: silent while active.
    fn get_ambient_sound(&self) -> Option<Sound> {
        if self.is_active() {
            None
        } else {
            Some(Sound::EntityCreakingAmbient)
        }
    }

    fn as_custom_sound(&self) -> Option<&dyn crate::entity::custom_sound::CustomSound> {
        Some(self)
    }

    /// Vanilla `Creaking.doHurtTarget`: a living target starts the attack animation
    /// (entity event 4) before the hit is resolved.
    fn do_hurt_target(&self, target: &dyn EntityBase) -> bool {
        if target.get_living_entity().is_none() {
            return false;
        }
        self.attack_animation_remaining_ticks
            .store(ATTACK_ANIMATION_DURATION, Ordering::Relaxed);
        let entity = self.get_entity();
        {
            let world = entity.world.load();
            world.broadcast_to_chunk(
                entity.chunk_pos.load(),
                &CEntityStatus::new(entity.entity_id, 4),
            );
        };
        // Vanilla lets damage callbacks run outside the brain lock. Reserve the hit
        // here and resolve it at the start of the next tick, after the brain tick ends.
        *self
            .pending_melee_attack
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(PendingMeleeAttack {
            target_id: target.get_entity().entity_id,
        });
        false
    }

    fn on_attack(&self, _target: &dyn EntityBase) {
        self.play_attack_sound();
    }

    /// Vanilla `Creaking.playAttackSound`.
    fn play_attack_sound(&self) {
        self.play_entity_sound(Sound::EntityCreakingAttack, 1.0, 1.0);
    }

    /// Vanilla gates navigation, look and body rotation on `canMove`; a frozen Creaking keeps
    /// its pose and never steers, even while goals such as swimming are still selected.
    fn ticks_navigation(&self) -> bool {
        self.can_move()
    }

    fn ticks_look_control(&self) -> bool {
        self.can_move()
    }

    fn ticks_body_rotation(&self) -> bool {
        self.can_move()
    }

    /// Vanilla `Creaking.canBeLeashed` is false; the mob is bound to its heart instead.
    fn can_be_leashed(&self) -> bool {
        false
    }

    fn before_mob_ai_step(&self) {
        let entity = &self.mob_entity.living_entity.entity;
        if entity
            .world
            .load()
            .get_light_level_dependent_magic_value(&entity.get_eye_pos().to_block_pos())
            > BRIGHTNESS_FOR_FASTER_DESPAWN
        {
            self.mob_entity
                .living_entity
                .no_action_time
                .fetch_add(2, Ordering::Relaxed);
        }
    }

    fn mob_is_pushable(&self) -> bool {
        self.mob_entity.living_entity.is_pushable() && self.can_move()
    }

    /// Vanilla `Creaking.knockback`: a frozen creaking ignores knockback entirely, so report the
    /// push as handled to skip the default knockback applied by the combat/living code.
    fn mob_knockback(
        &self,
        _power: f64,
        _x: f64,
        _z: f64,
        _source: Option<&Entity>,
        _damage: f32,
        _comes_from_effect: bool,
    ) -> bool {
        !self.can_move()
    }

    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {
        self.play_entity_sound(Sound::EntityCreakingStep, 0.15, 1.0);
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        match self.home_binding.load() {
            Some(HeartHome::Current(pos)) => nbt.put("home_pos", block_pos_to_tag(pos)),
            Some(HeartHome::Legacy(pos)) => {
                // A save before the home chunk loads must retain its migration eligibility.
                let mut legacy = NbtCompound::new();
                legacy.put_int("x", pos.0.x);
                legacy.put_int("y", pos.0.y);
                legacy.put_int("z", pos.0.z);
                nbt.put_compound("home_pos", legacy);
            }
            None => {}
        }
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if let Some(pos) = nbt.get("home_pos").and_then(block_pos_from_tag) {
            self.set_heart_home(Some(HeartHome::Current(pos)));
        } else if let Some(legacy) = nbt.get_compound("home_pos")
            && let (Some(x), Some(y), Some(z)) = (
                legacy.get_int("x"),
                legacy.get_int("y"),
                legacy.get_int("z"),
            )
        {
            self.set_heart_home(Some(HeartHome::Legacy(BlockPos::new(x, y, z))));
        }
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn make_brain(&self, packed: &PackedMemories) -> Brain {
        ai::make_brain(self, packed)
    }

    fn custom_server_ai_step(&self, _caller: &dyn EntityBase) {
        ai::tick(self);
    }

    fn after_brain_tick(&self, tick: &mut BrainTick<'_>) {
        ai::after_brain_tick(self, tick);
    }

    fn tick_links(&self) -> Vec<i32> {
        ai::tick_links(self)
    }

    fn apply_mob_target(&self, target: Option<Arc<dyn EntityBase>>) {
        // A damage callback can fire while the brain lock is held during an attack.
        if !self.get_entity().is_removed() {
            self.mob_entity.set_target(target);
        }
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        let entity = &self.mob_entity.living_entity.entity;

        // Vanilla `Creaking#tickDeath`: a heart-bound creaking that is tearing
        // down survives the normal 20-tick corpse removal, twitches for
        // `TWITCH_DEATH_DURATION` ticks and only then crumbles away.
        if self.is_heart_bound() && self.is_tearing_down() {
            let death_time = self.death_time.fetch_add(1, Ordering::Relaxed) + 1;
            if death_time > TWITCH_DEATH_DURATION && !entity.is_removed() {
                self.tear_down();
            }
            return;
        }

        if !entity.is_alive() {
            return;
        }

        // The melee hit reserved by the brain tick resolves here, before the next AI step.
        self.apply_pending_melee_attack();

        if self
            .invulnerability_animation_remaining_ticks
            .load(Ordering::Relaxed)
            > 0
        {
            self.invulnerability_animation_remaining_ticks
                .fetch_sub(1, Ordering::Relaxed);
        }
        if self
            .attack_animation_remaining_ticks
            .load(Ordering::Relaxed)
            > 0
        {
            self.attack_animation_remaining_ticks
                .fetch_sub(1, Ordering::Relaxed);
        }

        let can_move = self.can_move();
        let now_can_move = self.check_can_move();

        if now_can_move != can_move {
            let world = entity.world.load();
            // Vanilla `Creaking.aiStep` emits ENTITY_ACTION on every canMove flip.
            world.emit_game_event(GameEvent::EntityAction.name(), entity.pos.load());
            if now_can_move {
                world.play_sound_fine(
                    Sound::EntityCreakingUnfreeze,
                    SoundCategory::Hostile,
                    &entity.pos.load(),
                    1.0,
                    1.0,
                );
            } else {
                self.stop_in_place();
                world.play_sound_fine(
                    Sound::EntityCreakingFreeze,
                    SoundCategory::Hostile,
                    &entity.pos.load(),
                    1.0,
                    1.0,
                );
            }
            self.set_can_move(now_can_move);
        }

        // Home Creaking Heart check: an unloaded heart chunk is requested, not punished.
        if let Some(home) = self.get_home_pos() {
            let world = entity.world.load_full();
            let BlockEntityLookup::Loaded(block_entity) =
                world.get_block_entity_if_chunk_loaded(&home)
            else {
                self.request_home_chunk(&world, home);
                return;
            };
            let linked = block_entity
                .as_deref()
                .and_then(|be| be.as_any().downcast_ref::<CreakingHeartBlockEntity>())
                .is_some_and(|heart| heart.is_protector(entity.entity_uuid));
            if linked {
                self.migrate_legacy_home(home);
                self.apply_pending_heart_feedback(&world, home);
            } else {
                // Vanilla: losing the heart protection starts the tear-down twitch
                // instead of a regular death animation.
                self.mob_entity.living_entity.health.store(0.0);
                if !self.is_tearing_down() {
                    self.creaking_death_effects();
                }
            }
        }
    }

    fn on_removed(&self) {
        self.pending_home_feedback.store(None);
        self.release_home_ticket();
    }

    fn on_die(&self, _source: Option<&dyn EntityBase>) {
        if self.is_heart_bound() && !self.is_tearing_down() {
            self.creaking_death_effects();
        }
    }

    fn defers_tick_death(&self) -> bool {
        self.is_heart_bound() && self.is_tearing_down()
    }

    fn pre_damage_with_context(
        &self,
        damage_type: DamageType,
        source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
    ) -> bool {
        let entity = &self.mob_entity.living_entity.entity;

        if self.is_heart_bound()
            && damage_type != DamageType::OUT_OF_WORLD
            && !damage_type.has_tag(&tag::DamageType::MINECRAFT_BYPASSES_INVULNERABILITY)
        {
            let has_direct_attacker = source.is_some_and(|entity| {
                entity.get_living_entity().is_some() || entity.get_projectile_owner().is_some()
            });
            let has_responsible_player = source.is_some_and(|entity| entity.get_player().is_some())
                || cause.is_some_and(|entity| entity.get_player().is_some());
            if !heart_shake_allowed(has_direct_attacker, has_responsible_player) {
                // Environmental damage is ignored without starting the shake.
                return false;
            }
            if self
                .invulnerability_animation_remaining_ticks
                .load(Ordering::Relaxed)
                <= 0
                && entity.is_alive()
            {
                self.invulnerability_animation_remaining_ticks
                    .store(8, Ordering::Relaxed);

                let world = entity.world.load_full();
                world.emit_game_event(GameEvent::EntityAction.name(), entity.pos.load());
                world.broadcast_to_chunk(
                    entity.chunk_pos.load(),
                    &CEntityStatus::new(entity.entity_id, 66),
                );

                if let Some(home) = self.get_home_pos() {
                    match world.get_block_entity_if_chunk_loaded(&home) {
                        BlockEntityLookup::Unloaded => {
                            // Vanilla: the hit feedback waits for the home chunk instead
                            // of running inside the damage callback / brain lock.
                            self.pending_home_feedback
                                .store(Some((home, has_responsible_player)));
                            self.request_home_chunk(&world, home);
                        }
                        BlockEntityLookup::Loaded(block_entity) => {
                            let heart = block_entity
                                .as_deref()
                                .and_then(|be| {
                                    be.as_any().downcast_ref::<CreakingHeartBlockEntity>()
                                })
                                .filter(|heart| heart.is_protector(entity.entity_uuid));
                            // Vanilla: only a player-responsible hit counts toward the heart.
                            if let Some(heart) = heart {
                                if has_responsible_player {
                                    heart.creaking_hurt(&world);
                                }
                                self.play_sound(Sound::EntityCreakingSway);
                            }
                        }
                    }
                }
            }
            // Return false so heart-bound Creaking does not lose health from normal damage
            return false;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_data::{Block, entity::EntityType};
    use pumpkin_nbt::compound::NbtCompound;
    use pumpkin_world::{chunk::ChunkData, world::BlockFlags};

    use super::*;
    use crate::{
        block::entities::BlockEntity,
        entity::{
            ai::brain::memory::codec::block_pos_from_tag, ai::test_support::world,
            passive::cow::CowEntity,
        },
    };

    #[test]
    fn gaze_range_excludes_too_close_and_too_far_targets() {
        assert!(gaze_distance_in_range(1.0));
        assert!(gaze_distance_in_range(MAX_GAZE_RANGE));
        assert!(!gaze_distance_in_range(0.0));
        assert!(!gaze_distance_in_range(MAX_GAZE_RANGE + 1.0));
    }

    #[test]
    fn environmental_damage_does_not_start_heart_shake() {
        assert!(!heart_shake_allowed(false, false));
        assert!(heart_shake_allowed(true, false));
        assert!(heart_shake_allowed(false, true));
    }

    #[tokio::test]
    async fn legacy_heart_home_migrates_once_the_heart_chunk_loads()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let creaking = CreakingEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::CREAKING,
        ));
        let home = BlockPos::new(32, 64, 0);

        let mut coords = NbtCompound::new();
        coords.put_int("x", home.0.x);
        coords.put_int("y", home.0.y);
        coords.put_int("z", home.0.z);
        let mut legacy = NbtCompound::new();
        legacy.put_compound("home_pos", coords);
        Mob::mob_read_nbt(creaking.as_ref(), &legacy);
        assert_eq!(creaking.get_home_pos(), Some(home));

        // Before the heart chunk loads, saving keeps the legacy compound shape.
        let mut before = NbtCompound::new();
        Mob::mob_write_nbt(creaking.as_ref(), &mut before);
        assert!(before.get_compound("home_pos").is_some());
        assert!(
            before
                .get("home_pos")
                .and_then(block_pos_from_tag)
                .is_none()
        );

        let chunk_pos = home.chunk_position();
        world
            .level
            .loaded_chunks
            .insert(chunk_pos, ChunkData::empty_sync(chunk_pos.x, chunk_pos.y));
        world.set_block_state(
            &home,
            Block::CREAKING_HEART.default_state.id,
            BlockFlags::NOTIFY_ALL,
        );
        let mut heart_nbt = NbtCompound::new();
        heart_nbt.put_uuid("creaking", creaking.get_entity().entity_uuid);
        world.add_block_entity(Arc::new(CreakingHeartBlockEntity::from_nbt(
            &heart_nbt, home,
        )));

        creaking.mob_tick(creaking.as_ref());
        assert!(creaking.mob_entity.living_entity.is_alive());

        let mut migrated = NbtCompound::new();
        Mob::mob_write_nbt(creaking.as_ref(), &mut migrated);
        assert_eq!(
            migrated.get("home_pos").and_then(block_pos_from_tag),
            Some(home),
        );
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn melee_hit_is_resolved_on_the_next_tick() -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let creaking = CreakingEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.0, 64.0, 0.0),
            &EntityType::CREAKING,
        ));
        let prey = CowEntity::new(Entity::new(
            world.clone(),
            Vector3::new(1.5, 64.0, 0.0),
            &EntityType::COW,
        ));
        world.add_entity_silent(creaking.clone());
        world.add_entity_silent(prey.clone());
        let health = prey.get_mob_entity().living_entity.health.load();

        // The brain tick only reserves the hit; damage callbacks run outside its lock.
        assert!(!Mob::do_hurt_target(creaking.as_ref(), prey.as_ref()));
        assert!(
            creaking
                .pending_melee_attack
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .is_some()
        );
        assert_eq!(prey.get_mob_entity().living_entity.health.load(), health);

        creaking.apply_pending_melee_attack();
        assert!(
            creaking
                .pending_melee_attack
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .is_none()
        );
        assert!(prey.get_mob_entity().living_entity.health.load() < health);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn frozen_creaking_disables_goal_gate() -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let creaking = CreakingEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.0, 64.0, 0.0),
            &EntityType::CREAKING,
        ));
        assert!(creaking_can_move(creaking.as_ref()));
        creaking.set_can_move(false);
        assert!(!creaking_can_move(creaking.as_ref()));
        creaking.set_can_move(true);
        assert!(creaking_can_move(creaking.as_ref()));
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn dormant_creaking_disables_target_gate() -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let creaking = CreakingEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.0, 64.0, 0.0),
            &EntityType::CREAKING,
        ));
        assert!(!creaking_is_active(creaking.as_ref()));
        creaking.set_is_active(true);
        assert!(creaking_is_active(creaking.as_ref()));
        creaking.set_is_active(false);
        assert!(!creaking_is_active(creaking.as_ref()));
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn frozen_creaking_ignores_knockback() -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let creaking = CreakingEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.0, 64.0, 0.0),
            &EntityType::CREAKING,
        ));
        // A mobile creaking lets the default knockback apply (handled == false).
        assert!(!creaking.mob_knockback(0.4, 0.0, 0.0, None, 1.0, false));
        // A frozen creaking reports the push as handled so the default knockback is skipped.
        creaking.set_can_move(false);
        assert!(creaking.mob_knockback(0.4, 0.0, 0.0, None, 1.0, false));
        world.level.shutdown().await;
        Ok(())
    }
}
