use pumpkin_data::{Block, BlockState, FacingExt};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::GameMode;
use pumpkin_world::world::BlockFlags;

use crate::block::{BlockBehaviour, BrokenArgs, PathComputationType};

use super::piston::PistonProps;

pub(crate) type MovingPistonProps = pumpkin_data::block_properties::MovingPistonLikeProperties;

#[pumpkin_block("minecraft:moving_piston")]
pub struct PistonExtensionBlock;

impl BlockBehaviour for PistonExtensionBlock {
    fn broken(&self, args: BrokenArgs<'_>) {
        // Vanilla only removes the base behind a moving piston for creative players.
        if args.player.gamemode.load() != GameMode::Creative {
            return;
        }
        let props = MovingPistonProps::from_state_id(args.state.id);
        let pos = args
            .position
            .offset(props.facing.opposite().to_block_direction().to_offset());
        let (new_block, new_state) = args.world.get_block_and_state_id(&pos);
        if &Block::PISTON == new_block
            || &Block::STICKY_PISTON == new_block
            || &Block::MOVING_PISTON == new_block
        {
            if &Block::MOVING_PISTON != new_block {
                let base_props = PistonProps::from_state_id(new_state);
                if !base_props.extended {
                    return;
                }
            }
            args.world.break_block(
                &pos,
                Some(args.player),
                BlockFlags::SKIP_DROPS | BlockFlags::NOTIFY_ALL,
            );
        }
    }

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}
