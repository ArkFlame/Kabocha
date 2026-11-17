use std::sync::atomic::Ordering;
use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::{damage::DamageType, sound::Sound};

use crate::entity::ai::behavior::hunting::{self, Hunt};
use crate::entity::ai::target_match::TargetMatch;
use crate::entity::custom_sound::CustomSound;
use crate::entity::living::LivingEntity;
use crate::entity::{
    Entity, EntityBase,
    ai::goal::{
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal,
        random_swimming::RandomSwimmingGoal,
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

/// Vanilla `Guardian#hurt`: reflect 2.0 thorns when the direct entity is a living
/// non-guardian and the damage type is outside `minecraft:avoids_guardian_thorns`.
pub(crate) fn reflect_thorns(
    guardian: &dyn EntityBase,
    damage_type: DamageType,
    cause: Option<&dyn EntityBase>,
) {
    if damage_type.has_tag(&tag::DamageType::MINECRAFT_AVOIDS_GUARDIAN_THORNS) {
        return;
    }
    let Some(cause) = cause else { return };
    let cause_type = cause.get_entity().entity_type;
    if cause_type == &EntityType::GUARDIAN || cause_type == &EntityType::ELDER_GUARDIAN {
        return;
    }
    if let Some(living) = cause.get_living_entity() {
        let _ = living.damage(guardian, 2.0, DamageType::THORNS);
    }
}

pub struct GuardianEntity {
    pub mob_entity: MobEntity,
}

impl GuardianEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let guardian = Self { mob_entity };
        let mob_arc = Arc::new(guardian);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;

            goal_selector.add_goal(4, Box::new(RandomSwimmingGoal::new(1.0, 10)));
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
        Some(if self.get_entity().touching_water.load(Ordering::Relaxed) {
            Sound::EntityGuardianAmbient
        } else {
            Sound::EntityGuardianAmbientLand
        })
    }

    fn on_damage_with_context(
        &self,
        damage_type: DamageType,
        _source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
    ) -> bool {
        reflect_thorns(self.get_entity(), damage_type, cause);
        true
    }

    fn get_ambient_sound_interval(&self) -> i32 {
        160
    }
}
