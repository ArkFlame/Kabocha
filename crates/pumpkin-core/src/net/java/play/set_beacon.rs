#[allow(clippy::wildcard_imports)]
use super::*;
use pumpkin_data::data_component_impl::IDSetContent;
use pumpkin_data::effect::StatusEffect;
use pumpkin_inventory::beacon_screen_handler::BeaconScreenHandler;
use pumpkin_protocol::java::server::play::SSetBeacon;

impl JavaClient {
    /// Mirrors `ServerGamePacketListenerImpl.handleSetBeaconPacket`.
    pub fn handle_set_beacon(&self, player: &Arc<Player>, packet: &SSetBeacon) {
        // `None` = no effect; an id that is not a status effect is a malformed packet.
        let decode = |id: &Option<VarInt>| {
            id.as_ref().map_or(Ok(None), |id| {
                u16::try_from(id.0)
                    .ok()
                    .and_then(<StatusEffect as IDSetContent>::from_id)
                    .map(Some)
                    .ok_or(())
            })
        };
        let (primary, secondary) = match (
            decode(&packet.primary_effect),
            decode(&packet.secondary_effect),
        ) {
            (Ok(primary), Ok(secondary)) => (primary, secondary),
            _ => {
                warn!(
                    "Player {} sent undecodable beacon effects: primary {:?}, secondary {:?}",
                    player.gameprofile.name, packet.primary_effect, packet.secondary_effect
                );
                self.try_kick(&TextComponent::translate(
                    "multiplayer.disconnect.generic",
                    &[],
                ));
                return;
            }
        };

        // Vanilla `BeaconMenu.updateEffects` silently ignores selections the pyramid cannot grant.
        let consumed_item = {
            let handler_lock = player
                .current_screen_handler
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone();
            let mut handler = handler_lock
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let Some(beacon) = handler.as_any_mut().downcast_mut::<BeaconScreenHandler>() else {
                debug!(
                    "Player {} interacted with invalid menu (expected beacon)",
                    player.gameprofile.name
                );
                return;
            };
            let payment = beacon.payment_item_id();
            if beacon.update_effects(primary, secondary) {
                payment
            } else {
                debug!(
                    "Player {} selected beacon effects the pyramid cannot grant",
                    player.gameprofile.name
                );
                None
            }
        };

        if let Some(item_id) = consumed_item {
            // Vanilla `BeaconMenu.updateEffects` awards `Stats.ITEM_USED` for the payment item.
            player.increment_stat(pumpkin_data::statistic::StatisticCategory::Used, item_id, 1);
        }
    }
}
