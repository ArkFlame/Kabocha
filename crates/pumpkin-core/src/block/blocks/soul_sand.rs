use std::sync::Arc;

use pumpkin_data::Block;
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;

use crate::block::{
    BlockBehaviour, OnNeighborUpdateArgs, OnScheduledTickArgs, PathComputationType, PlacedArgs,
    blocks::bubble_column,
};
use crate::world::World;

/// Vanilla `SoulSandBlock.BUBBLE_COLUMN_CHECK_DELAY`.
const BUBBLE_COLUMN_CHECK_DELAY: u32 = 20;

#[pumpkin_block("minecraft:soul_sand")]
pub struct SoulSandBlock;

impl BlockBehaviour for SoulSandBlock {
    fn is_pathfindable(
        &self,
        _state: &pumpkin_data::BlockState,
        _computation_type: PathComputationType,
    ) -> bool {
        false
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        schedule_bubble_column_check(args.world, *args.position);
    }

    fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        // Vanilla only reacts to the block above becoming water; we cannot tell the update
        // direction here, so a water block above is the equivalent check.
        if args.world.get_block(&args.position.up()) == &Block::WATER {
            schedule_bubble_column_check(args.world, *args.position);
        }
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        if args.world.get_block(args.position) != &Block::SOUL_SAND {
            return;
        }
        bubble_column::update_column(args.world, args.position.up());
    }
}

fn schedule_bubble_column_check(world: &Arc<World>, position: BlockPos) {
    world.schedule_block_tick(
        &Block::SOUL_SAND,
        position,
        BUBBLE_COLUMN_CHECK_DELAY,
        TickPriority::Normal,
    );
}
