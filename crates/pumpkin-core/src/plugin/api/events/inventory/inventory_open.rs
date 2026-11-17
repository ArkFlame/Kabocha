use crate::entity::player::Player;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

/// An event that occurs when a player opens an inventory.
#[cancellable]
#[derive(Event, Clone)]
pub struct InventoryOpenEvent {
    /// The player opening the inventory.
    pub player: Arc<Player>,
    /// The position of the opened container block, if any.
    pub position: Option<BlockPos>,
}

impl InventoryOpenEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, position: Option<BlockPos>) -> Self {
        Self {
            player,
            position,
            cancelled: false,
        }
    }
}
