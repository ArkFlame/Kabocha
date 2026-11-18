use pumpkin_data::{Block, BlockState, BlockStateId, FacingExt};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::GameMode;
use pumpkin_world::world::BlockFlags;

use crate::block::{BlockBehaviour, BrokenArgs, PathComputationType};

use super::piston::PistonProps;

pub(crate) type MovingPistonProps = pumpkin_data::block_properties::MovingPistonLikeProperties;

#[pumpkin_block("minecraft:moving_piston")]
pub struct PistonExtensionBlock;

impl PistonExtensionBlock {
    // Vanilla `MovingPistonBlock.destroy` (see
    // .vendor/vanilla-decompiled/net/minecraft/world/level/block/piston/MovingPistonBlock.java)
    // is reached here only on a player break, where the hardness -1 moving piston can be
    // mined only in creative. It removes an extended `PistonBaseBlock`, never a moving piston.
    fn should_remove_base(
        player_is_creative: bool,
        base_block: &Block,
        base_state_id: BlockStateId,
    ) -> bool {
        if !player_is_creative {
            return false;
        }
        if base_block != &Block::PISTON && base_block != &Block::STICKY_PISTON {
            return false;
        }
        PistonProps::from_state_id(base_state_id).extended
    }
}

impl BlockBehaviour for PistonExtensionBlock {
    fn broken(&self, args: BrokenArgs<'_>) {
        let props = MovingPistonProps::from_state_id(args.state.id);
        let base_pos = args
            .position
            .offset(props.facing.opposite().to_block_direction().to_offset());
        let (base_block, base_state_id) = args.world.get_block_and_state_id(&base_pos);
        if !Self::should_remove_base(
            args.player.gamemode.load() == GameMode::Creative,
            base_block,
            base_state_id,
        ) {
            return;
        }
        args.world.break_block(
            &base_pos,
            Some(args.player),
            BlockFlags::SKIP_DROPS | BlockFlags::NOTIFY_ALL,
        );
    }

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn piston_state(block: &Block, extended: bool) -> BlockStateId {
        let mut props = PistonProps::default(block);
        props.extended = extended;
        props.to_state_id(block)
    }

    #[test]
    fn removes_only_extended_piston_base_for_creative_break() {
        assert!(PistonExtensionBlock::should_remove_base(
            true,
            &Block::PISTON,
            piston_state(&Block::PISTON, true),
        ));
        assert!(PistonExtensionBlock::should_remove_base(
            true,
            &Block::STICKY_PISTON,
            piston_state(&Block::STICKY_PISTON, true),
        ));
        assert!(!PistonExtensionBlock::should_remove_base(
            true,
            &Block::PISTON,
            piston_state(&Block::PISTON, false),
        ));
        assert!(!PistonExtensionBlock::should_remove_base(
            true,
            &Block::MOVING_PISTON,
            Block::MOVING_PISTON.default_state.id,
        ));
        assert!(!PistonExtensionBlock::should_remove_base(
            false,
            &Block::PISTON,
            piston_state(&Block::PISTON, true),
        ));
    }
}
