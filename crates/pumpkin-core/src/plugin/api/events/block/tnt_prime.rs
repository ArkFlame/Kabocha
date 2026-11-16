use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

use crate::world::World;

/// An event that occurs when TNT is primed.
#[cancellable]
#[derive(Event, Clone)]
pub struct TNTPrimeEvent {
    pub block_pos: BlockPos,
    pub prime_reason: String,
    pub world: Arc<World>,
}

impl TNTPrimeEvent {
    #[must_use]
    pub const fn new(block_pos: BlockPos, prime_reason: String, world: Arc<World>) -> Self {
        Self {
            block_pos,
            prime_reason,
            world,
            cancelled: false,
        }
    }
}
