use std::sync::Arc;

use pumpkin_data::environment_attribute::Activity;

use super::CreakingEntity;
use crate::entity::{
    EntityBase,
    ai::brain::{
        Brain, BrainProvider, BrainTick,
        activity::ActivityData,
        behavior::{
            DoNothing, LookAtTargetSink, MoveToTargetSink, Swim, Timed, melee_attack,
            random_stroll, run_one, set_entity_look_target_sometimes,
            set_walk_target_from_attack_target_if_target_out_of_reach,
            set_walk_target_from_look_target, start_attacking, stop_attacking_if_target_invalid,
        },
        memory::{MemoryStatus, PackedMemories, types},
        sensing::SensorType,
    },
    mob::Mob,
};

const ATTACK_INTERVAL: i32 = 40;
const SPEED_MULTIPLIER_WHEN_IDLING: f32 = 0.3;

pub(super) fn make_brain(creaking: &CreakingEntity, packed: &PackedMemories) -> Brain {
    BrainProvider {
        memory_types: &[],
        sensor_types: &[
            SensorType::NearestLivingEntities,
            SensorType::NearestPlayers,
        ],
        activities,
    }
    .make_brain(creaking, packed)
}

fn activities(_mob: &dyn Mob) -> Vec<ActivityData> {
    vec![
        ActivityData::with_priority_start(
            Activity::Core,
            0,
            vec![
                Box::new(Timed::new(Swim::with_condition(0.8, can_move))),
                Box::new(Timed::new(LookAtTargetSink::new(45, 90))),
                Box::new(Timed::new(MoveToTargetSink::default())),
            ],
        ),
        ActivityData::with_priority_start(
            Activity::Idle,
            10,
            vec![
                Box::new(start_attacking::create(
                    |tick| is_active(tick.mob),
                    |tick| {
                        tick.state
                            .get(types::NEAREST_VISIBLE_ATTACKABLE_PLAYER)
                            .cloned()
                            .map(|player| player as Arc<dyn EntityBase>)
                    },
                )),
                Box::new(set_entity_look_target_sometimes::create(8.0, 30, 60)),
                Box::new(run_one(vec![
                    (
                        Box::new(random_stroll::stroll(SPEED_MULTIPLIER_WHEN_IDLING)),
                        2,
                    ),
                    (
                        Box::new(set_walk_target_from_look_target::create(
                            SPEED_MULTIPLIER_WHEN_IDLING,
                            3,
                        )),
                        2,
                    ),
                    (Box::new(DoNothing::new(30, 60)), 1),
                ])),
            ],
        ),
        ActivityData::with_priority_start_and_conditions(
            Activity::Fight,
            10,
            vec![
                Box::new(
                    set_walk_target_from_attack_target_if_target_out_of_reach::create(|_| 1.0),
                ),
                Box::new(melee_attack::create_with(can_move, ATTACK_INTERVAL)),
                Box::new(stop_attacking_if_target_invalid::create_with_filter(
                    is_target_unreachable,
                )),
            ],
            vec![(types::ATTACK_TARGET.id(), MemoryStatus::ValuePresent)],
        ),
    ]
}

/// A frozen Creaking never swims or melees.
fn can_move(mob: &dyn Mob) -> bool {
    super::creaking_can_move(mob)
}

/// Only an active Creaking acquires a player target on its own.
fn is_active(mob: &dyn Mob) -> bool {
    super::creaking_is_active(mob)
}

fn is_target_unreachable(tick: &BrainTick<'_>, target: &Arc<dyn EntityBase>) -> bool {
    !tick
        .state
        .get(types::NEAREST_VISIBLE_ATTACKABLE_PLAYERS)
        .is_some_and(|players| {
            players
                .iter()
                .any(|player| player.entity_id() == target.get_entity().entity_id)
        })
}

pub(super) fn tick(creaking: &CreakingEntity) {
    creaking.mob_entity.tick_brain(creaking);
}

pub(super) fn after_brain_tick(creaking: &CreakingEntity, tick: &mut BrainTick<'_>) {
    if creaking.get_entity().is_removed() {
        return;
    }
    if creaking.can_move() {
        tick.state
            .set_active_activity_to_first_valid(&[Activity::Fight, Activity::Idle]);
    } else {
        tick.state.use_default_activity();
    }
}

fn entity_id(entity: &dyn EntityBase) -> i32 {
    entity.get_entity().entity_id
}

pub(super) fn tick_links(creaking: &CreakingEntity) -> Vec<i32> {
    let mut links = Vec::new();
    if let Some(stored) = creaking.mob_entity.get_target() {
        links.push(entity_id(stored.as_ref()));
    }
    creaking.mob_entity.with_brain(creaking, |brain| {
        if let Some(players) = brain.get(types::NEAREST_PLAYERS) {
            links.extend(players.iter().map(|player| player.entity_id()));
        }
        let look_target = brain
            .get(types::LOOK_TARGET)
            .and_then(|look| look.tracked_entity());
        let walk_target = brain
            .get(types::WALK_TARGET)
            .and_then(|walk| walk.target().tracked_entity());
        for target in [brain.get(types::ATTACK_TARGET), look_target, walk_target]
            .into_iter()
            .flatten()
        {
            links.push(target.get_entity().entity_id);
        }
    });
    links
}

#[cfg(test)]
mod tests {
    use pumpkin_data::entity::EntityType;
    use pumpkin_util::math::vector3::Vector3;

    use super::*;
    use crate::entity::{
        Entity, ai::brain::activity::ActivityData, ai::test_support::world, passive::cow::CowEntity,
    };

    #[tokio::test]
    async fn dormant_creaking_ignores_targets_that_an_active_creaking_takes()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let creaking = CreakingEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.0, 64.0, 0.0),
            &EntityType::CREAKING,
        ));
        let prey = CowEntity::new(Entity::new(
            world.clone(),
            Vector3::new(2.0, 64.0, 0.0),
            &EntityType::COW,
        ));
        let mut brain = Brain::default();
        brain.add_activity(ActivityData::with_priority_start(
            Activity::Core,
            0,
            vec![Box::new(start_attacking::create(
                |tick| is_active(tick.mob),
                {
                    let prey = prey.clone();
                    move |_| Some(prey.clone() as Arc<dyn EntityBase>)
                },
            ))],
        ));

        brain.tick(&world, creaking.as_ref(), 1);
        let dormant = brain.get(types::ATTACK_TARGET).is_some();
        creaking.set_is_active(true);
        brain.tick(&world, creaking.as_ref(), 2);
        let active = brain
            .get(types::ATTACK_TARGET)
            .map(|target| target.get_entity().entity_id);
        world.level.shutdown().await;

        assert!(!dormant);
        assert_eq!(active, Some(prey.get_entity().entity_id));
        Ok(())
    }
}
