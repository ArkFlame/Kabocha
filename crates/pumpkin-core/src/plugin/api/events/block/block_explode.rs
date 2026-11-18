use pumpkin_data::Block;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

use crate::world::World;

use super::BlockEvent;

/// An event that occurs when a block explodes.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockExplodeEvent {
    pub block_pos: BlockPos,
    /// Blocks that will be destroyed; plugins may remove entries to protect them.
    pub blocks: Vec<BlockPos>,
    pub yield_rate: f32,
    pub world: Arc<World>,
}

impl BlockExplodeEvent {
    #[must_use]
    pub const fn new(
        block_pos: BlockPos,
        blocks: Vec<BlockPos>,
        yield_rate: f32,
        world: Arc<World>,
    ) -> Self {
        Self {
            block_pos,
            blocks,
            yield_rate,
            world,
            cancelled: false,
        }
    }
}

/// Paper's `BlockExplodeEvent` extends `BlockEvent`; the block it reports is the one at the
/// explosion centre, which is where the event position points.
impl BlockEvent for BlockExplodeEvent {
    fn get_block(&self) -> &Block {
        self.world.get_block(&self.block_pos)
    }
}
