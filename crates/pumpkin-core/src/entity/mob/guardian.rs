use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Weak};

use pumpkin_data::attributes::Attributes;
use pumpkin_data::entity::{EntityStatus, EntityType};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_data::{damage::DamageType, sound::Sound};
use pumpkin_protocol::bedrock::server::actor_event::ActorEventID;
use pumpkin_util::{Difficulty, GameMode};

use crate::entity::ai::behavior::hunting::{self, Hunt};
use crate::entity::ai::control::move_control::{MoveControl, Operation};
use crate::entity::ai::control::{Control, LookControlTrait, MoveControlTrait};
use crate::entity::ai::target_match::TargetMatch;
use crate::entity::custom_sound::CustomSound;
use crate::entity::living::LivingEntity;
use crate::entity::{
    Entity, EntityBase,
    ai::goal::{
        Controls, Goal, look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal,
        move_towards_restriction::MoveTowardsRestrictionGoal, random_swimming::RandomSwimmingGoal,
    },
    mob::{Mob, MobEntity},
};
use crate::world::World;

const GUARDIAN_PREY: TargetMatch = TargetMatch::AnyOf(&[
    &EntityType::PLAYER,
    &EntityType::SQUID,
    &EntityType::GLOW_SQUID,
    &EntityType::AXOLOTL,
]);

/// Vanilla `GuardianAttackSelector`: nothing within 3 blocks.
fn outside_melee_range(hunter: &dyn Mob, target: &LivingEntity, _world: &World) -> bool {
    let hunter_pos = hunter.get_mob_entity().living_entity.entity.pos.load();
    target
        .entity
        .pos
        .load()
        .squared_distance_to_vec(&hunter_pos)
        > 9.0
}

pub const GUARDIAN_HUNTS: &[Hunt] =
    &[Hunt::new(1, GUARDIAN_PREY, true).matching(outside_melee_range)];

/// Early out of vanilla `Guardian#hurtServer` before reflecting thorns. `moving` is the
/// synced `isMoving()` flag driven by `GuardianMoveControl`.
fn thorns_reflection_blocked(moving: bool, damage_type: DamageType) -> bool {
    // The avoids-guardian-thorns tag already contains both thorns and magic.
    moving || damage_type.has_tag(&tag::DamageType::MINECRAFT_AVOIDS_GUARDIAN_THORNS)
}

/// Vanilla `Guardian#hurtServer`: reflect 2.0 thorns when the direct entity is living and
/// the damage type is outside `minecraft:avoids_guardian_thorns`.
pub(crate) fn reflect_thorns(
    guardian: &dyn EntityBase,
    damage_type: DamageType,
    cause: Option<&dyn EntityBase>,
    moving: bool,
) {
    if thorns_reflection_blocked(moving, damage_type) {
        return;
    }
    let Some(cause) = cause else { return };
    // Vanilla reflects onto any living direct cause. Pumpkin's damage() does not protect
    // creative players, so keep this guard to preserve vanilla's observable immunity.
    if cause
        .get_player()
        .is_some_and(|player| player.gamemode.load() == GameMode::Creative)
    {
        return;
    }
    if let Some(living) = cause.get_living_entity() {
        let _ = living.damage(guardian, THORNS_DAMAGE, DamageType::THORNS);
    }
}

/// Thorns damage reflected by guardians.
const THORNS_DAMAGE: f32 = 2.0;

/// Vanilla `Guardian.ATTACK_TIME`: beam charge ticks before the beam hurts.
const GUARDIAN_ATTACK_TIME: i32 = 80;
/// Vanilla ElderGuardian.getAttackDuration(): the elder charges a shorter beam.
const ELDER_GUARDIAN_ATTACK_TIME: i32 = 60;

const GUARDIAN_SPEED_LERP: f32 = 0.125;
const GUARDIAN_PUSH_SCALE: f64 = 0.05;
pub(crate) const GUARDIAN_WATER_SPEED: f64 = 0.1;
pub(crate) const GUARDIAN_WATER_DRAG: f64 = 0.9;
pub(crate) const GUARDIAN_IDLE_SINK: f64 = 0.005;

/// Vanilla `GuardianMoveControl#tick` look-at lerp: 0.125 toward the travel direction, snapping
/// immediately when no goal is steering the head (`isLookingAtTarget() == false`).
fn guardian_look_target(
    looking_at_target: bool,
    old: (f64, f64, f64),
    new: (f64, f64, f64),
) -> (f64, f64, f64) {
    let (ox, oy, oz) = if looking_at_target { old } else { new };
    const LERP: f64 = 0.125;
    (
        ox + LERP * (new.0 - ox),
        oy + LERP * (new.1 - oy),
        oz + LERP * (new.2 - oz),
    )
}

/// Vanilla `GuardianAttackGoal.tick` damage: 1.0 base, +2 on Hard, +2 for an elder guardian.
const fn beam_magic_damage(hard: bool, elder: bool) -> f32 {
    let mut damage = 1.0;
    if hard {
        damage += 2.0;
    }
    if elder {
        damage += 2.0;
    }
    damage
}

/// Vanilla `Guardian.GuardianAttackGoal`: charge a beam for 80 ticks, then hit with indirect
/// magic and a melee swing. The client renders the beam from `DATA_ID_ATTACK_TARGET`.
pub struct GuardianAttackGoal {
    attack_time: i32,
    elder: bool,
    attack_target: TrackedData,
}

impl GuardianAttackGoal {
    #[must_use]
    pub const fn new(elder: bool, attack_target: TrackedData) -> Self {
        Self {
            attack_time: -10,
            elder,
            attack_target,
        }
    }

    /// Vanilla Guardian/ElderGuardian.getAttackDuration().
    const fn attack_duration(&self) -> i32 {
        if self.elder {
            ELDER_GUARDIAN_ATTACK_TIME
        } else {
            GUARDIAN_ATTACK_TIME
        }
    }
}

impl Goal for GuardianAttackGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        mob.get_mob_entity()
            .get_target()
            .is_some_and(|target| target.get_entity().is_alive())
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        let Some(target) = mob.get_mob_entity().get_target() else {
            return false;
        };
        if !target.get_entity().is_alive() {
            return false;
        }
        // Vanilla `canContinueToUse`: an elder keeps its target at any range, a guardian only
        // while the target stays outside melee range (9 squared).
        if self.elder {
            return true;
        }
        let hunter_pos = mob.get_entity().pos.load();
        target
            .get_entity()
            .pos
            .load()
            .squared_distance_to_vec(&hunter_pos)
            > 9.0
    }

    fn start(&mut self, _mob: &dyn Mob) {
        self.attack_time = -10;
    }

    fn stop(&mut self, mob: &dyn Mob) {
        mob.get_entity().set_synced_data(self.attack_target, 0i32);
        mob.get_mob_entity().set_target(None);
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(target) = mob.get_mob_entity().get_target() else {
            return;
        };

        {
            let steered = mob.get_mob_entity().steered();
            steered.navigator().stop();
        }
        mob.get_mob_entity()
            .look_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .look_at_entity_with_range(&target, 90.0, 90.0);

        if !mob.has_line_of_sight(target.get_entity()) {
            mob.get_mob_entity().set_target(None);
            return;
        }

        self.attack_time += 1;
        if self.attack_time == 0 {
            let entity = mob.get_entity();
            entity.set_synced_data(self.attack_target, target.get_entity().entity_id);
            if !entity.is_silent() {
                let world = entity.world.load();
                world.broadcast_entity_event(
                    entity,
                    EntityStatus::GuardianAttackSound,
                    ActorEventID::GuardianAttackSound,
                );
            }
        } else if self.attack_time >= self.attack_duration() {
            let entity = mob.get_entity();
            let world = entity.world.load();
            let hard = world.level_info.load().difficulty == Difficulty::Hard;
            let damage = beam_magic_damage(hard, self.elder);
            let _ = target.damage_with_context(
                entity,
                damage,
                DamageType::INDIRECT_MAGIC,
                None,
                Some(entity),
                Some(entity),
            );
            mob.do_hurt_target(target.as_ref());
            mob.get_mob_entity().set_target(None);
        }
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}

/// Vanilla `Guardian.GuardianMoveControl`: swims toward the wanted position with a wave push,
/// smoothly steers the head toward the travel direction, and marks the guardian as moving, which
/// gates thorns reflection.
pub struct GuardianMoveControl {
    inner: MoveControl,
    moving: Arc<AtomicBool>,
}

impl GuardianMoveControl {
    pub fn new(moving: Arc<AtomicBool>) -> Self {
        Self {
            inner: MoveControl::default(),
            moving,
        }
    }
}

impl Control for GuardianMoveControl {}

impl MoveControlTrait for GuardianMoveControl {
    fn tick(&mut self, mob: &dyn Mob) {
        let mob_entity = mob.get_mob_entity();
        let living = &mob_entity.living_entity;
        let entity = &living.entity;
        if self.inner.operation != Operation::MoveTo || mob_entity.steered().navigator().is_done() {
            mob_entity.set_speed(0.0);
            self.moving.store(false, Ordering::Relaxed);
            entity.set_synced_data(pumpkin_data::tracked_data::guardian::DATA_ID_MOVING, false);
            return;
        }
        let pos = entity.pos.load();
        let xd = self.inner.wanted_x - pos.x;
        let yd = self.inner.wanted_y - pos.y;
        let zd = self.inner.wanted_z - pos.z;
        let length = (xd * xd + yd * yd + zd * zd).sqrt().max(f64::EPSILON);
        let (xd, yd, zd) = (xd / length, yd / length, zd / length);

        let yaw = MoveControl::rotlerp(
            entity.yaw.load(),
            zd.atan2(xd).to_degrees() as f32 - 90.0,
            MoveControl::MAX_TURN,
        );
        entity.yaw.store(yaw);
        entity.body_yaw.store(yaw);

        let target_speed = (self.inner.speed_modifier
            * living.get_attribute_value(&Attributes::MOVEMENT_SPEED))
            as f32;
        let speed = living.get_speed() + GUARDIAN_SPEED_LERP * (target_speed - living.get_speed());
        mob_entity.set_speed(speed);

        let time =
            f64::from(entity.tick_count.load(Ordering::Relaxed)) + f64::from(entity.entity_id);
        let push = (time * 0.5).sin() * GUARDIAN_PUSH_SCALE;
        let y_push = (time * 0.75).sin() * GUARDIAN_PUSH_SCALE;
        let yaw_rad = f64::from(yaw).to_radians();
        let (sin, cos) = (yaw_rad.sin(), yaw_rad.cos());
        entity.velocity.store(entity.velocity.load().add_raw(
            push * cos,
            y_push * (sin + cos) * 0.25 + f64::from(speed) * yd * 0.1,
            push * sin,
        ));
        self.moving.store(true, Ordering::Relaxed);
        entity.set_synced_data(pumpkin_data::tracked_data::guardian::DATA_ID_MOVING, true);

        // Vanilla `GuardianMoveControl#tick`: lerp the look target toward the travel direction.
        let new_look = (
            pos.x + xd * 2.0,
            entity.get_eye_y() + yd / length,
            pos.z + zd * 2.0,
        );
        let mut look_control = mob_entity
            .look_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let old_look = look_control
            .wanted_position()
            .map_or(new_look, |p| (p.x, p.y, p.z));
        let (look_x, look_y, look_z) =
            guardian_look_target(look_control.is_looking_at_target(), old_look, new_look);
        look_control.look_at_with_range(look_x, look_y, look_z, 10.0, 40.0);
    }
}

pub struct GuardianEntity {
    pub mob_entity: MobEntity,
    moving: Arc<AtomicBool>,
}

impl GuardianEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        // Vanilla `Guardian.defineSynchedData`: both values start at the client's default.
        entity
            .synched_data
            .define(pumpkin_data::tracked_data::guardian::DATA_ID_MOVING, false);
        entity.synched_data.define(
            pumpkin_data::tracked_data::guardian::DATA_ID_ATTACK_TARGET,
            0i32,
        );
        let moving = Arc::new(AtomicBool::new(false));
        let mut mob_entity = MobEntity::new(entity);
        *mob_entity
            .move_control
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            Box::new(GuardianMoveControl::new(moving.clone()));
        let guardian = Self { mob_entity, moving };
        let mob_arc = Arc::new(guardian);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;

            // Vanilla `Guardian.registerGoals`: the beam attack outranks movement.
            goal_selector.add_goal(
                3,
                Box::new(GuardianAttackGoal::new(
                    false,
                    pumpkin_data::tracked_data::guardian::DATA_ID_ATTACK_TARGET,
                )),
            );
            goal_selector.add_goal(4, Box::new(RandomSwimmingGoal::new(1.0, 10)));
            // Vanilla `Guardian.registerGoals`: restriction at 5 keeps the guardian near home.
            goal_selector.add_goal(5, Box::new(MoveTowardsRestrictionGoal::new(1.0)));
            goal_selector.add_goal(
                5,
                LookAtEntityGoal::with_default(mob_weak.clone(), &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(6, Box::new(RandomLookAroundGoal::default()));

            let target_selector = &mob_arc.mob_entity.target_selector;
            hunting::apply(target_selector, &mob_arc.mob_entity, GUARDIAN_HUNTS);
        };

        mob_arc
    }
}

impl CustomSound for GuardianEntity {
    fn hurt_sound(&self) -> Option<Sound> {
        let entity = self.get_entity();
        let is_water = entity.touching_water.load(Ordering::Relaxed);
        Some(if is_water {
            Sound::EntityGuardianHurt
        } else {
            Sound::EntityGuardianHurtLand
        })
    }

    fn death_sound(&self) -> Option<Sound> {
        let entity = self.get_entity();
        let is_water = entity.touching_water.load(Ordering::Relaxed);
        Some(if is_water {
            Sound::EntityGuardianDeath
        } else {
            Sound::EntityGuardianDeathLand
        })
    }
}

impl Mob for GuardianEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn get_ambient_sound(&self) -> Option<Sound> {
        Some(
            if self.get_entity().touching_water.load(Ordering::Relaxed) {
                Sound::EntityGuardianAmbient
            } else {
                Sound::EntityGuardianAmbientLand
            },
        )
    }

    fn on_damage_with_context(
        &self,
        damage_type: DamageType,
        source: Option<&dyn EntityBase>,
        _cause: Option<&dyn EntityBase>,
    ) -> bool {
        // Vanilla reflects only when the direct entity (`source`) is living, so an arrow shot by
        // a player does not reflect thorns onto that player.
        reflect_thorns(
            self.get_entity(),
            damage_type,
            source,
            self.moving.load(Ordering::Relaxed),
        );
        true
    }

    fn get_ambient_sound_interval(&self) -> i32 {
        160
    }

    // Vanilla `Guardian#travelInWater` has no gravity or ground friction, a uniform 0.9 drag,
    // and a slight sink while idle with no target.
    fn travel_in_water(&self, caller: &dyn EntityBase, _gravity: f64, _falling: bool) {
        let living = &self.mob_entity.living_entity;
        let entity = &living.entity;
        entity.update_velocity_from_input(living.movement_input.load(), GUARDIAN_WATER_SPEED);
        living.make_move(caller);
        let mut velocity = entity.velocity.load() * GUARDIAN_WATER_DRAG;
        if !self.moving.load(Ordering::Relaxed) && self.mob_entity.get_target().is_none() {
            velocity.y -= GUARDIAN_IDLE_SINK;
        }
        entity.velocity.store(velocity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn beam_damage_matches_vanilla_difficulty_and_elder_bonus() {
        assert_eq!(beam_magic_damage(false, false), 1.0);
        assert_eq!(beam_magic_damage(true, false), 3.0);
        assert_eq!(beam_magic_damage(false, true), 3.0);
        assert_eq!(beam_magic_damage(true, true), 5.0);
    }

    #[test]
    fn beam_charge_duration_matches_vanilla_elder_override() {
        use pumpkin_data::tracked_data::guardian::DATA_ID_ATTACK_TARGET;
        assert_eq!(
            GuardianAttackGoal::new(false, DATA_ID_ATTACK_TARGET).attack_duration(),
            80
        );
        assert_eq!(
            GuardianAttackGoal::new(true, DATA_ID_ATTACK_TARGET).attack_duration(),
            60
        );
    }

    #[test]
    fn thorns_reflection_blocked_matches_vanilla_tag() {
        // The avoids-guardian-thorns tag already contains thorns and magic.
        assert!(thorns_reflection_blocked(false, DamageType::THORNS));
        assert!(thorns_reflection_blocked(false, DamageType::MAGIC));
        assert!(thorns_reflection_blocked(true, DamageType::GENERIC));
        assert!(!thorns_reflection_blocked(false, DamageType::GENERIC));
        assert!(!thorns_reflection_blocked(false, DamageType::ARROW));
    }

    #[test]
    fn guardian_look_target_snaps_when_not_looking() {
        let old = (0.0, 0.0, 0.0);
        let new = (10.0, 5.0, -3.0);
        assert_eq!(guardian_look_target(false, old, new), new);
    }

    #[test]
    fn guardian_look_target_lerps_toward_new_when_looking() {
        let old = (0.0, 0.0, 0.0);
        let new = (10.0, 5.0, -3.0);
        let (x, y, z) = guardian_look_target(true, old, new);
        assert!((x - 1.25).abs() < 1.0e-9);
        assert!((y - 0.625).abs() < 1.0e-9);
        assert!((z + 0.375).abs() < 1.0e-9);
    }
}
