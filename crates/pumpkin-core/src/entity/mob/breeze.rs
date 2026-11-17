use crate::entity::EntityBase;
use pumpkin_data::damage::DamageType;
use crate::entity::projectile_deflection::{ProjectileDeflection, default_deflection};
use pumpkin_data::sound::Sound;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity,
    ai::goal::{
        active_target::ActiveTargetGoal, look_at_entity::LookAtEntityGoal,
        look_around::RandomLookAroundGoal, revenge::RevengeGoal, swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

pub struct BreezeEntity {
    pub mob_entity: MobEntity,
}

impl BreezeEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let breeze = Self { mob_entity };
        let mob_arc = Arc::new(breeze);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak.clone(), &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(7, Box::new(RandomLookAroundGoal::default()));

            let target_selector = &mob_arc.mob_entity.target_selector;
            target_selector.add_goal(
                1,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
            );
            // Vanilla `Breeze.registerGoals`: `new HurtByTargetGoal(this)`, no alerting.
            target_selector.add_goal(2, Box::new(RevengeGoal::new(true)));
        };

        mob_arc
    }
}

impl Mob for BreezeEntity {
    /// Vanilla `Breeze.isInvulnerableTo`: damage from another breeze is ignored.
    fn pre_damage_with_context(
        &self,
        _damage_type: DamageType,
        _source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
    ) -> bool {
        !cause.is_some_and(|entity| entity.get_entity().entity_type == &EntityType::BREEZE)
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
        Some(if self
            .mob_entity
            .living_entity
            .entity
            .on_ground
            .load(Ordering::Relaxed)
        {
            Sound::EntityBreezeIdleGround
        } else {
            Sound::EntityBreezeIdleAir
        })
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
