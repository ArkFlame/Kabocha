# Audit: crates/pumpkin-core/src/net/bedrock/play/player_auth_input.rs

Coverage: AUDIT. No TODO/FIXME markers in the file; the two tracker hits are the
`vanilla` mentions in the crawl/glide comments (lines 67 and 135), both documenting
deliberate Bedrock-only behaviour and correct as written.

Fixed in this pass: the parsed `item_interaction` (embedded item use) was dropped, so
eat / block / bow / place from `PlayerAuthInput` never reached the item use code.

## Remaining gaps

| file:line | vanilla / Geyser reference | gap | suggested fix | size |
|---|---|---|---|---|
| player_auth_input.rs:186 `await_teleport` | `ServerGamePacketListenerImpl.handleAcceptTeleportation` / Bedrock `Teleport` ack struct (HANDLED_TELEPORT) | SPlayerAuthInput has no `Teleport` field (transaction id + tick), so confirmation is inferred from position proximity with a 20-tick resend; a client that stops moving near but not on the target loops resends | add the `Teleport` struct to the protocol packet and clear `awaiting_teleport` on transaction id match, keep proximity as fallback | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| player_auth_input.rs:69-79 flags `StartJumping` (31) | `ServerboundPlayerCommandPacket` START_RIDING_JUMP; `Player::start_riding_jump` exists and is only reachable from the Java path (`net/java/play/player_command.rs:25`) | a saddled horse/pig on Bedrock never jumps because the flag is ignored | on `StartJumping` while riding a `Mob` with a saddle, call `player.start_riding_jump(server, scale)` with the charge scale derived from held `JumpDown` ticks | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| player_auth_input.rs:317-357 `apply_mount_move` | `ServerboundMoveVehiclePacket` (vehicle position/rotation/isOnGround) | `packet.vehicle_rotation` and `vehicle_unique_id` are parsed and ignored; the mount is moved from the rider's step and rotated to the rider's yaw, so a boat/camera-relative mount rotation never reaches the server | accept `vehicle_rotation` as the mount yaw when present and verify `vehicle_unique_id` matches the controlled mount before applying the step | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| player_auth_input.rs:36-38 `on_ground` | `ServerboundMovePlayerPacket` explicit `onGround` byte; Bedrock only sends `VerticalCollision` | requires Collision *and* `client_delta.y < 0`, so a stationary tick on the ground (zero delta) reports airborne, which feeds `apply_client_move` fall/ground state | keep the delta check for landing detection but fall back to the previous `on_ground` when the delta is exactly zero | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| player_auth_input.rs:50-56 sprint | `ServerboundPlayerCommandPacket` START/STOP_SPRINTING | only the edge flags are read; `SprintDown`/`Sprinting` state flags are ignored, so a client that reports held sprint without edges keeps vanilla sprint state unsynced | fall back to `SprintDown` when neither edge flag is set | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| player_auth_input.rs:43 `store_input_state` | vanilla `LastKinetic` / knockback recovery | `packet.delta` is stored unvalidated and later layered into entity pushes (`entity/player/client_push.rs:130`) without the `MAX_STEP_SQUARED` clamp used in `apply_mount_move` | clamp the stored delta to the same 100.0 squared-step budget | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| player_auth_input.rs:66-67 start/stop crawling | n/a (Bedrock has no crawl) | correct to ignore, kept as documentation | none | - |
| player_auth_input.rs:diff `MissedSwing` (39) / `StartUsingItem` (53) flags | Geyser `BedrockPlayerAuthInputTranslator` maps both to `ServerboundSwingPacket` / `ServerboundUseItemPacket` | the flags are a duplicate transport for effects already handled by `SAnimate` and the embedded item interaction; only worth mapping if a client is observed without those packets | none for now | - |

## Gap status
- 1 await_teleport has no Teleport ack (player_auth_input.rs:186): STILL OPEN, M.
- 2 StartJumping ignored on saddled mounts (player_auth_input.rs:69-79): STILL OPEN, M.
- 3 vehicle_rotation and vehicle_unique_id ignored (player_auth_input.rs:317-357): STILL OPEN, M.
- 4 on_ground from zero delta (player_auth_input.rs:36-38): STILL OPEN, S.
- 5 sprint from SprintDown fallback (player_auth_input.rs:50-56): STILL OPEN, S.
- 6 unclamped stored delta (player_auth_input.rs:43): STILL OPEN, S.
- Crawl flags (player_auth_input.rs:66-67): no gap, documented.

## Result

Re-audit against decompiled vanilla (ServerGamePacketListenerImpl.handleMoveVehicle,
handlePlayerCommand, ServerboundMoveVehiclePacket, ServerboundMovePlayerPacket).

| row | outcome |
|---|---|
| 1 await_teleport / Teleport ack | STILL OPEN. Vanilla handleAcceptTeleportation clears the pending teleport by id; Bedrock's ack needs a new Teleport value in SPlayerAuthInput (pumpkin-protocol) plus id-based clearing in await_teleport. That is a protocol packet + cross-crate change outside this unit's files, so the proximity/20-input resend fallback stays. |
| 2 StartJumping on saddled mounts | Already implemented in this file: StartJumping swaps the held JumpDown charge (0..90) into start_riding_jump, which gates on controlling passenger, mob and jump_scale > 0, matching vanilla handlePlayerCommand START_RIDING_JUMP. Verified, no further change. |
| 3 vehicle_rotation / vehicle_unique_id | FIXED. apply_mount_move now rejects a present-but-mismatched vehicle_unique_id before applying the step, and snaps the mount to vehicle_rotation (x=pitch, y=yaw) when present, falling back to the rider's yaw; matches vanilla handleMoveVehicle using the packet's targetYRot/targetXRot. Pure tests added for the rotation mapping and the id check. |
| 4 on_ground zero delta | Already implemented via is_on_ground (zero delta keeps the stored ground state) with unit tests. Verified, no further change. |
| 5 sprint SprintDown fallback | Already implemented: StartSprinting/StopSprinting edges first, SprintDown held state when neither edge is set. Verified, no further change. |
| 6 unclamped stored delta | Already implemented: store_input_state zeroes non-finite or over-budget deltas using the shared MAX_CLIENT_STEP_SQUARED. Verified, no further change. |

Touched: crates/pumpkin-core/src/net/bedrock/play/player_auth_input.rs (rustfmt 2024).
1 row resolved in this pass, 1 left open (protocol change outside unit), 4 confirmed already fixed.


## Result (re-audit, current checkout)

Verified the current checkout against decompiled vanilla
`ServerGamePacketListenerImpl` (`handleMoveVehicle` 578-810,
`handlePlayerCommand` 2915-2969, `handleAcceptTeleportPacket` 857-884,
`handleMovePlayer` 1582-1930) and the Bedrock `SPlayerAuthInput` protocol struct.
No source change was needed: every STILL OPEN row is already implemented in
`player_auth_input.rs` and compiles against the current protocol fields
(`SPlayerAuthInput.teleport`, `InputData::HandledTeleport`,
`BedrockClient.teleport_sent_pos`). The older Result section above is stale on
row 1, which is now implemented.

| row | outcome |
|---|---|
| 1 await_teleport / Teleport ack | FIXED (supersedes the stale "STILL OPEN" line below). `await_teleport` now clears the pending teleport on the `HandledTeleport` input flag, on a `PlayerAuthInputTeleportData.server_position` echo of `teleport_sent_pos`, or on proximity, with a 20-input resend fallback. `send_bedrock_teleport` stores `teleport_sent_pos` at the same point the pending teleport is set, so the target is not stale. Pure tests `teleport_ack_flag_confirms_without_reaching_the_target`, `teleport_echoed_server_position_confirms_a_stale_input_position`, `teleport_confirms_by_proximity_without_the_ack_flag` cover all three signals. No transaction-id match is needed: Bedrock has no teleport id in auth input, and vanilla `handleAcceptTeleportPacket` (857) only gates on an id to correlate the ack, which the flag/echo substitute here. |
| 2 StartJumping on saddled mounts | Verified fixed, no change. `StartJumping` swaps the held `JumpDown` charge (clamped 0..90) into `player.start_riding_jump`, which gates on controlling passenger, mount presence, `can_jump` and `jump_scale > 0`, matching vanilla `handlePlayerCommand` START_RIDING_JUMP (2947-2954). |
| 3 vehicle_rotation / vehicle_unique_id | Verified fixed, no change. `apply_mount_move` rejects a present but mismatched `vehicle_unique_id` and snaps the mount to `vehicle_rotation` (x=pitch, y=yaw), falling back to the rider yaw, matching vanilla `handleMoveVehicle` use of `targetYRot`/`targetXRot` (606-607, 704-710). Pure tests `mount_rotation_uses_the_reported_vehicle_yaw_when_present`, `mount_rotation_falls_back_to_the_rider_yaw`, `mount_step_rejects_an_unknown_vehicle_id` present. |
| 4 on_ground zero delta | Verified fixed, no change. `is_on_ground` keeps the stored ground state on a zero delta and lands on a downward delta; pure tests present. |
| 5 sprint SprintDown fallback | Verified fixed, no change. Edge flags win, then `SprintDown` held state, matching vanilla START/STOP_SPRINTING (2935-2940). |
| 6 unclamped stored delta | Verified fixed, no change. `store_input_state` zeroes non-finite or over-budget deltas via `MAX_CLIENT_STEP_SQUARED`, shared with `apply_mount_move`. |

No .rs files changed this pass. 0 newly fixed, 0 left open. rustfmt not needed (no .rs touched).


## Result (verification pass, post-restart)

Re-read the current checkout of `player_auth_input.rs`, `client_push.rs` and
`net/java/play/player_command.rs` and re-checked every STILL OPEN row against
decompiled vanilla `ServerGamePacketListenerImpl`
(`handlePlayerCommand` 2915-2974, `handleAcceptTeleportPacket` 857-886,
`handleMoveVehicle` 578-613). All six rows are already implemented; no source
change was needed this pass. Vanilla facts re-confirmed:
`handlePlayerCommand` START_RIDING_JUMP (2947-2953) gates on a controlled
`PlayerRideableJumping`, `canJump()` and `data > 0`; `handleMoveVehicle` (591,
606-607) gates on `getControllingPassenger() == player` and takes the mount yaw
and pitch from the packet's `targetYRot`/`targetXRot`;
`handleAcceptTeleportPacket` (859) correlates the ack by id, which Bedrock auth
input lacks, so the flag/echo signal substitutes.

| row | outcome |
|---|---|
| 1 await_teleport / Teleport ack | Verified fixed. `await_teleport` clears on `HandledTeleport`, on a `teleport.server_position` echo of `teleport_sent_pos`, or on proximity, with a 20-input resend. Tests present. |
| 2 StartJumping on saddled mounts | Verified fixed. Held `JumpDown` charge (clamped 0..90) is swapped into `start_riding_jump`, which gates on controlling passenger, mount, `can_jump` and `jump_scale > 0`, matching vanilla. |
| 3 vehicle_rotation / vehicle_unique_id | Verified fixed. `apply_mount_move` rejects a present-but-mismatched id and snaps the mount to `vehicle_rotation` (x=pitch, y=yaw) with rider-yaw fallback, matching `targetYRot`/`targetXRot`. Tests present. |
| 4 on_ground zero delta | Verified fixed. `is_on_ground` keeps stored ground state on a zero delta and lands on a downward delta. Tests present. |
| 5 sprint SprintDown fallback | Verified fixed. Edge flags win, then held `SprintDown` state. |
| 6 unclamped stored delta | Verified fixed. `store_input_state` zeroes non-finite / over-budget deltas via `MAX_CLIENT_STEP_SQUARED` before they reach `client_delta` and `client_push.rs`. |

No .rs files changed. 0 newly fixed, 0 left open, 6 confirmed fixed against
vanilla. All pure seams already carry tests; no new test needed. rustfmt not
applicable (no .rs touched).
