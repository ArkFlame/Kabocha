use crate::block::{
    BlockBehaviour, BlockMetadata, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, OnPlaceArgs,
    can_support_center,
};
use crate::entity::EntityBase;
use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::Facing;
use pumpkin_data::{Block, FacingExt, HorizontalFacingExt};
use pumpkin_data::{BlockDirection, BlockId};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockAccessor;

type WallTorchProps = pumpkin_data::block_properties::WallTorchLikeProperties;
// Normal tourches don't have properties

pub struct TorchBlock;

impl BlockMetadata for TorchBlock {
    fn ids() -> Box<[BlockId]> {
        [
            BlockId::TORCH,
            BlockId::SOUL_TORCH,
            BlockId::WALL_TORCH,
            BlockId::SOUL_WALL_TORCH,
            BlockId::COPPER_TORCH,
            BlockId::COPPER_WALL_TORCH,
        ]
        .into()
    }
}

impl BlockBehaviour for TorchBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut directions = args.player.get_entity().get_entity_facing_order();
        // Vanilla `BlockPlaceContext.getNearestLookingDirections` reorders only when the
        // clicked block does not give way, i.e. the block lands at the relative position.
        if *args.position != args.use_item_on.position {
            prefer_facing(&mut directions, args.direction.to_facing());
        }
        // Vanilla `StandingAndWallBlockItem` walks one ordered list: DOWN asks for the
        // standing torch, horizontals for the wall torch, UP is skipped.
        for dir in directions {
            if dir == Facing::Down {
                if can_support_center(args.world, &args.position.down(), BlockDirection::Up) {
                    return args.block.default_state.id;
                }
            } else if dir != Facing::Up
                && can_place_at(args.world, args.position, dir.to_block_direction())
            {
                let wall_block = {
                    if args.block == &Block::TORCH {
                        Block::WALL_TORCH
                    } else if args.block == &Block::SOUL_TORCH {
                        Block::SOUL_WALL_TORCH
                    } else {
                        Block::COPPER_WALL_TORCH
                    }
                };
                let mut torch_props = WallTorchProps::default(&wall_block);
                if let Some(facing) = dir.opposite().to_horizontal_facing() {
                    torch_props.facing = facing;
                    return torch_props.to_state_id(&wall_block);
                }
            }
        }
        BlockStateId::AIR
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        if args.player.is_none() {
            if args.block == &Block::WALL_TORCH
                || args.block == &Block::SOUL_WALL_TORCH
                || args.block == &Block::COPPER_WALL_TORCH
            {
                let props = WallTorchProps::from_state_id(args.state.id);
                return can_place_at(
                    args.block_accessor,
                    args.position,
                    props.facing.to_block_direction().opposite(),
                );
            }
            return can_support_center(
                args.block_accessor,
                &args.position.down(),
                BlockDirection::Up,
            );
        }

        if can_support_center(
            args.block_accessor,
            &args.position.down(),
            BlockDirection::Up,
        ) {
            return true;
        }
        for dir in BlockDirection::horizontal() {
            if can_place_at(args.block_accessor, args.position, dir.to_block_direction()) {
                return true;
            }
        }
        false
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if args.block == &Block::WALL_TORCH
            || args.block == &Block::SOUL_WALL_TORCH
            || args.block == &Block::COPPER_WALL_TORCH
        {
            let props = WallTorchProps::from_state_id(args.state_id);
            if props.facing.to_block_direction().opposite() == args.direction
                && !can_place_at(
                    args.world,
                    args.position,
                    props.facing.to_block_direction().opposite(),
                )
            {
                return BlockStateId::AIR;
            }
        } else if args.direction == BlockDirection::Down
            && !can_support_center(args.world, &args.position.down(), BlockDirection::Up)
        {
            return BlockStateId::AIR;
        }
        args.state_id
    }
}

fn can_place_at(world: &dyn BlockAccessor, block_pos: &BlockPos, facing: BlockDirection) -> bool {
    world
        .get_block_state(&block_pos.offset(facing.to_offset()))
        .is_side_solid(facing.opposite())
}

/// Moves `face` to the front of `directions`, keeping the rest of the order, the same
/// reorder vanilla `BlockPlaceContext.getNearestLookingDirections` applies.
fn prefer_facing(directions: &mut [Facing; 6], face: Facing) {
    let Some(index) = directions.iter().position(|&dir| dir == face) else {
        return;
    };
    if index > 0 {
        directions.copy_within(0..index, 1);
        directions[0] = face;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefer_facing_moves_the_clicked_face_first() {
        let mut directions = [
            Facing::North,
            Facing::South,
            Facing::East,
            Facing::West,
            Facing::Up,
            Facing::Down,
        ];
        prefer_facing(&mut directions, Facing::East);
        assert_eq!(
            directions,
            [
                Facing::East,
                Facing::North,
                Facing::South,
                Facing::West,
                Facing::Up,
                Facing::Down,
            ]
        );

        prefer_facing(&mut directions, Facing::North);
        assert_eq!(directions[0], Facing::North);
    }
}
