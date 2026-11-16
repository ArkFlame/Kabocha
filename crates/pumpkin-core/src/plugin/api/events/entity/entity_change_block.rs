use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

use crate::world::World;

/// An event that occurs when an entity changes a block in the world.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityChangeBlockEvent {
    /// The ID of the entity.
    pub entity_id: i32,
    /// Position of the block.
    pub block_pos: BlockPos,
    /// The new block state identifier.
    pub new_block: String,
    /// The world the block changes in.
    pub world: Arc<World>,
}

impl EntityChangeBlockEvent {
    #[must_use]
    pub const fn new(
        entity_id: i32,
        block_pos: BlockPos,
        new_block: String,
        world: Arc<World>,
    ) -> Self {
        Self {
            entity_id,
            block_pos,
            new_block,
            world,
            cancelled: false,
        }
    }
}
