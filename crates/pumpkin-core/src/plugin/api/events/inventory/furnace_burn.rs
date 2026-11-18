use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

/// An event that occurs when an item is consumed as fuel in a furnace.
#[cancellable]
#[derive(Event, Clone)]
pub struct FurnaceBurnEvent {
    /// The position of the furnace block.
    pub block_pos: BlockPos,

    /// The registry key of the fuel item.
    pub fuel_item: String,

    /// The burn time in ticks.
    pub burn_time: u32,

    /// Whether the fuel item is consumed when the furnace ignites. Paper's
    /// `FurnaceBurnEvent#setConsumeFuel(false)` keeps the furnace lit but leaves the fuel slot
    /// untouched.
    pub consume_fuel: bool,
}

impl FurnaceBurnEvent {
    #[must_use]
    pub const fn new(block_pos: BlockPos, fuel_item: String, burn_time: u32) -> Self {
        Self {
            block_pos,
            fuel_item,
            burn_time,
            consume_fuel: true,
            cancelled: false,
        }
    }
}
