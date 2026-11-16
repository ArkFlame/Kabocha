use pumpkin_protocol::java::client::play::CMoveVehicle;

#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub fn handle_move_vehicle(&self, player: &Arc<Player>, packet: &SMoveVehicle) {
        if player.is_movement_locked() {
            player.reject_client_move();
            return;
        }
        // A movement packet was received this tick — tracked for SClientTickEnd zeroing.
        self.received_movement_this_tick
            .store(true, Ordering::Relaxed);

        if !packet.x.is_finite()
            || !packet.y.is_finite()
            || !packet.z.is_finite()
            || !packet.yaw.is_finite()
            || !packet.pitch.is_finite()
        {
            self.try_kick(&TextComponent::translate_cross(
                translation::java::MULTIPLAYER_DISCONNECT_INVALID_PLAYER_MOVEMENT,
                translation::java::MULTIPLAYER_DISCONNECT_INVALID_PLAYER_MOVEMENT,
                [],
            ));
            return;
        }

        let entity = player.get_entity();
        let last_pos = entity.pos.load();
        let pos = Vector3::new(
            crate::net::clamp_horizontal_position(packet.x),
            crate::net::clamp_vertical_position(packet.y),
            crate::net::clamp_horizontal_position(packet.z),
        );
        let vehicle = entity.get_vehicle();
        if let Some(vehicle) = &vehicle {
            let vehicle_entity = vehicle.get_entity();
            // Only the controlling passenger (the first one) steers.
            let is_controlling = vehicle_entity
                .passengers
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .first()
                .is_some_and(|passenger| passenger.get_entity().entity_id == entity.entity_id);
            if !is_controlling {
                return;
            }

            // Vanilla's "moved too quickly" check, net of the vehicle's own motion. The
            // vehicle is sent back so the client doesn't stay out of sync.
            let vehicle_pos = vehicle_entity.pos.load();
            let expected = vehicle_entity.velocity.load().length_squared();
            if vehicle_pos.squared_distance_to_vec(&pos) - expected > 100.0 {
                warn!(
                    "{} moved a vehicle too quickly ({vehicle_pos:?} -> {pos:?})",
                    player.gameprofile.name
                );
                self.try_send_packet(&CMoveVehicle::new(
                    vehicle_pos.x,
                    vehicle_pos.y,
                    vehicle_pos.z,
                    vehicle_entity.yaw.load(),
                    vehicle_entity.pitch.load(),
                ));
                return;
            }

            let old_pos = vehicle_entity.pos.load();
            let old_y = old_pos.y;
            vehicle_entity.set_pos(pos);
            vehicle_entity.set_rotation(packet.yaw, packet.pitch);
            let pos = if let Some(vehicle_move) = vehicle.get_vehicle_entity() {
                let move_result = vehicle_move.move_vehicle(old_pos, pos);
                vehicle_entity.set_pos(move_result.position);
                if move_result.cancelled {
                    vehicle_entity.velocity.store(Vector3::new(0.0, 0.0, 0.0));
                    vehicle_entity.send_velocity();
                }
                if move_result.position != pos {
                    let final_pos = move_result.position;
                    self.try_send_packet(&CMoveVehicle::new(
                        final_pos.x,
                        final_pos.y,
                        final_pos.z,
                        packet.yaw,
                        packet.pitch,
                    ));
                }
                move_result.position
            } else {
                pos
            };
            vehicle_entity
                .on_ground
                .store(packet.on_ground, Ordering::Relaxed);
            vehicle_entity.emit_client_movement(vehicle.as_ref(), pos - old_pos);
            if let Some(living) = vehicle.get_living_entity() {
                living.fall(vehicle.as_ref(), pos.y - old_y, packet.on_ground);
            }

            let world = player.world();
            let vehicle_pos = pos.to_block_pos();
            let check_below = BlockPos::new(vehicle_pos.0.x, vehicle_pos.0.y - 1, vehicle_pos.0.z);
            let in_lava = world.get_block(&vehicle_pos) == &pumpkin_data::Block::LAVA
                || world.get_block(&check_below) == &pumpkin_data::Block::LAVA;

            if in_lava {
                let start_pos = player.entered_lava_pos.load().unwrap_or(pos);
                if player.entered_lava_pos.load().is_none() {
                    player.entered_lava_pos.store(Some(pos));
                }
                let dx = pos.x - start_pos.x;
                let dz = pos.z - start_pos.z;
                let distance_horizontal = dx.hypot(dz);
                player.trigger_advancement(
                    crate::entity::player::advancement::trigger::AdvancementTrigger::RideEntityInLava {
                        distance_horizontal,
                        dimension: world.dimension.minecraft_name.to_string(),
                        vehicle_type: Some(format!(
                            "minecraft:{}",
                            vehicle_entity.entity_type.resource_name
                        )),
                    },
                );
            } else {
                player.entered_lava_pos.store(None);
            }
        } else {
            player.entered_lava_pos.store(None);
        }
        match &vehicle {
            Some(vehicle) => crate::entity::position_rider(vehicle.as_ref(), player.as_ref()),
            None => entity.set_pos(pos),
        }
        let new_pos = entity.pos.load();
        let distance = last_pos.squared_distance_to_vec(&new_pos).sqrt();
        let cm = (distance * 100.0).round() as i32;
        if cm > 0 {
            let stat = player.get_movement_statistic();
            player.increment_stat(
                pumpkin_data::statistic::StatisticCategory::Custom,
                stat as i32,
                cm,
            );
        }
        chunker::update_position(player);
    }
}
