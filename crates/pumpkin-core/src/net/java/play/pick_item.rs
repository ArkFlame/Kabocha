#[allow(clippy::wildcard_imports)]
use super::*;
use crate::entity::decoration::painting::PaintingEntity;
use crate::item::items::boat::BoatItem;
use pumpkin_data::data_component_impl::PaintingVariantImpl;
use pumpkin_data::tag::Taggable;

/// What a pick request does with the slot that holds the picked stack.
#[derive(Debug, PartialEq, Eq)]
enum PickSlotAction {
    /// Stack already on the hotbar: select it.
    SelectHotbar(usize),
    /// Stack in the main inventory: swap it onto the hotbar.
    SwapIntoHotbar(usize),
    /// No match in creative: put a copy on the hotbar.
    InsertStack,
    /// Out-of-range index, or no match outside creative.
    Ignore,
}

/// Maps the `get_slot_with_stack` result (`-1` on miss) to an action, bounded to the main inventory.
fn pick_slot_action(slot_with_stack: i16, creative: bool) -> PickSlotAction {
    match usize::try_from(slot_with_stack) {
        Ok(slot) if slot < PlayerInventory::MAIN_SIZE => {
            if PlayerInventory::is_valid_hotbar_index(slot) {
                PickSlotAction::SelectHotbar(slot)
            } else {
                PickSlotAction::SwapIntoHotbar(slot)
            }
        }
        Ok(_) => PickSlotAction::Ignore,
        Err(_) if creative => PickSlotAction::InsertStack,
        Err(_) => PickSlotAction::Ignore,
    }
}

fn apply_pick_slot(player: &Arc<Player>, stack: ItemStack) {
    let creative = player.gamemode.load() == GameMode::Creative;
    let slot_with_stack = player.inventory().get_slot_with_stack(&stack);
    match pick_slot_action(slot_with_stack, creative) {
        PickSlotAction::SelectHotbar(slot) => player.inventory.set_selected_slot(slot as u8),
        PickSlotAction::SwapIntoHotbar(slot) => player.inventory.swap_slot_with_hotbar(slot),
        PickSlotAction::InsertStack => player.inventory.swap_stack_with_hotbar(stack),
        PickSlotAction::Ignore => {}
    }
}

impl JavaClient {
    pub fn handle_pick_item_from_block(
        &self,
        player: &Arc<Player>,
        pick_item: &SPickItemFromBlock,
    ) {
        if !player.can_interact_with_block_at(&pick_item.pos, 1.0) {
            return;
        }

        let world = player.world();
        let block = world.get_block(&pick_item.pos);

        if block.item_id == 0 {
            // Invalid block id (blocks such as tall seagrass)
            return;
        }

        let Some(item) = Item::from_id(block.item_id) else {
            return;
        };
        let mut stack = ItemStack::new(1, item);
        if player.gamemode.load() == GameMode::Creative
            && pick_item.include_data
            && let Some(block_entity) = world.get_block_entity(&pick_item.pos)
        {
            let mut nbt = pumpkin_nbt::compound::NbtCompound::new();
            block_entity.write_nbt(&mut nbt);
            if !nbt.is_empty() {
                nbt.put_string("id", block_entity.resource_location().to_string());
                stack.set_data_component(pumpkin_data::data_component_impl::BlockEntityDataImpl {
                    nbt,
                });
            }
            block_entity.collect_pick_item_components(&mut stack);
        }
        if block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_COPPER_GOLEM_STATUES)
            && let Some(block_entity) = world.get_block_entity(&pick_item.pos)
            && let Some(statue) = block_entity.as_any().downcast_ref::<
                crate::block::entities::copper_golem_statue::CopperGolemStatueBlockEntity,
            >()
        {
            let props = pumpkin_data::block_properties::CopperGolemStatueLikeProperties::from_state_id(
                world.get_block_state_id(&pick_item.pos),
            );
            statue.get_item(&mut stack, props.copper_golem_pose);
        }

        apply_pick_slot(player, stack);

        player.try_send_client_packet(&CSetSelectedSlot::new(
            player.inventory.get_selected_slot() as i8
        ));
        player
            .player_screen_handler
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .send_content_updates();
    }

    pub fn handle_pick_item_from_entity(
        &self,
        player: &Arc<Player>,
        pick_item: &SPickItemFromEntity,
    ) {
        use pumpkin_data::entity::{entity_from_egg, spawn_egg_ids};

        let world = player.world();
        let Some(target) = world.get_entity_by_id(pick_item.id.0) else {
            return;
        };

        let p_eye = player.get_entity().get_eye_pos();
        let t_eye = target.get_eye_pos();
        let dx = p_eye.x - t_eye.x;
        let dy = p_eye.y - t_eye.y;
        let dz = p_eye.z - t_eye.z;
        if dx * dx + dy * dy + dz * dz > 64.0 {
            return;
        }

        let target_type_id = target.get_entity().entity_type.id;
        let mut found_egg: Option<u16> = None;
        for &egg_id in &spawn_egg_ids() {
            if let Some(et) = entity_from_egg(egg_id)
                && et.id == target_type_id
            {
                found_egg = Some(egg_id);
                break;
            }
        }

        let picked_stack = target
            .cast_any()
            .downcast_ref::<PaintingEntity>()
            .map_or_else(
                || {
                    found_egg
                        .and_then(Item::from_id)
                        .or_else(|| BoatItem::entity_to_item(target.get_entity().entity_type))
                        .map(|item| ItemStack::new(1, item))
                },
                |painting| {
                    // Vanilla `Painting#getPickResult` returns the painting item carrying the
                    // variant of the entity that was picked.
                    let mut stack = ItemStack::new(1, &Item::PAINTING);
                    stack.set_data_component(PaintingVariantImpl {
                        value: std::borrow::Cow::Borrowed(painting.variant().to_name()),
                    });
                    Some(stack)
                },
            );

        if let Some(stack) = picked_stack {
            apply_pick_slot(player, stack);

            player.try_send_client_packet(&CSetSelectedSlot::new(
                player.inventory.get_selected_slot() as i8,
            ));
            player
                .player_screen_handler
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .send_content_updates();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pick_slot_action_is_bounded_to_main_inventory() {
        assert_eq!(pick_slot_action(-1, false), PickSlotAction::Ignore);
        assert_eq!(pick_slot_action(-1, true), PickSlotAction::InsertStack);
        assert_eq!(pick_slot_action(3, false), PickSlotAction::SelectHotbar(3));
        assert_eq!(
            pick_slot_action(20, false),
            PickSlotAction::SwapIntoHotbar(20)
        );
        assert_eq!(
            pick_slot_action(35, true),
            PickSlotAction::SwapIntoHotbar(35)
        );
        assert_eq!(pick_slot_action(36, true), PickSlotAction::Ignore);
        assert_eq!(pick_slot_action(i16::MAX, true), PickSlotAction::Ignore);
    }
}
