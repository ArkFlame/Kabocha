#[allow(clippy::wildcard_imports)]
use super::*;

/// Vanilla `Inventory.getSelectionSize()`.
const HOTBAR_SIZE: i16 = 9;

/// Maps a client-claimed held slot to a hotbar index, `None` outside `0..HOTBAR_SIZE`.
fn hotbar_slot(slot: i16) -> Option<u8> {
    (0..HOTBAR_SIZE).contains(&slot).then_some(slot as u8)
}

impl JavaClient {
    pub fn handle_set_held_item(&self, server: &Arc<Server>, player: &Player, held: &SSetHeldItem) {
        player.update_last_action_time();
        // Vanilla `handleSetCarriedItem` logs and ignores an out-of-range slot instead of kicking.
        let Some(slot) = hotbar_slot(held.slot) else {
            warn!(
                "Player id {} tried to set an invalid carried item slot {}",
                player.entity_id(),
                held.slot
            );
            return;
        };
        let previous_slot = player.inventory.get_selected_slot();
        let Some(player_arc) = player.world().get_player_by_uuid(player.gameprofile.id) else {
            return;
        };
        let mut event = PlayerItemHeldEvent::new(player_arc, previous_slot, slot);
        server.plugin_manager.fire_blocking(server, &mut event);
        if event.cancelled {
            player.try_send_client_packet(&CSetSelectedSlot::new(previous_slot as i8));
            return;
        }

        // Vanilla `ServerGamePacketListenerImpl.handleSetCarriedItem` stops using the item when
        // the main hand slot changes, so a switch cancels eating/drinking instead of completing it.
        let active_hand = *player
            .living_entity
            .active_hand
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if previous_slot != slot && active_hand == Some(Hand::Right) {
            // Vanilla `LivingEntity.stopUsingItem` calls `ItemStack.releaseUsing` before clearing
            // the active item, so switching slots mid-draw fires the bow instead of cancelling it.
            let item_in_use = player
                .living_entity
                .item_in_use
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone();
            if let Some(stack) = item_in_use {
                server.item_registry.on_stopped_using(&stack, player);
            }
            player.living_entity.clear_active_hand();
        }

        let inv = player.inventory();
        inv.set_selected_slot(slot);
        let stack = inv.held_item();
        let equipment = &[(EquipmentSlot::MAIN_HAND, stack)];
        player.living_entity.send_equipment_changes(equipment);
    }
}

#[cfg(test)]
mod tests {
    use super::hotbar_slot;

    #[test]
    fn held_slot_is_bounded_to_hotbar() {
        assert_eq!(hotbar_slot(0), Some(0));
        assert_eq!(hotbar_slot(8), Some(8));
        assert_eq!(hotbar_slot(9), None);
        assert_eq!(hotbar_slot(-1), None);
        assert_eq!(hotbar_slot(i16::MAX), None);
    }
}
