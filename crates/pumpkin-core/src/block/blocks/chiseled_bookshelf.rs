use std::sync::{Arc, atomic::Ordering};

use pumpkin_macros::pumpkin_block;

use crate::block::entities::chiseled_bookshelf::ChiseledBookshelfBlockEntity;
use crate::{
    block::{
        BlockBehaviour, GetComparatorOutputArgs, NormalUseArgs, OnPlaceArgs, PlacedArgs,
        UseWithItemArgs, registry::BlockActionResult,
    },
    entity::{EntityBase, player::Player},
    world::World,
};
use pumpkin_data::{
    BlockStateId,
    block_properties::ChiseledBookshelfLikeProperties,
    item::Item,
    item_stack::ItemStack,
    sound::{Sound, SoundCategory},
    statistic::StatisticCategory,
    tag,
    tag::Taggable,
};
use pumpkin_inventory::screen_handler::InventoryPlayer;
use pumpkin_util::math::position::BlockPos;

#[pumpkin_block("minecraft:chiseled_bookshelf")]
pub struct ChiseledBookshelfBlock;

impl ChiseledBookshelfBlock {
    const ROWS: usize = 2;
    const COLUMNS: usize = 3;
}

impl BlockBehaviour for ChiseledBookshelfBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut properties = ChiseledBookshelfLikeProperties::default(args.block);

        // Face in the opposite direction the player is facing
        properties.facing = args.player.get_entity().get_horizontal_facing().opposite();

        properties.to_state_id(args.block)
    }

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        let state = args.world.get_block_state(args.position);
        let properties = ChiseledBookshelfLikeProperties::from_state_id(state.id);

        let Some(slot) = args
            .hit
            .get_hit_slot(properties.facing, Self::ROWS, Self::COLUMNS)
        else {
            return BlockActionResult::Pass;
        };
        // Vanilla reads occupancy from the block-entity stack; the blockstate slot
        // flags are only render hints and can disagree after /setblock.
        let Some(block_entity) = args.world.get_block_entity(args.position) else {
            return BlockActionResult::Pass;
        };
        let Some(block_entity) = block_entity
            .as_any()
            .downcast_ref::<ChiseledBookshelfBlockEntity>()
        else {
            return BlockActionResult::Pass;
        };
        if Self::try_remove_book(
            args.world,
            args.player,
            args.position,
            block_entity,
            properties,
            slot,
        ) {
            return BlockActionResult::Success;
        }
        BlockActionResult::Pass
    }

    fn use_with_item(&self, args: UseWithItemArgs<'_>) -> BlockActionResult {
        let state = args.world.get_block_state(args.position);
        let properties = ChiseledBookshelfLikeProperties::from_state_id(state.id);

        let Some(slot) = args
            .hit
            .get_hit_slot(properties.facing, Self::ROWS, Self::COLUMNS)
        else {
            return BlockActionResult::Pass;
        };
        let Some(block_entity) = args.world.get_block_entity(args.position) else {
            return BlockActionResult::Pass;
        };
        let Some(block_entity) = block_entity
            .as_any()
            .downcast_ref::<ChiseledBookshelfBlockEntity>()
        else {
            return BlockActionResult::Pass;
        };

        // Vanilla resolves the hit slot before checking the held item and returns
        // TRY_WITH_EMPTY_HAND for an occupied slot regardless of the item, which
        // Pumpkin models as falling through to `normal_use`.
        if block_entity.has_book(slot) {
            return BlockActionResult::PassToDefaultBlockAction;
        }
        if !args
            .item_stack
            .get_item()
            .has_tag(&tag::Item::MINECRAFT_BOOKSHELF_BOOKS)
        {
            return BlockActionResult::Pass;
        }

        Self::try_add_book(
            args.world,
            args.player,
            args.position,
            block_entity,
            properties,
            slot,
            args.item_stack,
        );
        BlockActionResult::Success
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        let block_entity = ChiseledBookshelfBlockEntity::new(*args.position);
        args.world.add_block_entity(Arc::new(block_entity));
    }

    fn get_comparator_output(&self, args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        if let Some(block_entity) = args.world.get_block_entity(args.position)
            && let Some(block_entity) = block_entity
                .as_any()
                .downcast_ref::<ChiseledBookshelfBlockEntity>()
        {
            return Some((block_entity.last_interacted_slot.load(Ordering::Relaxed) + 1) as u8);
        }
        None
    }
}

impl ChiseledBookshelfBlock {
    fn try_add_book(
        world: &Arc<World>,
        player: &Player,
        position: &BlockPos,
        entity: &ChiseledBookshelfBlockEntity,
        properties: ChiseledBookshelfLikeProperties,
        slot: usize,
        item: &mut ItemStack,
    ) {
        let used_item_id = item.item.id;
        player.increment_stat(StatisticCategory::Used, i32::from(used_item_id), 1);

        let sound = if item.get_item() == &Item::ENCHANTED_BOOK {
            Sound::BlockChiseledBookshelfInsertEnchanted
        } else {
            Sound::BlockChiseledBookshelfInsert
        };

        entity.set_book(slot, item.split_unless_creative(player.gamemode.load(), 1));
        entity.update_state(properties, world, slot);

        world.play_sound(sound, SoundCategory::Blocks, &position.to_centered_f64());
    }

    fn try_remove_book(
        world: &Arc<World>,
        player: &Arc<Player>,
        position: &BlockPos,
        entity: &ChiseledBookshelfBlockEntity,
        properties: ChiseledBookshelfLikeProperties,
        slot: usize,
    ) -> bool {
        let mut stack = entity.remove_book(slot, 1);
        if stack.is_empty() {
            return false;
        }

        let sound = if stack.get_item() == &Item::ENCHANTED_BOOK {
            Sound::BlockChiseledBookshelfPickupEnchanted
        } else {
            Sound::BlockChiseledBookshelfPickup
        };

        // Vanilla: creative players do not collect the removed book.
        if !player.is_creative() {
            if player.get_inventory().insert_stack_anywhere(&mut stack) {
                player.sync_inventory_to_client();
            }
            if !stack.is_empty() {
                player.drop_item(stack);
            }
        }
        entity.update_state(properties, world, slot);

        world.play_sound(sound, SoundCategory::Blocks, &position.to_centered_f64());
        true
    }
}
