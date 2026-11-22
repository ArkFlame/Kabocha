mod ai;
mod movement;

use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Mutex, Weak};

use arc_swap::ArcSwapOption;
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::{EntityPose, EntityType};
use pumpkin_data::sound::Sound;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::random::{get_seed, legacy_rand::LegacyRand};

use crate::entity::EntityBase;
use crate::entity::ai::brain::memory::types;
use crate::entity::ai::brain::{Brain, BrainTick, memory::PackedMemories};
use crate::entity::ai::pathfinder::node::PathType;
use crate::entity::ai::target_match::TargetMatch;
use crate::entity::projectile_deflection::{ProjectileDeflection, default_deflection};
use crate::entity::{
    Entity,
    ai::goal::{
        active_target::ActiveTargetGoal, breeze_shoot::BreezeShootGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal, revenge::RevengeGoal,
        swim::SwimGoal,
    },
    mob::{Mob, MobEntity},
};
use crate::plugin::api::events::entity::entity_pose_change::EntityPoseChangeEvent;

/// Vanilla `Breeze` `NearestAttackableSensor` filter: `canAttack` accepts players and iron golems.
const BREEZE_TARGET_TYPES: TargetMatch =
    TargetMatch::AnyOf(&[&EntityType::PLAYER, &EntityType::IRON_GOLEM]);

/// Vanilla `Breeze.WHIRL_SOUND_FREQUENCY_MIN` / `_MAX`: ticks between idle whirls.
const WHIRL_SOUND_FREQUENCY_MIN: i32 = 1;
const WHIRL_SOUND_FREQUENCY_MAX: i32 = 80;

/// Vanilla `Breeze.tick` whirl countdown. A zero countdown draws a new interval and
/// does not play; otherwise the countdown steps down and the whirl plays when it hits zero.
/// `roll` is only called when a new interval is needed, as vanilla consumes randomness.
fn advance_whirl_countdown(ticks: i32, roll: impl FnOnce() -> i32) -> (i32, bool) {
    let next = if ticks == 0 { roll() } else { ticks - 1 };
    (next, next == 0)
}

pub(super) fn look_from_eyes(entity: &Entity, target: Vector3<f64>) {
    entity.look_at(target.add_raw(0.0, -entity.get_eye_height(), 0.0));
    let yaw = entity.yaw.load();
    entity.head_yaw.store(yaw);
    entity.body_yaw.store(yaw);
}

struct PoseCleanup {
    previous: EntityPose,
    target: Option<Weak<dyn EntityBase>>,
}

pub struct BreezeEntity {
    pub mob_entity: MobEntity,
    whirl_countdown: AtomicI32,
    pose_cleanup: ArcSwapOption<PoseCleanup>,
    movement_random: Mutex<LegacyRand>,
}

impl BreezeEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let breeze = Self {
            mob_entity,
            whirl_countdown: AtomicI32::new(0),
            pose_cleanup: ArcSwapOption::empty(),
            movement_random: Mutex::new(LegacyRand::from_seed(get_seed())),
        };
        let mob_arc = Arc::new(breeze);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            // Vanilla Breeze ranged attack: inhale, fire a wind charge, recover.
            goal_selector.add_goal(4, Box::new(BreezeShootGoal::new(Arc::downgrade(&mob_arc))));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak.clone(), &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(7, Box::new(RandomLookAroundGoal::default()));

            let target_selector = &mob_arc.mob_entity.target_selector;
            target_selector.add_goal(
                1,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, BREEZE_TARGET_TYPES, true),
            );
            // Vanilla `Breeze.registerGoals`: `new HurtByTargetGoal(this)`, no alerting.
            target_selector.add_goal(2, Box::new(RevengeGoal::new(true)));
        };
        // Vanilla `Breeze` ctor: trapdoor tops (`ON_TOP_OF_TRAPDOOR`, Pumpkin's
        // `DangerTrapdoor`) and fire are walkable at -1 malus.
        let mut navigator = mob_arc
            .mob_entity
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        navigator.set_pathfinding_malus(PathType::DangerTrapdoor, -1.0);
        navigator.set_pathfinding_malus(PathType::DamageFire, -1.0);
        drop(navigator);

        mob_arc
    }

    fn with_movement_random<T>(&self, sample: impl FnOnce(&mut LegacyRand) -> T) -> T {
        let mut random = self
            .movement_random
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        sample(&mut random)
    }

    /// Queues the return to `Standing`, keeping the cleanup alive if a plugin vetoes it.
    fn finish_pose(
        tick: &mut BrainTick<'_>,
        previous: EntityPose,
        target: Option<Weak<dyn EntityBase>>,
    ) {
        tick.defer_pose_change(EntityPose::Standing, move |mob| {
            let Some(breeze) = mob.cast_any().downcast_ref::<Self>() else {
                return;
            };
            let pose = breeze.get_entity().pose.load();
            if pose != previous && pose != EntityPose::Standing {
                return;
            }
            let cleanup = Arc::new(PoseCleanup { previous, target });
            breeze.pose_cleanup.store(Some(cleanup.clone()));
            breeze.restore_standing(&cleanup, true);
        });
    }

    fn clear_pose_cleanup(&self, cleanup: &Arc<PoseCleanup>) -> bool {
        let active = self.pose_cleanup.load_full();
        if active
            .as_ref()
            .is_some_and(|active| Arc::ptr_eq(active, cleanup))
        {
            self.pose_cleanup.store(None);
            true
        } else {
            false
        }
    }

    fn restore_standing(&self, cleanup: &Arc<PoseCleanup>, allow_standing: bool) {
        let entity = self.get_entity();
        let living = &self.mob_entity.living_entity;
        let previous = entity.pose.load();
        if !living.is_alive()
            || !(previous == cleanup.previous || allow_standing && previous == EntityPose::Standing)
        {
            self.clear_pose_cleanup(cleanup);
            return;
        }
        let restored = entity.set_pose_if(
            EntityPose::Standing,
            || {
                living.is_alive()
                    && entity.pose.load() == previous
                    && self
                        .pose_cleanup
                        .load()
                        .as_ref()
                        .is_some_and(|active| Arc::ptr_eq(active, cleanup))
            },
            || {},
        );
        if restored
            && self.clear_pose_cleanup(cleanup)
            && living.is_alive()
            && entity.pose.load() == EntityPose::Standing
            && let Some(target) = cleanup.target.as_ref().and_then(Weak::upgrade)
            && self
                .mob_entity
                .stored_target()
                .is_some_and(|current| Arc::ptr_eq(&current, &target))
        {
            self.mob_entity.with_brain(self, |brain| {
                if brain
                    .get(types::ATTACK_TARGET)
                    .is_some_and(|current| Arc::ptr_eq(current, &target))
                {
                    brain.set_with_expiry(types::BREEZE_SHOOT, (), 60);
                }
            });
        }
    }
}

impl Mob for BreezeEntity {
    fn make_brain(&self, packed: &PackedMemories) -> Brain {
        ai::make_brain(self, packed)
    }

    fn custom_server_ai_step(&self, _caller: &dyn EntityBase) {
        // A veto leaves the old pose in place. Retry before the next Brain tick.
        if let Some(cleanup) = self.pose_cleanup.load_full() {
            self.restore_standing(&cleanup, false);
            if !self.mob_entity.living_entity.is_alive() {
                return;
            }
        }

        // Plugins can set the mob's target between Brain ticks.
        let current = self.mob_entity.get_target();
        let target_id = |target: &Arc<dyn EntityBase>| target.get_entity().entity_id;
        self.mob_entity.with_brain(self, |brain| {
            if brain.get(types::ATTACK_TARGET).map(target_id) != current.as_ref().map(target_id) {
                brain.set_optional(types::ATTACK_TARGET, current.clone());
            }
        });
        self.mob_entity.tick_brain(self);
        self.mob_entity
            .with_brain(self, |brain| ai::update_activity(brain));
    }

    /// Vanilla Breeze long jumps add extra vertical speed scaled by follow range.
    fn additional_tick_movement(&self) -> f64 {
        movement::max_jump_speed(&self.mob_entity.living_entity)
    }

    /// Landing blocks and overhead sight rays can lie beyond the normal follow range.
    fn conservative_block_read_radius(&self) -> f64 {
        movement::block_read_radius(&self.mob_entity.living_entity)
    }

    fn has_global_tick_reach(&self) -> bool {
        let Some(server) = self.get_entity().world.load().server.upgrade() else {
            return false;
        };
        server
            .plugin_manager
            .has_handlers::<EntityPoseChangeEvent>()
    }

    /// Vanilla `Breeze.isInvulnerableTo`: damage from another breeze is ignored.
    fn pre_damage_with_context(
        &self,
        _damage_type: DamageType,
        source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
    ) -> bool {
        !cause
            .or(source)
            .is_some_and(|entity| entity.get_entity().entity_type == &EntityType::BREEZE)
    }

    fn cause_fall_damage(
        &self,
        caller: &dyn EntityBase,
        fall_distance: f32,
        damage_modifier: f32,
    ) -> bool {
        if fall_distance > 3.0 {
            caller.play_entity_sound(Sound::EntityBreezeLand, 1.0, 1.0);
        }
        self.mob_entity
            .living_entity
            .cause_fall_damage(caller, fall_distance, damage_modifier)
    }

    /// Vanilla `Breeze.getAmbientSound`: ground and air idles differ.
    fn get_ambient_sound(&self) -> Option<Sound> {
        Some(
            if self
                .mob_entity
                .living_entity
                .entity
                .on_ground
                .load(Ordering::Relaxed)
            {
                Sound::EntityBreezeIdleGround
            } else {
                Sound::EntityBreezeIdleAir
            },
        )
    }

    /// Vanilla `Breeze.canAttack`: only players and iron golems are valid targets.
    fn can_attack(&self, target: &dyn EntityBase) -> bool {
        let target_type = target.get_entity().entity_type;
        (target_type == &EntityType::PLAYER || target_type == &EntityType::IRON_GOLEM)
            && self.can_attack_base(target)
    }

    /// Vanilla Breeze.getMaxHeadYRot: the head only turns 30 degrees off the body.
    fn get_max_head_rotation(&self) -> f32 {
        30.0
    }

    /// Vanilla `Breeze.getHeadRotSpeed`: the head turns toward a target at 25 degrees per tick.
    fn get_max_look_yaw_change(&self) -> f32 {
        25.0
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    /// Vanilla `Breeze.playAmbientSound`: idles only while it has no target or is airborne.
    fn play_ambient_sound(&self) {
        if self.mob_entity.get_target().is_some()
            && self
                .mob_entity
                .living_entity
                .entity
                .on_ground
                .load(Ordering::Relaxed)
        {
            return;
        }
        if let Some(sound) = self.get_ambient_sound() {
            self.make_sound(sound);
        }
    }

    /// Vanilla `Breeze.tick`: idle whirl on a random 1..=80 tick countdown.
    fn mob_tick(&self, _caller: &dyn EntityBase) {
        let (next, play) =
            advance_whirl_countdown(self.whirl_countdown.load(Ordering::Relaxed), || {
                rand::random_range(WHIRL_SOUND_FREQUENCY_MIN..=WHIRL_SOUND_FREQUENCY_MAX)
            });
        self.whirl_countdown.store(next, Ordering::Relaxed);
        if play {
            let pitch = 0.7 + 0.4 * rand::random::<f32>();
            let volume = 0.8 + 0.2 * rand::random::<f32>();
            self.play_entity_sound(Sound::EntityBreezeWhirl, volume, pitch);
        }
    }

    fn deflection(&self, projectile: &dyn EntityBase) -> ProjectileDeflection {
        let projectile_type = projectile.get_entity().entity_type;
        if projectile_type == &EntityType::WIND_CHARGE
            || projectile_type == &EntityType::BREEZE_WIND_CHARGE
        {
            ProjectileDeflection::None
        } else if default_deflection(&self.mob_entity.living_entity.entity)
            == ProjectileDeflection::Reverse
        {
            ProjectileDeflection::Breeze
        } else {
            ProjectileDeflection::None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::ai::test_support::world;
    use std::cell::Cell;

    #[test]
    fn whirl_plays_once_per_drawn_interval() {
        let rolls = Cell::new(0);
        let mut ticks = 0;
        let mut plays = Vec::new();
        for _ in 0..8 {
            let (next, play) = advance_whirl_countdown(ticks, || {
                rolls.set(rolls.get() + 1);
                3
            });
            ticks = next;
            plays.push(play);
        }
        assert_eq!(
            plays,
            [false, false, false, true, false, false, false, true]
        );
        // Randomness is drawn only when a new interval starts (ticks 1 and 5 of 8).
        assert_eq!(rolls.get(), 2);
    }

    #[test]
    fn breeze_targets_players_and_iron_golems_only() {
        assert!(BREEZE_TARGET_TYPES.matches(&EntityType::PLAYER));
        assert!(BREEZE_TARGET_TYPES.matches(&EntityType::IRON_GOLEM));
        assert!(!BREEZE_TARGET_TYPES.matches(&EntityType::ZOMBIE));
    }

    #[tokio::test]
    async fn head_rot_speed_matches_vanilla_breeze() -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let breeze = BreezeEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::BREEZE,
        ));
        assert_eq!(breeze.get_max_look_yaw_change(), 25.0);
        world.level.shutdown().await;
        Ok(())
    }
}
