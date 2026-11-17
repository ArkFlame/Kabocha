#[allow(clippy::wildcard_imports)]
use super::*;
use crate::entity::vehicle::boat::BoatEntity;

impl JavaClient {
    pub fn handle_paddle_boat(&self, player: &Arc<Player>, packet: &SPaddleBoat) {
        let entity = player.get_entity();
        // Vanilla getControlledVehicle: the immediate vehicle, and only when the
        // player is its controlling passenger (the first rider for a boat).
        let Some(vehicle) = entity.get_vehicle() else {
            return;
        };
        let is_controlling = vehicle
            .get_controlling_passenger()
            .is_some_and(|passenger| passenger.get_entity().entity_id == entity.entity_id);
        if !is_controlling {
            return;
        }
        let Some(boat) = vehicle.cast_any().downcast_ref::<BoatEntity>() else {
            return;
        };
        boat.set_paddle_state(packet.left_paddle, packet.right_paddle);
    }
}
