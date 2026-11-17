use pumpkin_data::entity::EntityPose;
use pumpkin_protocol::PositionFlag;
use pumpkin_util::math::vector3::Vector3;

use super::BoatEntity;
use crate::entity::{EntityBase, player::Player};

// Avatar.DEFAULT_VEHICLE_ATTACHMENT is shared by standing and crouching players.
const DEFAULT_VEHICLE_ATTACHMENT: Vector3<f64> = Vector3::new(0.0, 0.6, 0.0);

impl BoatEntity {
    pub(in crate::entity) fn synchronize_mount(&self, player: &Player) -> bool {
        let Some(position) = self.rider_position(player) else {
            return false;
        };
        if !player.request_teleport_with_flags(
            position,
            0.0,
            0.0,
            vec![PositionFlag::YRot, PositionFlag::XRot],
        ) {
            return false;
        }
        player.get_entity().velocity.store(Vector3::default());
        true
    }

    fn rider_position(&self, passenger: &dyn EntityBase) -> Option<Vector3<f64>> {
        passenger.get_player()?;
        let attachment = self.get_passenger_attachment_point(passenger)?;
        let vehicle_attachment = match passenger.get_entity().pose.load() {
            EntityPose::Standing | EntityPose::Crouching => DEFAULT_VEHICLE_ATTACHMENT,
            _ => Vector3::default(),
        };
        Some(self.vehicle.entity.pos.load() + attachment - vehicle_attachment)
    }

    fn get_passenger_attachment_point(&self, passenger: &dyn EntityBase) -> Option<Vector3<f64>> {
        let entity = &self.vehicle.entity;
        let passengers = entity
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let index = passengers
            .iter()
            .position(|rider| rider.get_entity().entity_id == passenger.get_entity().entity_id)?;
        let passenger_count = passengers.len();
        let is_animal = passenger
            .get_mob()
            .and_then(|mob| mob.as_animal())
            .is_some();
        drop(passengers);
        super::passenger_attachment(entity, index, passenger_count, is_animal)
    }
}
