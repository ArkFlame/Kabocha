#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub fn handle_set_creative_slot(
        &self,
        player: &Arc<Player>,
        packet: SSetCreativeSlot,
    ) -> Result<(), InventoryError> {
        if player.gamemode.load() != GameMode::Creative {
            return Err(InventoryError::PermissionError);
        }
        let is_negative = packet.slot < 0;
        let valid_slot = packet.slot >= 1 && packet.slot as usize <= 45;
        let item_stack = packet
            .clicked_item
            .to_stack_for_version(&pumpkin_data::packet::CURRENT_MC_VERSION);
        let mut creative_event =
            crate::plugin::api::events::inventory::inventory_creative::InventoryCreativeEvent::new(
                player.clone(),
                packet.slot,
                item_stack.item.registry_key.to_string(),
                item_stack.item_count,
            );
        if let Some(server) = player.world().server.upgrade() {
            server
                .plugin_manager
                .fire_blocking(&server, &mut creative_event);
        }
        if creative_event.cancelled {
            return Ok(());
        }

        let is_legal =
            item_stack.is_empty() || item_stack.item_count <= item_stack.get_max_stack_size();

        if valid_slot && is_legal {
            let mut player_screen_handler = player
                .player_screen_handler
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            let is_armor_equipped = player_screen_handler
                .get_slot(packet.slot as usize)
                .get_stack()
                .are_equal(&item_stack);
            if !is_armor_equipped {
                player.sync_screen_slot_equipment(packet.slot as usize, &item_stack);
            }

            player_screen_handler
                .get_slot(packet.slot as usize)
                .set_stack(item_stack.clone());
            player_screen_handler.set_received_stack(packet.slot as usize, item_stack);
            player_screen_handler.send_content_updates();
            drop(player_screen_handler);
        } else if is_negative && is_legal {
            // Vanilla `dropSpamThrottler`: excess creative drops are ignored.
            let tick = player
                .world()
                .level_time
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .world_age;
            let allowed = self
                .creative_drop_throttle
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .try_drop(tick);
            if allowed {
                player.drop_item(item_stack);
            } else {
                warn!(
                    "Player {} was dropping items too fast in creative mode, ignoring.",
                    player.gameprofile.name
                );
            }
        }
        Ok(())
    }
}

const CREATIVE_DROP_STEP: i64 = 20;
const CREATIVE_DROP_THRESHOLD: i64 = 1480;

/// Vanilla `TickThrottler(20, 1480)`. Each drop adds 20 and the count decays by 1 per tick.
/// The decay is applied lazily from the world age instead of ticking every connection.
#[derive(Default)]
pub struct CreativeDropThrottle {
    count: i64,
    last_tick: i64,
}

impl CreativeDropThrottle {
    /// Decays to `tick`, then records and allows the drop when the count is under the threshold.
    pub fn try_drop(&mut self, tick: i64) -> bool {
        let elapsed = tick.saturating_sub(self.last_tick).max(0);
        self.count = self.count.saturating_sub(elapsed).max(0);
        self.last_tick = tick;
        if self.count < CREATIVE_DROP_THRESHOLD {
            self.count += CREATIVE_DROP_STEP;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CreativeDropThrottle;

    #[test]
    fn burst_drops_stop_at_threshold() {
        let mut throttle = CreativeDropThrottle::default();
        let accepted = (0..80).filter(|_| throttle.try_drop(0)).count();
        assert_eq!(accepted, 74);
    }

    #[test]
    fn count_decays_one_per_tick() {
        let mut throttle = CreativeDropThrottle::default();
        for _ in 0..74 {
            throttle.try_drop(0);
        }
        assert!(!throttle.try_drop(0));
        assert!(throttle.try_drop(1));
        assert!(!throttle.try_drop(1));
    }
}
