#[allow(clippy::wildcard_imports)]
use super::*;
use pumpkin_protocol::bedrock::server::player_auth_input::{InputMode, InteractionModel};
use pumpkin_protocol::java::server::play::SPlayerInput;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::player::ClientMove;

/// A client cannot legitimately move more than this squared distance in one input tick, which
/// is vanilla's "moved too quickly" budget. Mount steps and the delta pushes layer on share it.
const MAX_CLIENT_STEP_SQUARED: f64 = 100.0;

/// Vanilla caps the horse jump charge at 90 ticks of held jump.
const MAX_JUMP_CHARGE_TICKS: i32 = 90;

impl BedrockClient {
    #[expect(
        clippy::too_many_lines,
        reason = "auth input validation runs as one ordered pass"
    )]
    pub fn handle_player_auth_input(
        &self,
        player: &Arc<Player>,
        packet: SPlayerAuthInput,
        server: &Arc<Server>,
    ) {
        if player.living_entity.dead.load(Ordering::Relaxed)
            || player.living_entity.health.load() <= 0.0
        {
            return;
        }
        if self.await_teleport(player, &packet) {
            // Stale poses before a respawn teleport must not count as client motion.
            player.get_entity().movement.store(Vector3::default());
            return;
        }
        if !player.has_client_loaded() {
            return;
        }

        let entity = player.get_entity();
        let flags = &packet.input_data;
        // Like Geyser: the motion this tick started with was downward. Landing can already
        // have cleared the end delta (gliding into the ground).
        let on_ground = is_on_ground(
            flags.get(InputData::VerticalCollision as usize),
            self.client_delta.load().y,
            entity.on_ground.load(Ordering::Relaxed),
            entity.has_vehicle(),
        );
        let horizontal_collision = flags.get(InputData::HorizontalCollision as usize);

        self.store_input_state(&packet);

        let sneaking = sneaking_from_auth(player, flags);
        let was_sneaking = entity.is_sneaking();
        player.apply_client_input(server, java_input_from_auth(&packet, sneaking));
        if entity.is_sneaking() != was_sneaking {
            player.update_bedrock_shield_sneaking();
        }

        let started_sprint = flags.get(InputData::StartSprinting as usize);
        let stopped_sprint = flags.get(InputData::StopSprinting as usize);
        let held_sprint = flags.get(InputData::SprintDown as usize);
        if started_sprint && !stopped_sprint {
            player.apply_sprint_input(server, true);
        } else if stopped_sprint && !started_sprint {
            player.apply_sprint_input(server, false);
        } else if !started_sprint && !stopped_sprint && held_sprint {
            // Clients that report only the held sprint state, without the edges, still sprint.
            player.apply_sprint_input(server, true);
        }

        if flags.get(InputData::StartSwimming as usize) {
            entity.set_swimming(true);
            player.update_player_pose();
        } else if flags.get(InputData::StopSwimming as usize) {
            entity.set_swimming(false);
            player.update_player_pose();
        }

        // Bedrock has no crawl pose: the client sends StartCrawling when it squeezes
        // into a one block gap, which vanilla never turns into a crawl.

        // Vanilla derives SPIN_ATTACK from the server-side riptide state
        // (Player.getDesiredPose -> isAutoSpinAttack), never from a client flag.
        if flags.get(InputData::StartSpinAttack as usize)
            || flags.get(InputData::StopSpinAttack as usize)
        {
            player.update_player_pose();
        }

        if flags.get(InputData::StartFlying as usize) {
            player.apply_flight_input(server, true);
        } else if flags.get(InputData::StopFlying as usize) {
            player.apply_flight_input(server, false);
        }

        // Bedrock can stop in the same input it started, then it never glided.
        if flags.get(InputData::StartGliding as usize)
            && !flags.get(InputData::StopGliding as usize)
        {
            Self::start_gliding(player, server);
        } else if flags.get(InputData::StopGliding as usize) {
            Self::stop_gliding(player, on_ground);
        }

        let new_pos = player.feet_from_bedrock_pos(packet.position.to_f64());
        player.apply_client_move(
            server,
            ClientMove {
                position: Some(new_pos),
                yaw: Some(packet.yaw),
                pitch: Some(packet.pitch),
                head_yaw: Some(packet.head_yaw),
                on_ground,
                horizontal_collision,
            },
        );

        if let Some(vehicle) = entity.get_vehicle() {
            vehicle.set_paddle_state(
                flags.get(InputData::PaddlingLeft as usize),
                flags.get(InputData::PaddlingRight as usize),
            );
            apply_mount_move(
                player,
                &vehicle,
                &packet,
                flags.get(InputData::VerticalCollision as usize),
            );
        }

        // Bedrock has no jump charge packet: the charge is the held `JumpDown` ticks, and
        // `StartJumping` releases it as a rider jump (Java sends it as START_RIDING_JUMP).
        if flags.get(InputData::StartJumping as usize) {
            let charge = player
                .bedrock_jump_charge_ticks
                .swap(0, Ordering::Relaxed)
                .clamp(0, MAX_JUMP_CHARGE_TICKS);
            player.start_riding_jump(server, charge);
        } else if flags.get(InputData::JumpDown as usize) {
            let charge = player.bedrock_jump_charge_ticks.load(Ordering::Relaxed);
            player.bedrock_jump_charge_ticks.store(
                charge.saturating_add(1).min(MAX_JUMP_CHARGE_TICKS),
                Ordering::Relaxed,
            );
        } else {
            player.bedrock_jump_charge_ticks.store(0, Ordering::Relaxed);
        }

        // Item use (eat, block, place, bow) is embedded in the input tick since 1.19.80
        // instead of arriving as a standalone `InventoryTransaction`.
        if let Some(interaction) = packet.item_interaction {
            self.handle_embedded_use_item(player, interaction);
        }

        // Inventory moves can ride inside the input packet, like the standalone request
        if let Some(request) = packet.item_stack_request {
            self.handle_item_stack_request(
                player,
                pumpkin_protocol::bedrock::server::item_stack_request::SItemStackRequest {
                    requests: vec![request],
                },
            );
        }

        if let Some(block_actions) = packet.block_actions {
            for action in &block_actions {
                self.handle_player_block_action(player, server, action);
            }
        }
    }
}

/// Bedrock has no explicit `onGround` byte: it reports `VerticalCollision` plus the delta the
/// tick started with. A landing tick zeroes the end delta, so a downward previous delta is the
/// landing signal; a zero delta is also what a standing client reports, which keeps the stored
/// ground state instead of calling every standing tick airborne.
fn is_on_ground(
    vertical_collision: bool,
    previous_delta_y: f64,
    was_on_ground: bool,
    has_vehicle: bool,
) -> bool {
    !has_vehicle
        && vertical_collision
        && (previous_delta_y < 0.0 || (previous_delta_y == 0.0 && was_on_ground))
}

impl BedrockClient {
    /// Routes the item use embedded in a `PlayerAuthInput` tick through the same handler as a
    /// standalone `InventoryTransaction`, which is the only place item use is implemented.
    fn handle_embedded_use_item(
        &self,
        player: &Arc<Player>,
        interaction: pumpkin_protocol::bedrock::server::player_auth_input::PlayerInventoryAction,
    ) {
        use pumpkin_protocol::bedrock::server::inventory_transaction::{
            HandSlot, UseItemTransactionData,
        };

        let data = interaction.transaction;
        self.handle_inventory_action(
            player,
            SInventoryTransaction {
                legacy_request_id: interaction.legacy_request_id,
                legacy_set_item_slots: interaction.legacy_slots,
                has_value: !interaction.actions.is_empty(),
                actions: interaction.actions,
                transaction_type: VarUInt(2),
                transaction_data: TransactionData::UseItem(UseItemTransactionData {
                    action_type: data.action_type,
                    trigger_type: data.trigger_type,
                    block_position: data.block_position,
                    block_face: data.block_face,
                    hot_bar_slot: data.hot_bar_slot,
                    // Auth input item use is always the main hand.
                    hand: HandSlot::Mainhand,
                    item_in_hand: data.item_in_hand,
                    player_position: data.player_position,
                    click_position: data.click_position,
                    block_runtime_id: data.block_runtime_id,
                    client_prediction: data.client_prediction,
                    client_cooldown_state: data.client_cooldown_state,
                }),
            },
        );
    }

    /// The Bedrock client glides straight out of creative flight, which vanilla `canGlide`
    /// forbids, so flight ends first and comes back if the glide is refused.
    pub(super) fn start_gliding(player: &Arc<Player>, server: &Arc<Server>) {
        if player.get_entity().is_fall_flying() {
            return;
        }
        let was_flying = player.is_flying();
        let set_flying = |flying: bool| {
            player
                .abilities
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .flying = flying;
        };
        if was_flying {
            set_flying(false);
        }
        if !player.try_to_start_fall_flying(server) {
            if was_flying {
                set_flying(true);
            }
            player.get_entity().stop_fall_flying();
        }
        if was_flying {
            player.send_abilities_update();
        }
    }

    /// Java cannot cancel elytra in mid-air, landing / `canGlide` already stops it.
    /// A rejected stop resends GLIDING true, as `StopGliding` expects an answer either way.
    pub(super) fn stop_gliding(player: &Arc<Player>, on_ground: bool) {
        let entity = player.get_entity();
        if !entity.is_fall_flying() {
            return;
        }
        let stop = (on_ground || !player.can_glide()) && player.glide_stop_allowed();
        entity.set_fall_flying(!stop);
    }

    /// Client state at this input, which push motions build on.
    fn store_input_state(&self, packet: &SPlayerAuthInput) {
        self.input_tick.store(packet.tick.0, Ordering::Relaxed);
        // Client predicted end-of-tick velocity. Knockback is an absolute `SetActorMotion`.
        // later entity pushes layer on this delta once that motion is no longer in flight.
        // A forged or stale teleport step must not become a push.
        let delta = packet.delta.to_f64();
        let plausible = delta.x.is_finite()
            && delta.y.is_finite()
            && delta.z.is_finite()
            && delta.length_squared() <= MAX_CLIENT_STEP_SQUARED;
        self.client_delta
            .store(if plausible { delta } else { Vector3::default() });
    }

    /// Inputs are ignored until the client reaches a pending teleport, so stale
    /// positions (e.g. the death spot before a respawn) never count as movement or a fall.
    /// Returns true while still waiting.
    fn await_teleport(&self, player: &Arc<Player>, packet: &SPlayerAuthInput) -> bool {
        const RESEND_INPUTS: u32 = 20;

        let mut awaiting = player
            .awaiting_teleport
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if awaiting.is_none() {
            return false;
        }
        // Bedrock acknowledges a teleport with the `HandledTeleport` input flag (Java uses
        // `ServerboundAcceptTeleportationPacket`). A `TeleportData` ack repeats the server
        // position we sent, which correlates it to this pending teleport the way vanilla
        // matches by id; reaching the target is the fallback for clients that send neither.
        let handled = packet.input_data.get(InputData::HandledTeleport as usize);
        let sent = self.teleport_sent_pos.load();
        let echoed = packet
            .teleport
            .is_some_and(|data| position_within(data.server_position, sent));
        if teleport_confirmed(handled, echoed, packet.position, sent) {
            *awaiting = None;
            drop(awaiting);
            player.is_changing_dimension.store(false, Ordering::Relaxed);
            // Bedrock sends no `PlayerLoaded` after a respawn, reaching the spawn stands in
            player.set_client_loaded(true);
        } else if self
            .teleport_unconfirmed_inputs
            .fetch_add(1, Ordering::Relaxed)
            + 1
            >= RESEND_INPUTS
        {
            drop(awaiting);
            player.send_bedrock_teleport(self);
        }
        true
    }
}

/// A Bedrock client confirms a teleport by setting the `HandledTeleport` input flag or by
/// echoing the server position in `TeleportData`, the equivalent of vanilla
/// `handleAcceptTeleportPacket`. Reaching the sent target is the fallback for clients that
/// send neither.
fn teleport_confirmed(
    handled: bool,
    echoed: bool,
    position: Vector3<f32>,
    target: Vector3<f32>,
) -> bool {
    handled || echoed || position_within(position, target)
}

/// f32 positions can sit either side of the stored target, hence the small tolerance.
fn position_within(position: Vector3<f32>, target: Vector3<f32>) -> bool {
    const TELEPORT_ERROR: f32 = 0.1;
    (position.x - target.x).abs() < TELEPORT_ERROR
        && (position.y - target.y).abs() < TELEPORT_ERROR
        && (position.z - target.z).abs() < TELEPORT_ERROR
}

fn sneaking_from_auth(
    player: &Player,
    flags: &pumpkin_protocol::codec::bitset::Bitset<66>,
) -> bool {
    sneaking_from_flags(player.get_entity().is_sneaking(), player.is_flying(), flags)
}

/// Some Bedrock clients only report toggle or raw sneak state and never the `Start`/`Stop`
/// edges. Those flags drive crouching too, while the edges still win when they are present.
fn sneaking_from_flags(
    was_sneaking: bool,
    flying: bool,
    flags: &pumpkin_protocol::codec::bitset::Bitset<66>,
) -> bool {
    if flying {
        return flags.get(InputData::Descend as usize) || flags.get(InputData::SneakDown as usize);
    }
    // Stop wins when both edges arrive on the same tick.
    if flags.get(InputData::StopSneaking as usize) {
        false
    } else if flags.get(InputData::StartSneaking as usize) {
        true
    } else if flags.get(InputData::SneakToggleDown as usize) {
        !was_sneaking
    } else if flags.get(InputData::SneakPressedRaw as usize) {
        true
    } else if flags.get(InputData::SneakReleasedRaw as usize) {
        false
    } else if flags.get(InputData::SneakCurrentRaw as usize) {
        true
    } else {
        was_sneaking
    }
}

fn java_input_from_auth(packet: &SPlayerAuthInput, sneaking: bool) -> i8 {
    let flags = &packet.input_data;
    let mouse = packet.input_mode.0 == InputMode::Mouse as u32;
    let classic_touch = packet.input_mode.0 == InputMode::Touch as u32
        && packet.interaction_model.0 == InteractionModel::Classic as i32;
    // Camera-relative movement reports the stick as a world-space raw_move plus the camera
    // look direction; project it back onto the view axes for the Java (view-relative) bits.
    let camera_relative = flags
        .get(InputData::CameraRelativeMovementEnabled as usize)
        .then(|| camera_relative_move(packet.camera_orientation, packet.raw_move))
        .filter(|move_bits| *move_bits != (false, false, false, false));
    let (up, down, left, right) = match camera_relative {
        Some(move_bits) => move_bits,
        None if mouse || classic_touch => digital_move(flags, classic_touch),
        None => analog_move(packet.analog_move),
    };

    let mut input = 0i8;
    if up {
        input |= SPlayerInput::FORWARD;
    }
    if down {
        input |= SPlayerInput::BACKWARD;
    }
    if left {
        input |= SPlayerInput::LEFT;
    }
    if right {
        input |= SPlayerInput::RIGHT;
    }
    if flags.get(InputData::JumpCurrentRaw as usize)
        || flags.get(InputData::JumpDown as usize)
        || flags.get(InputData::StartJumping as usize)
        || flags.get(InputData::AutoJumpingInWater as usize)
    {
        input |= SPlayerInput::JUMP;
    }
    if sneaking {
        input |= SPlayerInput::SNEAK;
    }
    if flags.get(InputData::SprintDown as usize) {
        input |= SPlayerInput::SPRINT;
    }
    input
}

fn digital_move(
    flags: &pumpkin_protocol::codec::bitset::Bitset<66>,
    classic_touch: bool,
) -> (bool, bool, bool, bool) {
    let mut up = flags.get(InputData::Up as usize);
    let mut down = flags.get(InputData::Down as usize);
    let mut left = flags.get(InputData::Left as usize);
    let mut right = flags.get(InputData::Right as usize);
    if classic_touch {
        if flags.get(InputData::UpLeft as usize) {
            up = true;
            left = true;
        }
        if flags.get(InputData::UpRight as usize) {
            up = true;
            right = true;
        }
        if flags.get(InputData::DownLeft as usize) {
            down = true;
            left = true;
        }
        if flags.get(InputData::DownRight as usize) {
            down = true;
            right = true;
        }
    }
    (up, down, left, right)
}

/// Analog stick components at or below this magnitude are controller drift, not input.
const ANALOG_DEADZONE: f32 = 0.1;

fn analog_move(analog: Vector2<f32>) -> (bool, bool, bool, bool) {
    (
        analog.y > ANALOG_DEADZONE,
        analog.y < -ANALOG_DEADZONE,
        analog.x > ANALOG_DEADZONE,
        analog.x < -ANALOG_DEADZONE,
    )
}

/// Projects Bedrock's camera-relative `raw_move` back onto the Java view-relative input axes.
///
/// Vanilla maps a pressed LEFT to +X and FORWARD to +Z at yaw 0
/// (`ServerPlayer.getLastClientMoveIntent` builds `(left, 0, forward)`). For a camera look
/// vector (x, _, z) the horizontal forward axis is (x, z) and the left-intent axis is (z, -x).
/// A zero look vector yields no movement bits.
fn camera_relative_move(
    camera_orientation: Vector3<f32>,
    raw_move: Vector2<f32>,
) -> (bool, bool, bool, bool) {
    let look_length = camera_orientation.x.hypot(camera_orientation.z);
    if look_length <= f32::EPSILON {
        return (false, false, false, false);
    }
    let (forward_x, forward_z) = (
        camera_orientation.x / look_length,
        camera_orientation.z / look_length,
    );
    let forward = raw_move.x.mul_add(forward_x, raw_move.y * forward_z);
    // Vanilla strafe intent is +1 for LEFT, -1 for RIGHT; at yaw 0 that axis is +X.
    let left_intent = raw_move.y.mul_add(-forward_x, raw_move.x * forward_z);
    (
        forward > ANALOG_DEADZONE,
        forward < -ANALOG_DEADZONE,
        left_intent > ANALOG_DEADZONE,
        left_intent < -ANALOG_DEADZONE,
    )
}

/// Bedrock has no move-vehicle packet: the client simulates the mount it rides and reports
/// only the rider's step, so the server has to carry that step onto the mount itself.
/// Without it a Bedrock-controlled saddled mount or boat never leaves its mounted spot.
/// Vanilla's `handleMoveVehicle` applies the client step to any vehicle whose controlling
/// passenger is the player, not just living mounts, so boats are carried here too.
fn apply_mount_move(
    player: &Arc<Player>,
    vehicle: &Arc<dyn EntityBase>,
    packet: &SPlayerAuthInput,
    on_ground: bool,
) {
    let rider = player.get_entity();
    // Vanilla `Entity.getControllingPassenger`: a saddle-steering first passenger or a boat's
    // first player passenger moves the vehicle; minecarts report none and steer through their
    // server-side input bits instead (see `minecart.rs`).
    let controls = vehicle
        .get_controlling_passenger()
        .is_some_and(|controller| controller.get_entity().entity_id == rider.entity_id);
    if !controls {
        return;
    }
    let vehicle_entity = vehicle.get_entity();
    // The reported runtime id must name the mount being controlled; a stale id means the
    // step belongs to a previous vehicle. Bedrock sends the runtime id as the entity id.
    if !vehicle_id_matches(packet.vehicle_unique_id, vehicle_entity.entity_id) {
        return;
    }
    let delta = packet.delta.to_f64();
    let step = delta.length_squared();
    let moved = delta.x.is_finite()
        && delta.y.is_finite()
        && delta.z.is_finite()
        && step > 0.0
        && step <= MAX_CLIENT_STEP_SQUARED;
    if moved {
        vehicle_entity.set_pos(vehicle_entity.pos.load().add_raw(delta.x, delta.y, delta.z));
        vehicle_entity.emit_client_movement(vehicle.as_ref(), delta);
    }
    // Vanilla `handleMoveVehicle` snaps the mount to the packet's `targetYRot`/`targetXRot`,
    // not the rider's body yaw. Bedrock reports that as `vehicle_rotation` (x=pitch, y=yaw).
    let (yaw, pitch) = mount_rotation(
        packet.vehicle_rotation,
        packet.yaw,
        vehicle_entity.pitch.load(),
    );
    vehicle_entity.set_rotation(yaw, pitch);
    vehicle_entity.on_ground.store(on_ground, Ordering::Relaxed);
    crate::entity::position_rider(vehicle.as_ref(), player.as_ref());
    if moved {
        chunker::update_position(player);
    }
}

/// Bedrock reports the controlled mount's runtime id as a `VarLong` of its entity id.
/// An absent id is fine; a present one that does not name this mount rejects the step.
fn vehicle_id_matches(reported: Option<VarLong>, entity_id: i32) -> bool {
    reported.is_none_or(|id| i32::try_from(id.0).is_ok_and(|id| id == entity_id))
}

/// Vanilla `handleMoveVehicle` takes the mount yaw and pitch from the movement packet.
/// Bedrock's `vehicle_rotation` is (x=pitch, y=yaw); clients that omit it leave the rider's
/// yaw as the only steering signal.
const fn mount_rotation(
    vehicle_rotation: Option<Vector2<f32>>,
    rider_yaw: f32,
    current_pitch: f32,
) -> (f32, f32) {
    match vehicle_rotation {
        Some(rotation) => (rotation.y, rotation.x),
        None => (rider_yaw, current_pitch),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Vector2, Vector3, analog_move, camera_relative_move, is_on_ground, mount_rotation,
        sneaking_from_flags, teleport_confirmed, vehicle_id_matches,
    };
    use pumpkin_protocol::bedrock::server::player_auth_input::InputData;
    use pumpkin_protocol::codec::bitset::Bitset;

    #[test]
    fn analog_drift_inside_deadzone_is_ignored() {
        assert_eq!(
            analog_move(Vector2::new(0.05, -0.05)),
            (false, false, false, false)
        );
        // Returns (y>d, y<-d, x>d, x<-d), i.e. (up, down, left, right) as wired in the caller.
        assert_eq!(
            analog_move(Vector2::new(0.5, -0.5)),
            (false, true, true, false)
        );
    }

    #[test]
    fn camera_relative_move_projects_world_move_onto_view_axes() {
        // Camera facing +Z (yaw 0): forward is +Z, left is +X (vanilla getLastClientMoveIntent).
        let facing_z = Vector3::new(0.0, 0.0, 1.0);
        assert_eq!(
            camera_relative_move(facing_z, Vector2::new(0.0, 1.0)),
            (true, false, false, false)
        );
        assert_eq!(
            camera_relative_move(facing_z, Vector2::new(1.0, 0.0)),
            (false, false, true, false)
        );
        assert_eq!(
            camera_relative_move(facing_z, Vector2::new(0.0, -1.0)),
            (false, true, false, false)
        );
        assert_eq!(
            camera_relative_move(facing_z, Vector2::new(-1.0, 0.0)),
            (false, false, false, true)
        );
        // Camera facing -X (yaw 90): forward is -X, left is +Z.
        let facing_neg_x = Vector3::new(-1.0, 0.0, 0.0);
        assert_eq!(
            camera_relative_move(facing_neg_x, Vector2::new(-1.0, 0.0)),
            (true, false, false, false)
        );
        assert_eq!(
            camera_relative_move(facing_neg_x, Vector2::new(0.0, 1.0)),
            (false, false, true, false)
        );
    }

    #[test]
    fn camera_relative_move_needs_a_look_direction() {
        assert_eq!(
            camera_relative_move(Vector3::new(0.0, 1.0, 0.0), Vector2::new(1.0, 1.0)),
            (false, false, false, false)
        );
    }
    #[test]
    fn landing_tick_uses_the_downward_delta() {
        assert!(is_on_ground(true, -0.5, false, false));
    }

    #[test]
    fn standing_tick_keeps_the_stored_ground_state() {
        assert!(is_on_ground(true, 0.0, true, false));
        assert!(!is_on_ground(true, 0.0, false, false));
    }

    #[test]
    fn rising_or_mounted_ticks_are_airborne() {
        assert!(!is_on_ground(true, 0.5, true, false));
        assert!(!is_on_ground(true, -0.5, true, true));
        assert!(!is_on_ground(false, -0.5, true, false));
    }

    #[test]
    fn mount_rotation_uses_the_reported_vehicle_yaw_when_present() {
        // vehicle_rotation is (x=pitch, y=yaw): vanilla snaps the mount to yRot/xRot.
        assert_eq!(
            mount_rotation(Some(Vector2::new(12.0, 90.0)), 5.0, 3.0),
            (90.0, 12.0)
        );
    }

    #[test]
    fn mount_rotation_falls_back_to_the_rider_yaw() {
        assert_eq!(mount_rotation(None, 5.0, 3.0), (5.0, 3.0));
    }

    #[test]
    fn mount_step_rejects_an_unknown_vehicle_id() {
        use pumpkin_protocol::codec::var_long::VarLong;
        assert!(vehicle_id_matches(None, 7));
        assert!(vehicle_id_matches(Some(VarLong::new(7)), 7));
        assert!(!vehicle_id_matches(Some(VarLong::new(8)), 7));
        assert!(!vehicle_id_matches(
            Some(VarLong::new(i64::from(i32::MAX) + 1)),
            7
        ));
    }

    #[test]
    fn teleport_ack_flag_confirms_without_reaching_the_target() {
        assert!(teleport_confirmed(
            true,
            false,
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(100.0, 100.0, 100.0)
        ));
    }

    #[test]
    fn teleport_confirms_by_proximity_without_the_ack_flag() {
        assert!(teleport_confirmed(
            false,
            false,
            Vector3::new(10.0, 20.0, 30.0),
            Vector3::new(10.05, 19.95, 30.0)
        ));
        assert!(!teleport_confirmed(
            false,
            false,
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(100.0, 100.0, 100.0)
        ));
    }

    #[test]
    fn teleport_echoed_server_position_confirms_a_stale_input_position() {
        assert!(teleport_confirmed(
            false,
            true,
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(100.0, 100.0, 100.0)
        ));
    }

    #[test]
    fn sneak_toggle_down_flips_the_stored_state() {
        let mut flags = Bitset::<66>::default();
        flags.set(InputData::SneakToggleDown as usize, true);
        assert!(sneaking_from_flags(false, false, &flags));
        assert!(!sneaking_from_flags(true, false, &flags));
    }

    #[test]
    fn sneak_raw_flags_drive_the_state_without_edges() {
        let mut held = Bitset::<66>::default();
        held.set(InputData::SneakCurrentRaw as usize, true);
        assert!(sneaking_from_flags(false, false, &held));

        let mut pressed = Bitset::<66>::default();
        pressed.set(InputData::SneakPressedRaw as usize, true);
        assert!(sneaking_from_flags(false, false, &pressed));

        let mut released = Bitset::<66>::default();
        released.set(InputData::SneakReleasedRaw as usize, true);
        assert!(!sneaking_from_flags(true, false, &released));
    }

    #[test]
    fn sneak_edges_win_over_raw_state() {
        let mut flags = Bitset::<66>::default();
        flags.set(InputData::SneakCurrentRaw as usize, true);
        flags.set(InputData::StopSneaking as usize, true);
        assert!(!sneaking_from_flags(true, false, &flags));

        let mut flags = Bitset::<66>::default();
        flags.set(InputData::SneakReleasedRaw as usize, true);
        flags.set(InputData::StartSneaking as usize, true);
        assert!(sneaking_from_flags(false, false, &flags));
    }

    #[test]
    fn flying_sneak_still_descends() {
        let mut flags = Bitset::<66>::default();
        flags.set(InputData::Descend as usize, true);
        assert!(sneaking_from_flags(false, true, &flags));
        assert!(!sneaking_from_flags(false, true, &Bitset::<66>::default()));
    }
}
