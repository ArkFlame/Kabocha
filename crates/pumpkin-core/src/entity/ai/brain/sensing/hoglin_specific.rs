use std::sync::Arc;

use pumpkin_data::entity::EntityType;

use crate::entity::living::LivingEntity;

use super::super::BrainTick;
use super::super::memory::{MemoryModuleId, types};
use super::Sensor;

const REQUIRES: [MemoryModuleId; 5] = [
    types::NEAREST_VISIBLE_LIVING_ENTITIES.id(),
    types::NEAREST_VISIBLE_ADULT_PIGLIN.id(),
    types::NEAREST_VISIBLE_ADULT_HOGLINS.id(),
    types::VISIBLE_ADULT_PIGLIN_COUNT.id(),
    types::VISIBLE_ADULT_HOGLIN_COUNT.id(),
];

/// Vanilla hoglin sensor: records the nearest visible adult piglin and every visible adult
/// hoglin so the brain can decide when to retreat from an outnumbering crowd.
pub struct HoglinSpecificSensor;

impl Sensor for HoglinSpecificSensor {
    fn requires(&self) -> &'static [MemoryModuleId] {
        &REQUIRES
    }

    fn do_tick(&mut self, tick: &mut BrainTick) {
        let mut adult_piglin: Option<Arc<dyn crate::entity::EntityBase>> = None;
        let mut adult_hoglins: Vec<Arc<dyn crate::entity::EntityBase>> = Vec::new();
        let mut piglin_count = 0_i32;
        let mut hoglin_count = 0_i32;

        if let Some(visible) = tick.state.get(types::NEAREST_VISIBLE_LIVING_ENTITIES) {
            for entity in visible.find_all(&tick.visibility(), |_| true) {
                // Vanilla filters out babies and non-piglin/hoglin before the loop body.
                if entity
                    .get_living_entity()
                    .is_some_and(LivingEntity::is_baby)
                {
                    continue;
                }
                let entity_type = entity.get_entity().entity_type;
                if entity_type == &EntityType::PIGLIN {
                    piglin_count += 1;
                    if adult_piglin.is_none() {
                        adult_piglin = Some(Arc::clone(entity));
                    }
                } else if entity_type == &EntityType::HOGLIN {
                    hoglin_count += 1;
                    adult_hoglins.push(Arc::clone(entity));
                }
            }
        }

        tick.state
            .set_optional(types::NEAREST_VISIBLE_ADULT_PIGLIN, adult_piglin);
        tick.state
            .set(types::NEAREST_VISIBLE_ADULT_HOGLINS, adult_hoglins);
        tick.state
            .set(types::VISIBLE_ADULT_PIGLIN_COUNT, piglin_count);
        tick.state
            .set(types::VISIBLE_ADULT_HOGLIN_COUNT, hoglin_count);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering;

    use pumpkin_data::entity::EntityType;
    use pumpkin_util::math::vector3::Vector3;

    use super::*;
    use crate::entity::Entity;
    use crate::entity::ai::brain::BrainState;
    use crate::entity::ai::brain::sensing::Sensor;
    use crate::entity::ai::brain::sensing::nearest_living_entities::NearestLivingEntitySensor;
    use crate::entity::ai::test_support::world;
    use crate::entity::mob::hoglin::HoglinEntity;
    use crate::entity::mob::piglin::PiglinEntity;
    use crate::world::World;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn piglin(world: &Arc<World>, x: f64) -> Arc<PiglinEntity> {
        PiglinEntity::new(Entity::new(
            world.clone(),
            Vector3::new(x, 64.0, 0.0),
            &EntityType::PIGLIN,
        ))
    }

    fn hoglin(world: &Arc<World>, x: f64) -> Arc<HoglinEntity> {
        HoglinEntity::new(Entity::new(
            world.clone(),
            Vector3::new(x, 64.0, 0.0),
            &EntityType::HOGLIN,
        ))
    }

    #[tokio::test]
    async fn hoglin_sensor_counts_only_visible_adults() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let body = hoglin(&world, 0.0);
        let piglin_a = piglin(&world, 2.0);
        let piglin_b = piglin(&world, 3.0);
        let baby_piglin = piglin(&world, 4.0);
        baby_piglin
            .mob_entity
            .living_entity
            .entity
            .age
            .store(-1, Ordering::Relaxed);
        let hoglin_adult = hoglin(&world, 5.0);
        let baby_hoglin = hoglin(&world, 6.0);
        baby_hoglin
            .mob_entity
            .living_entity
            .entity
            .age
            .store(-1, Ordering::Relaxed);

        world.entities.store(Arc::new(vec![
            body.clone(),
            piglin_a.clone(),
            piglin_b,
            baby_piglin,
            hoglin_adult.clone(),
            baby_hoglin,
        ]));

        let mut state = BrainState::default();
        let mut near = NearestLivingEntitySensor;
        for id in near.requires() {
            state.register_memory(*id);
        }
        let mut specific = HoglinSpecificSensor;
        for id in specific.requires() {
            state.register_memory(*id);
        }
        near.do_tick(&mut BrainTick {
            state: &mut state,
            world: &world,
            mob: &body,
            time: 0,
        });
        specific.do_tick(&mut BrainTick {
            state: &mut state,
            world: &world,
            mob: &body,
            time: 0,
        });
        world.level.shutdown().await;

        assert_eq!(state.get(types::VISIBLE_ADULT_PIGLIN_COUNT), Some(&2));
        assert_eq!(state.get(types::VISIBLE_ADULT_HOGLIN_COUNT), Some(&1));
        assert_eq!(
            state
                .get(types::NEAREST_VISIBLE_ADULT_PIGLIN)
                .map(|e| e.get_entity().entity_id),
            Some(piglin_a.get_entity().entity_id)
        );
        assert_eq!(
            state
                .get(types::NEAREST_VISIBLE_ADULT_HOGLINS)
                .map(|e| e.len()),
            Some(1)
        );
        Ok(())
    }
}
