#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub fn handle_player_command(
        &self,
        player: &Arc<Player>,
        command: &SPlayerCommand,
        server: &Arc<Server>,
    ) {
        if command.entity_id != player.entity_id().into() {
            return;
        }
        if !player.has_client_loaded() {
            return;
        }
        player.update_last_action_time();

        let entity = &player.get_entity();
        match command.action {
            Action::StartSprinting => player.apply_sprint_input(server, true),
            Action::StopSprinting => player.apply_sprint_input(server, false),
            Action::LeaveBed => player.wake_up(),

            Action::StartHorseJump => player.start_riding_jump(server, command.jump_boost.0),
            // Releasing the jump key does nothing on any rideable mob.
            Action::StopHorseJump => {}
            Action::OpenVehicleInventory => {
                player.open_vehicle_inventory();
            }
            Action::StartFlyingElytra => {
                // A failed start is also the client asking to stop gliding; the stop event
                // veto applies here like every other stop site.
                if !player.try_to_start_fall_flying(server)
                    && entity.is_fall_flying()
                    && player.glide_stop_allowed()
                {
                    entity.stop_fall_flying();
                }
            }
            // <= 1.21.5
            Action::StartSneaking | Action::StopSneaking => {
                self.handle_player_input(
                    player,
                    &SPlayerInput {
                        input: SPlayerInput::SNEAK,
                    },
                    server,
                );
            }
        }
    }
}
