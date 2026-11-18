use super::{Controls, Goal};

use crate::entity::ai::goal::revenge::MobFilter;
use crate::entity::mob::Mob;
use crate::world::World;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::world::WorldEvent;
use pumpkin_data::{Block, BlockStateId, Property};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::BlockFlags;
use rand::RngExt;
use std::sync::Arc;

/// Vanilla `BeeGrowCropGoal.tick` grows on a `random.nextInt(30) == 0` roll.
const GROW_CHANCE_ONE_IN: i32 = 30;
/// Crops checked below the bee, nearest first.
const GROW_DEPTH: i32 = 2;

/// Vanilla `BeeGrowCropGoal.canBeeUse` rejects 30% of eligible checks.
#[must_use]
pub fn passes_grow_gate(roll: f32) -> bool {
    roll >= 0.3
}

/// Vanilla `BeeGrowCropGoal.tick` only grows on a zero roll.
#[must_use]
pub const fn should_grow_this_tick(roll: i32) -> bool {
    roll == 0
}

/// Vanilla checks `belowState.is(BlockTags.BEE_GROWABLES)`. Pitcher crop is excluded because
/// its upper half has to be kept in sync with the lower age, which needs block-specific handling.
fn is_growable_block(block: &Block) -> bool {
    block.has_tag(&tag::Block::MINECRAFT_BEE_GROWABLES) && block != &Block::PITCHER_CROP
}

/// Positions `1..=GROW_DEPTH` blocks below `origin` that `is_growable` accepts, nearest first.
#[must_use]
pub fn growable_positions(
    origin: BlockPos,
    is_growable: impl Fn(&BlockPos) -> bool,
) -> Vec<BlockPos> {
    (1..=GROW_DEPTH)
        .map(|depth| origin.offset(Vector3::new(0, -depth, 0)))
        .filter(|pos| is_growable(pos))
        .collect()
}

/// Vanilla grows a crop by exactly one age (`CropBlock.getStateForAge(getAge + 1)`), also used
/// for stems and sweet berry bush. Bone meal instead jumps several stages, so bees write the
/// next age state directly.
#[must_use]
pub fn grown_state_id(block: &Block, state_id: BlockStateId) -> Option<BlockStateId> {
    // Cave vines have no age property: vanilla grows them through `performBonemeal` instead.
    if block == &Block::CAVE_VINES || block == &Block::CAVE_VINES_PLANT {
        return None;
    }
    let max_age = u8::try_from(block.states.len()).ok()?.checked_sub(1)?;
    let mut props = block.properties(state_id)?.to_props();
    let age = props.iter_mut().find_map(|prop| match prop {
        Property::Age(age) => Some(age),
        _ => None,
    })?;
    if *age >= max_age {
        return None;
    }
    *age += 1;
    Some(block.from_properties(&props).to_state_id(block))
}

/// Vanilla bee crop growth: occasionally fertilizes the crop under the bee, counting each success.
pub struct GrowCropGoal {
    gate: MobFilter,
    on_crop_grown: fn(&dyn Mob),
}

impl GrowCropGoal {
    /// `gate` says whether the bee may grow crops. `on_crop_grown` runs once per fertilized crop.
    #[must_use]
    pub const fn new(gate: MobFilter, on_crop_grown: fn(&dyn Mob)) -> Self {
        Self {
            gate,
            on_crop_grown,
        }
    }

    fn is_growable_at(world: &Arc<World>, pos: &BlockPos) -> bool {
        is_growable_block(world.get_block(pos))
    }
}

impl Goal for GrowCropGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if !(self.gate)(mob) || !passes_grow_gate(rand::random::<f32>()) {
            return false;
        }
        let world = mob.get_entity().world.load();
        let origin = mob.get_entity().block_pos.load();
        !growable_positions(origin, |pos| Self::is_growable_at(&world, pos)).is_empty()
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let roll = rand::rng().random_range(0..self.get_tick_count(GROW_CHANCE_ONE_IN));
        if !should_grow_this_tick(roll) {
            return;
        }
        let world = mob.get_entity().world.load();
        let origin = mob.get_entity().block_pos.load();
        for pos in growable_positions(origin, |pos| Self::is_growable_at(&world, pos)) {
            let (block, state) = world.get_block_and_state(&pos);
            let grown = grown_state_id(block, state.id);
            // Mature crops and cave vines fall through to `bone_meal`, which rejects the
            // former and applies the vanilla bonemeal growth to the latter.
            if grown.is_none()
                && !world
                    .block_registry
                    .bone_meal(block, &world, &pos, state.id)
            {
                continue;
            }
            if let Some(grown) = grown {
                world.set_block_state(&pos, grown, BlockFlags::NOTIFY_LISTENERS);
            }
            world.sync_world_event(WorldEvent::ParticlesBeeGrowth, pos, 15);
            (self.on_crop_grown)(mob);
        }
    }

    fn controls(&self) -> Controls {
        Controls::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn growable_positions_checks_two_blocks_below_nearest_first() {
        let origin = BlockPos::new(0, 64, 0);
        let crops = [BlockPos::new(0, 63, 0), BlockPos::new(0, 62, 0)];
        assert_eq!(
            growable_positions(origin, |pos| crops.contains(pos)),
            vec![BlockPos::new(0, 63, 0), BlockPos::new(0, 62, 0)]
        );
    }

    #[test]
    fn grow_gate_mirrors_vanilla_thirty_percent_rejection() {
        assert!(passes_grow_gate(0.0));
        assert!(passes_grow_gate(0.299_999));
        assert!(!passes_grow_gate(0.3));
        assert!(!passes_grow_gate(1.0));
    }

    #[test]
    fn growth_roll_only_succeeds_on_zero() {
        assert!(should_grow_this_tick(0));
        assert!(!should_grow_this_tick(1));
        assert!(!should_grow_this_tick(29));
    }

    #[test]
    fn growable_positions_ignores_blocks_outside_the_depth() {
        let origin = BlockPos::new(0, 64, 0);
        let far = [BlockPos::new(0, 61, 0), BlockPos::new(0, 64, 0)];
        assert!(growable_positions(origin, |pos| far.contains(pos)).is_empty());
    }

    #[test]
    fn grown_state_id_increments_age_by_one() {
        let wheat = &Block::WHEAT;
        assert!(grown_state_id(wheat, wheat.default_state.id).is_some());
        let Some(next) = grown_state_id(wheat, wheat.default_state.id) else {
            return;
        };
        let age = wheat.properties(next).and_then(|props| {
            props.to_props().into_iter().find_map(|prop| match prop {
                Property::Age(age) => Some(age),
                _ => None,
            })
        });
        assert_eq!(age, Some(1));
    }

    #[test]
    fn grown_state_id_stops_at_max_age() {
        let wheat = &Block::WHEAT;
        let mature = wheat
            .from_properties(&[Property::Age(7)])
            .to_state_id(wheat);
        assert!(grown_state_id(wheat, mature).is_none());
        let bush = &Block::SWEET_BERRY_BUSH;
        let mature = bush.from_properties(&[Property::Age(3)]).to_state_id(bush);
        assert!(grown_state_id(bush, mature).is_none());
    }

    #[test]
    fn grown_state_id_ignores_blocks_without_age() {
        let dirt = &Block::DIRT;
        assert!(grown_state_id(dirt, dirt.default_state.id).is_none());
    }
}
