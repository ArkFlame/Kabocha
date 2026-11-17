use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Weak};

use pumpkin_data::attributes::Attributes;
use pumpkin_data::entity::EntityType;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::{damage::DamageType, sound::Sound};
use pumpkin_util::GameMode;

use crate::entity::ai::behavior::hunting::{self, Hunt};
use crate::entity::ai::control::move_control::{MoveControl, Operation};
use crate::entity::ai::control::{Control, MoveControlTrait};
use crate::entity::ai::target_match::TargetMatch;
use crate::entity::custom_sound::CustomSound;
use crate::entity::living::LivingEntity;
use crate::entity::{
    Entity, EntityBase,
    ai::goal::{
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal,
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

const GUARDIAN_SPEED_LERP: f32 = 0.125;
const GUARDIAN_PUSH_SCALE: f64 = 0.05;
pub(crate) const GUARDIAN_WATER_SPEED: f64 = 0.1;
pub(crate) const GUARDIAN_WATER_DRAG: f64 = 0.9;
pub(crate) const GUARDIAN_IDLE_SINK: f64 = 0.005;

/// Vanilla `Guardian.GuardianMoveControl`: swims toward the wanted position with a wave push
/// and marks the guardian as moving, which gates thorns reflection. The look-at update is not
/// ported yet.
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
    }
}

pub struct GuardianEntity {
    pub mob_entity: MobEntity,
    moving: Arc<AtomicBool>,
}

impl GuardianEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
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
    fn thorns_reflection_blocked_matches_vanilla_tag() {
        // The avoids-guardian-thorns tag already contains thorns and magic.
        assert!(thorns_reflection_blocked(false, DamageType::THORNS));
        assert!(thorns_reflection_blocked(false, DamageType::MAGIC));
        assert!(thorns_reflection_blocked(true, DamageType::GENERIC));
        assert!(!thorns_reflection_blocked(false, DamageType::GENERIC));
        assert!(!thorns_reflection_blocked(false, DamageType::ARROW));
    }
}
