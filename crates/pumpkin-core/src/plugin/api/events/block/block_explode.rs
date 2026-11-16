use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

use crate::world::World;

/// An event that occurs when a block explodes.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockExplodeEvent {
    pub block_pos: BlockPos,
    pub yield_rate: f32,
    pub world: Arc<World>,
}

impl BlockExplodeEvent {
    #[must_use]
    pub const fn new(block_pos: BlockPos, yield_rate: f32, world: Arc<World>) -> Self {
        Self {
            block_pos,
            yield_rate,
            world,
            cancelled: false,
        }
    }
}
