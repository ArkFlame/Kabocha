use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, AtomicI32, Ordering},
};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::damage::DamageType;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::entity::EntityType;
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
        goal::{
            active_target::ActiveTargetGoal, look_around::RandomLookAroundGoal,
            look_at_entity::LookAtEntityGoal, melee_attack::MeleeAttackGoal, swim::SwimGoal,
            wander_around::WanderAroundGoal,
        },
        pathfinder::{node::PathType, walk_node_evaluator::HomeRestriction},
    },
    mob::{Mob, MobEntity},
    player::Player,
};

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

        let mob_arc = Arc::new(creaking);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            // Vanilla CreakingAi gates melee and stroll on canMove and idles at 0.3 speed.
            goal_selector.add_goal(
                4,
                Box::new(
                    MeleeAttackGoal::new(1.0, false)
                        .with_attack_interval(ATTACK_INTERVAL)
                        .gated_by(creaking_can_move),
                ),
            );
            goal_selector.add_goal(
                5,
                Box::new(
                    WanderAroundGoal::new(f64::from(SPEED_MULTIPLIER_WHEN_IDLING))
                        .with_start_condition(creaking_can_move),
                ),
            );
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak.clone(), &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(7, Box::new(RandomLookAroundGoal::default()));

            let target_selector = &mob_arc.mob_entity.target_selector;
            target_selector.add_goal(
                1,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true)
                    .gated_by(creaking_is_active),
            );
        };

        mob_arc
    }

    pub fn set_transient(&self, pos: BlockPos) {
        self.set_home_pos(Some(pos));
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
            if distance > 0.001 {
                let norm_dir = Vector3::new(dir.x / distance, dir.y / distance, dir.z / distance);
                let dot = look_dir.dot(&norm_dir);
                // Vanilla `isLookingAtMe` divides the cone size by distance only when
                // `adjustForDistance` is true; the creaking passes false, so the
                // threshold stays constant.
                if dot > 1.0 - GAZE_CONE_SIZE && player_entity.has_line_of_sight(creaking_entity) {
                    return true;
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
        let damaged = self.get_mob_entity().try_attack(entity, target);
        if damaged {
            self.play_attack_sound();
        }
        damaged
    }

    /// Vanilla `Creaking.playAttackSound`.
    fn play_attack_sound(&self) {
        self.play_entity_sound(Sound::EntityCreakingAttack, 1.0, 1.0);
    }

    fn mob_is_pushable(&self) -> bool {
        self.mob_entity.living_entity.is_pushable() && self.can_move()
    }

    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {
        self.play_entity_sound(Sound::EntityCreakingStep, 0.15, 1.0);
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        if let Some(pos) = self.get_home_pos() {
            let mut sub = NbtCompound::new();
            sub.put_int("x", pos.0.x);
            sub.put_int("y", pos.0.y);
            sub.put_int("z", pos.0.z);
            nbt.put_compound("home_pos", sub);
        }
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if let Some(sub) = nbt.get_compound("home_pos")
            && let (Some(x), Some(y), Some(z)) =
                (sub.get_int("x"), sub.get_int("y"), sub.get_int("z"))
        {
            let pos = BlockPos::new(x, y, z);
            self.set_transient(pos);
        }
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
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

        // Home Creaking Heart check
        if let Some(home_pos) = self.get_home_pos() {
            let world = entity.world.load();
            let has_protection = world.get_block_entity(&home_pos).is_some_and(|be| {
                be.as_any()
                    .downcast_ref::<CreakingHeartBlockEntity>()
                    .is_some_and(|heart_be| heart_be.is_protector(entity.entity_uuid))
            });

            if !has_protection {
                // Vanilla: losing the heart protection starts the tear-down twitch
                // instead of a regular death animation.
                self.mob_entity.living_entity.health.store(0.0);
                if !self.is_tearing_down() {
                    self.creaking_death_effects();
                }
            }
        }
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

                let world = entity.world.load();
                world.emit_game_event(GameEvent::EntityAction.name(), entity.pos.load());
                world.broadcast_to_chunk(
                    entity.chunk_pos.load(),
                    &CEntityStatus::new(entity.entity_id, 66),
                );

                if let Some(home_pos) = self.get_home_pos()
                    && let Some(be) = world.get_block_entity(&home_pos)
                    && let Some(heart_be) = be.as_any().downcast_ref::<CreakingHeartBlockEntity>()
                    && heart_be.is_protector(entity.entity_uuid)
                {
                    // Vanilla: only a player-responsible hit counts toward the heart.
                    if has_responsible_player {
                        heart_be.creaking_hurt(&world);
                    }
                    self.play_sound(Sound::EntityCreakingSway);
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
    use super::*;
    use crate::entity::ai::test_support::world;

    #[test]
    fn environmental_damage_does_not_start_heart_shake() {
        assert!(!heart_shake_allowed(false, false));
        assert!(heart_shake_allowed(true, false));
        assert!(heart_shake_allowed(false, true));
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
}
