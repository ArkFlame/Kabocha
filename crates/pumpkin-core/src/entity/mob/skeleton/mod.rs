use pumpkin_data::tag::Taggable;
use std::sync::{Arc, Weak};

use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_util::Difficulty;

use crate::entity::{
    Entity,
    ai::behavior::fleeing::{self, Flee},
    ai::behavior::hunting::{self, Hunt},
    ai::goal::{
        bow_attack::BowAttackGoal, flee_sun::FleeSunGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, melee_attack::MeleeAttackGoal,
        restrict_sun::RestrictSunGoal, revenge::RevengeGoal, swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    ai::target_match::TargetMatch,
    mob::{Mob, MobEntity},
};

pub mod bogged;
pub mod parched;
#[allow(clippy::module_inception)]
pub mod skeleton;
pub mod stray;
pub mod wither;

/// Vanilla `AbstractSkeleton` bow attack interval, 20 on hard.
const SKELETON_ATTACK_INTERVAL: i32 = 40;
const SKELETON_HARD_ATTACK_INTERVAL: i32 = 20;

const SKELETON_FLEES: &[Flee] = &[Flee::new(
    3,
    TargetMatch::Type(&EntityType::WOLF),
    6.0,
    1.0,
    1.2,
)];

const SKELETON_HUNTS: &[Hunt] = &[
    Hunt::new(2, TargetMatch::Type(&EntityType::PLAYER), true),
    Hunt::new(3, TargetMatch::Type(&EntityType::IRON_GOLEM), true),
    hunting::baby_turtles(3, true),
];

pub struct SkeletonEntityBase {
    pub mob_entity: MobEntity,
}

impl SkeletonEntityBase {
    pub fn new(entity: Entity) -> Arc<Self> {
        Self::with_hunts(entity, &[])
    }

    /// `extra` goes first, like vanilla subclasses adding targets before `super`.
    pub fn with_hunts(entity: Entity, extra: &[Hunt]) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let mob = Self { mob_entity };
        let mob_arc = Arc::new(mob);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };
        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;
            let target_selector = &mob_arc.mob_entity.target_selector;

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(2, Box::new(RestrictSunGoal::new()));
            goal_selector.add_goal(3, Box::new(FleeSunGoal::new(1.0)));
            fleeing::apply(goal_selector, SKELETON_FLEES);
            // Vanilla `reassessWeaponGoal`: the bow or the melee goal at 4, by the held weapon.
            goal_selector.add_goal(
                4,
                Box::new(BowAttackGoal::new(1.0, SKELETON_ATTACK_INTERVAL, 15.0)),
            );
            goal_selector.add_goal(
                4,
                Box::new(
                    MeleeAttackGoal::new(1.2, false)
                        .gated_by(|mob| !BowAttackGoal::is_holding_bow(mob)),
                ),
            );
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::water_avoiding(1.0)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(6, Box::new(RandomLookAroundGoal::default()));

            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));
            hunting::apply(target_selector, &mob_arc.mob_entity, extra);
            hunting::apply(target_selector, &mob_arc.mob_entity, SKELETON_HUNTS);
        };

        mob_arc
    }

    /// Vanilla `AbstractSkeleton#populateDefaultEquipmentSlots`: a skeleton always spawns with a
    /// bow in its main hand. Spawns that skip the registry roll still need it, so this is
    /// idempotent and syncs the slot to nearby clients.
    pub fn equip_default_bow(&self) {
        let living = &self.mob_entity.living_entity;
        let has_bow = living
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&EquipmentSlot::MAIN_HAND)
            .item
            .id
            == Item::BOW.id;
        if !has_bow {
            self.mob_entity
                .set_item_slot(&EquipmentSlot::MAIN_HAND, ItemStack::new(1, &Item::BOW));
        }
    }
}

pub fn wants_to_pick_up<M: Mob + ?Sized>(mob: &M, stack: &ItemStack) -> bool {
    !stack
        .item
        .has_tag(&pumpkin_data::tag::Item::MINECRAFT_SPEARS)
        && mob.can_hold_item(stack)
}

impl Mob for SkeletonEntityBase {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn get_min_attack_interval(&self) -> Option<i32> {
        let world = self.mob_entity.living_entity.entity.world.load();
        Some(if world.level_info.load().difficulty == Difficulty::Hard {
            SKELETON_HARD_ATTACK_INTERVAL
        } else {
            SKELETON_ATTACK_INTERVAL
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attack_interval_matches_vanilla() {
        assert_eq!(SKELETON_ATTACK_INTERVAL, 40);
    }

    #[test]
    fn hard_attack_interval_matches_vanilla() {
        assert_eq!(SKELETON_HARD_ATTACK_INTERVAL, 20);
    }
}
