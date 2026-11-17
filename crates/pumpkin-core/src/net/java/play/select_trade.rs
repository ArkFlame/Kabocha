#[allow(clippy::wildcard_imports)]
use super::*;

/// The `u8` offer index passed to `TradeSelectEvent`, or `None` when the index does not fit.
fn event_slot_index(selected_slot: i32) -> Option<u8> {
    u8::try_from(selected_slot).ok()
}

impl JavaClient {
    pub fn handle_select_trade(&self, player: &Arc<Player>, packet: &SSelectTrade) {
        // Vanilla ignores negative trade indices; they must not reach the usize offer index.
        let Ok(selected_offer) = usize::try_from(packet.selected_slot.0) else {
            return;
        };
        // The event carries a `u8`; an index that does not fit is not fired so plugins never see
        // a truncated offer. The merchant still bounds the selection itself.
        if let Some(slot_index) = event_slot_index(packet.selected_slot.0) {
            let mut event =
                crate::plugin::api::events::inventory::trade_select::TradeSelectEvent::new(
                    player.clone(),
                    slot_index,
                );
            if let Some(server) = player.world().server.upgrade() {
                server.plugin_manager.fire_blocking(&server, &mut event);
            }
            if event.cancelled {
                return;
            }
        }

        let screen_handler = player
            .current_screen_handler
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        let mut screen_handler = screen_handler
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !screen_handler.can_use(player.as_ref()) {
            return;
        }
        if let Some(merchant) = screen_handler
            .as_any_mut()
            .downcast_mut::<MerchantScreenHandler>()
        {
            merchant.set_selected_offer(selected_offer);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::event_slot_index;

    #[test]
    fn trade_event_index_is_not_truncated() {
        assert_eq!(event_slot_index(0), Some(0));
        assert_eq!(event_slot_index(255), Some(255));
        assert_eq!(event_slot_index(256), None);
        assert_eq!(event_slot_index(-1), None);
    }
}
