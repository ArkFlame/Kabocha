use pumpkin_data::fluid::Fluid;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, tag};

/// Mirrors vanilla `FlowingFluid.canHoldFluid` for the given fluid.
///
/// A `LiquidBlockContainer` is only a hole for a fluid its `canPlaceLiquid` accepts:
/// `SimpleWaterloggedBlock` (slabs, stairs, ...) accepts water and sets its `waterlogged`
/// property, but rejects lava. Kelp, seagrass and lily pad reject every fluid. Any other
/// block in `#minecraft:washed_away_by_fluids` can hold fluid.
#[must_use]
pub fn can_hold_fluid(block: &Block, fluid: &Fluid) -> bool {
    if is_liquid_block_container(block) {
        return block.is_waterloggable()
            && (fluid.id == Fluid::FLOWING_WATER.id || fluid.id == Fluid::WATER.id);
    }
    block.has_tag(&tag::Block::MINECRAFT_WASHED_AWAY_BY_FLUIDS)
}

/// Blocks implementing vanilla's `LiquidBlockContainer`.
///
/// Every `SimpleWaterloggedBlock` counts, plus kelp, seagrass and lily pad. The last three are
/// in the washed-away tag but carry no `waterlogged` property, so they are named explicitly.
#[must_use]
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
    fn kelp_and_seagrass_are_liquid_block_containers_but_stone_is_not() {
        assert!(is_liquid_block_container(&Block::KELP));
        assert!(is_liquid_block_container(&Block::TALL_SEAGRASS));
        assert!(!is_liquid_block_container(&Block::STONE));
    }

    #[test]
    fn lily_pad_is_a_liquid_block_container() {
        // Vanilla `WaterlilyBlock` implements `LiquidBlockContainer` with a rejecting
        // `canPlaceLiquid`, so neither flowing fluid nor a water bucket replaces it.
        assert!(is_liquid_block_container(&Block::LILY_PAD));
        assert!(!can_hold_fluid(&Block::LILY_PAD, &Fluid::FLOWING_WATER));
        assert!(can_hold_fluid(&Block::FIRE, &Fluid::FLOWING_WATER));
        assert!(!can_hold_fluid(&Block::STONE, &Fluid::FLOWING_WATER));
    }

    #[test]
    fn water_is_washed_away_so_lava_needs_no_special_hole_case() {
        // Water is in #minecraft:washed_away_by_fluids and is not a liquid container,
        // so `can_hold_fluid` already reports it as a hole for every fluid, lava included.
        assert!(can_hold_fluid(&Block::WATER, &Fluid::FLOWING_LAVA));
    }

    #[test]
    fn waterloggable_blocks_hold_water_but_not_lava() {
        // Vanilla `SimpleWaterloggedBlock.canPlaceLiquid` accepts only water, so a
        // waterloggable slab is a hole for water but never for lava.
        let slab = &Block::OAK_SLAB;
        assert!(slab.is_waterloggable());
        assert!(can_hold_fluid(slab, &Fluid::FLOWING_WATER));
        assert!(!can_hold_fluid(slab, &Fluid::FLOWING_LAVA));
    }

    #[test]
    fn dry_waterloggable_state_can_be_waterlogged() {
        let slab = &Block::OAK_SLAB;
        let dry = slab.default_state.id;
        assert!(!slab.is_waterlogged(dry));
        let wet = slab.set_waterlogged(dry, true).unwrap_or(dry);
        assert!(slab.is_waterlogged(wet));
    }
}
