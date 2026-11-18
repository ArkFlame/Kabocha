use crate::generation::proto_chunk::{GenerationCache, StructureEntity};
use bitflags::bitflags;
use pumpkin_data::fluid::{Fluid, FluidState};
use pumpkin_data::{Block, BlockState, BlockStateId, Mirror, Property, Rotation, chunk::Biome};
use pumpkin_util::math::position::BlockPos;
use thiserror::Error;

bitflags! {
    /// Flags used to control the side effects of a block state change.
    /// These match the internal bitmask used by Minecraft's `setBlockState` method
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct BlockFlags: u32 {
        /// Causes a neighbor update to be sent to surrounding blocks.
        /// This is what makes observers detect changes or redstone components react
        const NOTIFY_NEIGHBORS                      = 0b000_0000_0001;
        /// Notifies listeners (like clients) that the block has changed.
        /// On the server, this triggers a packet to be sent to players in range
        const NOTIFY_LISTENERS                      = 0b000_0000_0010;
        /// Combines neighbor notification and listener notification.
        /// This is the "standard" update used for most block placements
        const NOTIFY_ALL                            = 0b000_0000_0011;
        /// Forces the block state to be set even if it matches the current state
        /// Used by items like the Debug Stick to bypass "virtual" change checks
        const FORCE_STATE                           = 0b000_0000_0100;
        /// Prevents the previous block from dropping items when it is replaced
        /// Commonly used when a block is "transformed" rather than destroyed
        const SKIP_DROPS                            = 0b000_0000_1000;
        /// Signals that the block is being moved (usually by a piston)
        /// This prevents certain "on-break" logic from firing until the move is complete
        const MOVED                                 = 0b000_0001_0000;
        /// Prevents redstone wire from re-calculating its shape/power immediately
        /// Used during massive redstone updates to reduce calculation lag
        const SKIP_REDSTONE_WIRE_STATE_REPLACEMENT  = 0b000_0010_0000;
        /// If set, the `on_replaced` callback for block entities (containers) is skipped
        /// Useful if you are moving a Block Entity and don't want it to drop its contents yet
        const SKIP_BLOCK_ENTITY_REPLACED_CALLBACK   = 0b000_0100_0000;
        /// Prevents the `on_added` logic from firing for the new block state
        /// Use this to avoid recursive placement loops or unnecessary initialization
        const SKIP_BLOCK_ADDED_CALLBACK             = 0b000_1000_0000;
        /// Vanilla `Block.UPDATE_KNOWN_SHAPE`: skips the neighbour shape updates for this
        /// change. Used when a multi-block structure such as a nether portal is written in
        /// one go, so the blocks are not removed again while the structure is incomplete.
        const UPDATE_KNOWN_SHAPE                    = 0b001_0000_0000;
    }
}

#[derive(Debug, Error)]
pub enum GetBlockError {
    InvalidBlockId,
    BlockOutOfWorldBounds,
}

impl std::fmt::Display for GetBlockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

pub trait WorldPortalExt: Send + Sync {
    fn can_place_at(
        &self,
        block: &Block,
        state: &BlockState,
        block_accessor: &dyn BlockAccessor,
        block_pos: &BlockPos,
    ) -> bool;

    fn mirror(&self, block: &Block, state_id: BlockStateId, mirror: Mirror) -> &'static BlockState;

    fn rotate(
        &self,
        block: &Block,
        state_id: BlockStateId,
        rotation: Rotation,
    ) -> &'static BlockState;

    fn spawn_mobs_for_chunk_generation(
        &self,
        cache: &mut dyn GenerationCache,
        biome: &'static Biome,
        chunk_x: i32,
        chunk_z: i32,
    );

    fn spawn_structure_entities(&self, _entities: Vec<(StructureEntity, &'static Biome)>) {}
}

pub trait BlockAccessor: Send + Sync {
    fn get_block(&self, position: &BlockPos) -> &'static Block;

    fn get_block_state(&self, position: &BlockPos) -> &'static BlockState;

    fn get_block_state_id(&self, position: &BlockPos) -> BlockStateId;

    fn get_block_and_state(&self, position: &BlockPos) -> (&'static Block, &'static BlockState);

    /// Clips a ray segment against the block and fluid shapes in this accessor.
    fn clip(&self, context: &crate::clip::ClipContext) -> crate::clip::BlockHitResult {
        crate::clip::clip(self, context)
    }
}

/// Returns the normalized fluid family contained in a block state.
#[must_use]
pub fn fluid_from_block_state(id: BlockStateId) -> &'static Fluid {
    if let Some(fluid) = Fluid::from_state_id(id) {
        return fluid.to_flowing();
    }
    // These blocks contain source water without a `waterlogged` property.
    if matches!(
        id.to_block_id(),
        pumpkin_data::BlockId::KELP
            | pumpkin_data::BlockId::KELP_PLANT
            | pumpkin_data::BlockId::SEAGRASS
            | pumpkin_data::BlockId::TALL_SEAGRASS
            | pumpkin_data::BlockId::BUBBLE_COLUMN
    ) || id.is_waterlogged()
    {
        &Fluid::FLOWING_WATER
    } else {
        &Fluid::EMPTY
    }
}

// Vanilla caches the fluid type's amount; the source variant is the family member
// whose fluid states report isSource == true. Find it generically so water, lava and
// any future registered fluid are handled by the same code path.
fn source_variant(fluid: &'static Fluid) -> &'static Fluid {
    (0u16..)
        .map_while(Fluid::from_id)
        .find(|candidate| {
            candidate.matches_type(fluid) && candidate.states.iter().any(|state| state.is_source)
        })
        .unwrap_or(&Fluid::EMPTY)
}

// Vanilla LiquidBlock#getFluidState reads the LEVEL property of any liquid block and
// clamps it to 8. Reading the property generically keeps getAmount() correct for every
// fluid that exposes a level, not just water and lava.
fn liquid_level(id: BlockStateId) -> Option<(u8, BlockStateId)> {
    let block = id.to_block();
    let values = block.properties(id)?.to_props();
    let level = values.iter().find_map(|prop| match prop {
        Property::Level(value) => Some(*value),
        _ => None,
    })?;
    let legacy = block
        .states
        .get(usize::from(level.min(8)))
        .map_or(id, |state| state.id);
    Some((level, legacy))
}

/// Returns the fluid family and its source, flowing, or falling state.
#[must_use]
pub fn fluid_state_from_block_state(id: BlockStateId) -> (&'static Fluid, FluidState) {
    let fluid = fluid_from_block_state(id);
    let source = source_variant(fluid);
    let mut state = source.states[source.default_state_index as usize].clone();

    if let Some((level, block_state_id)) = liquid_level(id) {
        // LiquidBlock#getFluidState: source, amounts 7..1, then falling amount 8.
        let amount = if level == 0 || level >= 8 {
            8
        } else {
            8 - level
        };
        state.height = f32::from(amount) / 9.0;
        state.level = i16::from(amount);
        state.is_source = level == 0;
        state.is_still = state.is_source;
        state.falling = level >= 8;
        state.block_state_id = block_state_id;
    }

    // Keep the normalized family used by fluid callbacks, independently of source state.
    (fluid, state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_data::block_properties::WaterLikeProperties;

    #[test]
    fn liquid_level_uses_any_liquid_blocks_level_property() {
        for block in [&Block::WATER, &Block::LAVA] {
            for level in [0u8, 1, 4, 8, 15] {
                let id = WaterLikeProperties { level }.to_state_id(block);
                let result = liquid_level(id);
                assert!(result.is_some());
                if let Some((read_level, legacy)) = result {
                    assert_eq!(read_level, level);
                    assert_eq!(
                        legacy,
                        WaterLikeProperties {
                            level: level.min(8)
                        }
                        .to_state_id(block)
                    );
                }
            }
        }
        assert!(liquid_level(Block::STONE.default_state.id).is_none());
    }

    #[test]
    fn source_variant_finds_source_for_any_fluid_family() {
        assert_eq!(source_variant(&Fluid::FLOWING_WATER).id, Fluid::WATER.id);
        assert_eq!(source_variant(&Fluid::FLOWING_LAVA).id, Fluid::LAVA.id);
        assert_eq!(source_variant(&Fluid::EMPTY).id, Fluid::EMPTY.id);
    }

    #[test]
    fn fluid_amount_is_generic_across_liquid_blocks() {
        for (block, flowing) in [
            (&Block::WATER, &Fluid::FLOWING_WATER),
            (&Block::LAVA, &Fluid::FLOWING_LAVA),
        ] {
            for (level, amount) in [(0u8, 8i16), (1, 7), (4, 4), (7, 1), (8, 8), (15, 8)] {
                let id = WaterLikeProperties { level }.to_state_id(block);
                let (fluid, state) = fluid_state_from_block_state(id);
                assert_eq!(fluid.id, flowing.id);
                assert_eq!(state.level, amount);
                assert_eq!(state.is_source, level == 0);
                assert_eq!(state.falling, level >= 8);
            }
        }
    }
}
