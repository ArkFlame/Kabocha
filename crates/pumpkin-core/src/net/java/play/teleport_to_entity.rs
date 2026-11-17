#[allow(clippy::wildcard_imports)]
use super::*;
use crate::plugin::api::events::player::player_teleport::TeleportCause;

impl JavaClient {
    pub fn handle_teleport_to_entity(
        &self,
        player: &Arc<Player>,
        packet: &STeleportToEntity,
        server: &Server,
    ) {
        if !player.has_client_loaded() {
            return;
        }
        player.update_last_action_time();

        if player.gamemode.load() != GameMode::Spectator {
            return;
        }

        // Vanilla handleTeleportToEntityPacket: the first level holding the UUID wins,
        // the spectator moves there and no camera target is set.
        let Some((target_world, target)) = server.worlds.load().iter().find_map(|world| {
            world
                .get_entity_by_uuid(packet.target)
                .map(|entity| (world.clone(), entity))
                .or_else(|| {
                    world
                        .get_player_by_uuid(packet.target)
                        .map(|target| (world.clone(), target as Arc<dyn EntityBase>))
                })
        }) else {
            return;
        };

        let target_pos = target.get_entity().pos.load();
        let yaw = player.living_entity.entity.yaw.load();
        let pitch = player.living_entity.entity.pitch.load();
        player.teleport_with_cause(
            target_pos,
            Some(yaw),
            Some(pitch),
            target_world,
            TeleportCause::Spectate,
        );
    }
}
