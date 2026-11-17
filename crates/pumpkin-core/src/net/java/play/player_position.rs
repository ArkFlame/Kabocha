#[allow(clippy::wildcard_imports)]
use super::*;
use crate::entity::player::ClientMove;
use pumpkin_util::math::vector3::Vector3;

fn position_is_finite(position: &Vector3<f64>) -> bool {
    position.x.is_finite() && position.y.is_finite() && position.z.is_finite()
}

fn rotation_is_finite(yaw: f32, pitch: f32) -> bool {
    yaw.is_finite() && pitch.is_finite()
}

#[cfg(test)]
mod tests {
    use super::{position_is_finite, rotation_is_finite};
    use pumpkin_util::math::vector3::Vector3;

    #[test]
    fn rejects_non_finite_movement() {
        assert!(position_is_finite(&Vector3::new(1.0, 64.0, -3.5)));
        assert!(!position_is_finite(&Vector3::new(f64::NAN, 64.0, 0.0)));
        assert!(!position_is_finite(&Vector3::new(0.0, f64::INFINITY, 0.0)));
        assert!(rotation_is_finite(90.0, -45.0));
        assert!(!rotation_is_finite(f32::NAN, 0.0));
        assert!(!rotation_is_finite(0.0, f32::NEG_INFINITY));
    }
}

impl JavaClient {
    fn kick_invalid_player_movement(&self) {
        self.try_kick(&TextComponent::translate_cross(
            translation::java::MULTIPLAYER_DISCONNECT_INVALID_PLAYER_MOVEMENT,
            translation::java::MULTIPLAYER_DISCONNECT_INVALID_PLAYER_MOVEMENT,
            [],
        ));
    }

    pub fn handle_position(
        &self,
        player: &Arc<Player>,
        server: &Arc<Server>,
        packet: &SPlayerPosition,
    ) {
        if !player.has_client_loaded() {
            return;
        }
        // A movement packet was received this tick — tracked for SClientTickEnd zeroing.
        if !position_is_finite(&packet.position) {
            self.kick_invalid_player_movement();
            return;
        }
        self.received_movement_this_tick
            .store(true, Ordering::Relaxed);
        player.apply_client_move(
            server,
            ClientMove {
                position: Some(packet.position),
                yaw: None,
                pitch: None,
                head_yaw: None,
                on_ground: packet.collision & FLAG_ON_GROUND != 0,
                horizontal_collision: packet.collision & FLAG_HORIZONTAL_COLLISION != 0,
            },
        );
    }

    pub fn handle_position_rotation(
        &self,
        player: &Arc<Player>,
        server: &Arc<Server>,
        packet: &SPlayerPositionRotation,
    ) {
        if !player.has_client_loaded() {
            return;
        }
        if !position_is_finite(&packet.position) || !rotation_is_finite(packet.yaw, packet.pitch) {
            self.kick_invalid_player_movement();
            return;
        }
        self.received_movement_this_tick
            .store(true, Ordering::Relaxed);
        player.apply_client_move(
            server,
            ClientMove {
                position: Some(packet.position),
                yaw: Some(packet.yaw),
                pitch: Some(packet.pitch),
                head_yaw: Some(packet.yaw),
                on_ground: packet.collision & FLAG_ON_GROUND != 0,
                horizontal_collision: packet.collision & FLAG_HORIZONTAL_COLLISION != 0,
            },
        );
    }
}
