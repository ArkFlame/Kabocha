use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, tag};

/// Mirrors vanilla `FlowingFluid.canHoldFluid` for flowing fluid.
///
/// A `LiquidBlockContainer` passes `canHoldAnyFluid`, but its `canPlaceLiquid` rejects flowing
/// fluid, so it never holds it. Vanilla only waterlogs one when source water reaches it through
/// source conversion, which Pumpkin's flow doesn't do yet.
#[must_use]
pub fn can_hold_fluid(block: &Block) -> bool {
    block.has_tag(&tag::Block::MINECRAFT_WASHED_AWAY_BY_FLUIDS) && !is_liquid_block_container(block)
}

/// Blocks implementing vanilla's `LiquidBlockContainer`: every `SimpleWaterloggedBlock`, plus
/// kelp, seagrass and lily pad. The last three are in the washed-away tag but carry no
/// `waterlogged` property, so they have to be named explicitly.
pub fn is_liquid_block_container(block: &Block) -> bool {
    block.is_waterloggable()
        || block == &Block::KELP
        || block == &Block::KELP_PLANT
        || block == &Block::SEAGRASS
        || block == &Block::TALL_SEAGRASS
        || block == &Block::LILY_PAD
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lily_pad_is_a_liquid_block_container() {
        // Vanilla `WaterlilyBlock` implements `LiquidBlockContainer` with a rejecting
        // `canPlaceLiquid`, so neither flowing fluid nor a water bucket replaces it.
        assert!(is_liquid_block_container(&Block::LILY_PAD));
        assert!(!can_hold_fluid(&Block::LILY_PAD));
        assert!(can_hold_fluid(&Block::FIRE));
        assert!(!can_hold_fluid(&Block::STONE));
    }
}
