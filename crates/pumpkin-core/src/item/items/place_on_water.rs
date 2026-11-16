use std::any::Any;

use crate::block::registry::BlockActionResult;
use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use crate::world::BlockFlags;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::statistic::StatisticCategory;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::{Block, BlockDirection, BlockId};
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_util::Hand;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

pub struct PlaceOnWaterBlockItem;

impl ItemMetadata for PlaceOnWaterBlockItem {
    fn ids() -> Box<[u16]> {
        [Item::LILY_PAD.id, Item::FROGSPAWN.id].into()
    }
}

impl ItemBehaviour for PlaceOnWaterBlockItem {
    fn use_on_block(
        &self,
        _item: &mut ItemStack,
        _player: &Player,
        _location: BlockPos,
        _face: BlockDirection,
        _cursor_pos: Vector3<f32>,
        _block: &Block,
        _server: &Server,
    ) -> BlockActionResult {
        // Vanilla PlaceOnWaterBlockItem.useOn returns PASS without calling BlockItem.useOn,
        // and instead places via use() after raycasting Fluid.SOURCE_ONLY.
        BlockActionResult::Fail
    }

    fn normal_use(&self, item: &Item, player: &Player, hand: Hand) {
        let (yaw, pitch) = player.rotation();
        self.normal_use_with_rotation(item, player, yaw, pitch, hand);
    }

    fn normal_use_with_rotation(
        &self,
        item: &Item,
        player: &Player,
        yaw: f32,
        pitch: f32,
        hand: Hand,
    ) {
        let world = player.world();
        let (start_pos, end_pos) = self.get_start_and_end_pos_with_rotation(player, yaw, pitch);
        let hit = world.clip(&pumpkin_world::clip::ClipContext::new(
            start_pos,
            end_pos,
            pumpkin_world::clip::ClipBlock::Outline,
            pumpkin_world::clip::ClipFluid::SourceOnly,
        ));
        if hit.is_miss {
            return;
        }

        let hit_pos = hit.block_pos;
        let above_pos = hit_pos.up();
        if !world.is_in_build_limit(above_pos) {
            return;
        }
        let above_state = world.get_block_state(&above_pos);
        if !above_state.is_air() || *world.get_fluid(&above_pos) != Fluid::EMPTY {
            return;
        }

        let hit_block = world.get_block(&hit_pos);
        let hit_state = world.get_block_state(&hit_pos);
        let hit_fluid = world.get_fluid(&hit_pos);
        let can_support = if item.id == Item::LILY_PAD.id {
            // Vanilla checks the fluid state (`minecraft:supports_lily_pad` holds source water
            // only), so waterlogged blocks count too. `get_fluid` normalizes sources to flowing
            // water, hence the state lookup.
            let is_source_water = hit_state.is_waterlogged()
                || Fluid::from_state_id(hit_state.id)
                    .is_some_and(|fluid| fluid.id == Fluid::WATER.id)
                // These blocks hold source water without a `waterlogged` property.
                || matches!(
                    hit_block.id,
                    BlockId::KELP
                        | BlockId::KELP_PLANT
                        | BlockId::SEAGRASS
                        | BlockId::TALL_SEAGRASS
                        | BlockId::BUBBLE_COLUMN
                );
            is_source_water || hit_block.has_tag(&tag::Block::MINECRAFT_SUPPORTS_LILY_PAD)
        } else {
            hit_fluid.has_tag(&tag::Fluid::MINECRAFT_SUPPORTS_FROGSPAWN)
                || hit_block.has_tag(&tag::Block::MINECRAFT_SUPPORTS_FROGSPAWN)
                || *hit_fluid == Fluid::WATER
                || hit_block == &Block::WATER
        };
        if !can_support {
            return;
        }

        let (placed_block, sound) = if item.id == Item::LILY_PAD.id {
            (&Block::LILY_PAD, Sound::BlockLilyPadPlace)
        } else {
            (&Block::FROGSPAWN, Sound::BlockFrogspawnPlace)
        };

        world.set_block_state(
            &above_pos,
            placed_block.default_state.id,
            BlockFlags::NOTIFY_ALL,
        );
        world.play_sound(sound, SoundCategory::Blocks, &above_pos.to_f64());
        world.emit_game_event(GameEvent::BlockPlace.name(), above_pos.to_f64());

        let mut stack = player.inventory.get_stack_in_hand(hand);
        if !stack.is_empty() && stack.item.id == item.id {
            stack.decrement_unless_creative(player.gamemode.load(), 1);
            player.inventory.set_stack_in_hand(hand, stack.clone());
            let slot_index = match hand {
                Hand::Right => player.inventory.get_selected_slot() as usize,
                Hand::Left => PlayerInventory::OFF_HAND_SLOT,
            };
            player.sync_hand_slot(slot_index, stack);
        }
        player.increment_stat(StatisticCategory::Used, i32::from(item.id), 1);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
