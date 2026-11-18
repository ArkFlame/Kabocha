use crate::block::push_entities_up;
use crate::block::{
    BlockBehaviour, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, OnPlaceArgs,
    OnScheduledTickArgs, PathComputationType,
};
use pumpkin_data::{
    Block, BlockDirection, BlockState, BlockStateId,
    tag::{self, Taggable},
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockAccessor;
use pumpkin_world::world::BlockFlags;

#[pumpkin_block("minecraft:dirt_path")]
pub struct DirtPathBlock;

impl BlockBehaviour for DirtPathBlock {
    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        // Vanilla PathBlock#tick returns early when the block can survive again;
        // only turnToBaseBlock (which pushes entities up) runs otherwise.
        if can_place_at(args.world.as_ref(), args.position) {
            return;
        }
        push_entities_up(
            args.world,
            args.position,
            args.world.get_block_state_id(args.position),
            Block::DIRT.default_state.id,
        );
        args.world.set_block_state(
            args.position,
            Block::DIRT.default_state.id,
            BlockFlags::NOTIFY_ALL,
        );
    }

    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        if !can_place_at(args.world, args.position) {
            // Vanilla getStateForPlacement returns pushEntitiesUp(path, dirt, ...)
            // for a blocked placement, lifting entities in the now-taller column.
            push_entities_up(
                args.world,
                args.position,
                args.block.default_state.id,
                Block::DIRT.default_state.id,
            );
            return Block::DIRT.default_state.id;
        }

        args.block.default_state.id
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if args.direction == BlockDirection::Up && !can_place_at(args.world, args.position) {
            args.world
                .schedule_block_tick(args.block, *args.position, 1, TickPriority::Normal);
        }
        args.state_id
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        can_place_at(args.block_accessor, args.position)
    }

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}

fn can_place_at(world: &dyn BlockAccessor, block_pos: &BlockPos) -> bool {
    let above_pos = block_pos.up();
    let state = world.get_block_state(&above_pos);
    !state.is_solid()
        || world
            .get_block(&above_pos)
            .has_tag(&tag::Block::MINECRAFT_FENCE_GATES)
}

#[cfg(test)]
mod tests {
    use pumpkin_data::BlockState;
    use rustc_hash::FxHashMap;

    use super::*;

    #[derive(Default)]
    struct TestBlocks(FxHashMap<BlockPos, BlockStateId>);

    impl BlockAccessor for TestBlocks {
        fn get_block(&self, position: &BlockPos) -> &'static Block {
            self.get_block_state_id(position).to_block()
        }

        fn get_block_state(&self, position: &BlockPos) -> &'static BlockState {
            BlockState::from_id(self.get_block_state_id(position))
        }

        fn get_block_state_id(&self, position: &BlockPos) -> BlockStateId {
            self.0
                .get(position)
                .copied()
                .unwrap_or(Block::AIR.default_state.id)
        }

        fn get_block_and_state(
            &self,
            position: &BlockPos,
        ) -> (&'static Block, &'static BlockState) {
            BlockState::from_id_with_block(self.get_block_state_id(position))
        }
    }

    #[test]
    fn can_place_respects_obstruction_and_fence_gate() {
        let position = BlockPos::new(0, 64, 0);
        let above = position.up();

        let open = TestBlocks::default();
        assert!(can_place_at(&open, &position));

        let mut solid = TestBlocks::default();
        solid.0.insert(above, Block::STONE.default_state.id);
        assert!(!can_place_at(&solid, &position));

        let mut gate = TestBlocks::default();
        gate.0.insert(above, Block::OAK_FENCE_GATE.default_state.id);
        assert!(can_place_at(&gate, &position));
    }
}
