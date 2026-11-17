# Audit: crates/pumpkin-core/src/net/bedrock/play/player_auth_input.rs

Coverage: AUDIT. No TODO/FIXME markers in the file; the two tracker hits are the
`vanilla` mentions in the crawl/glide comments (lines 67 and 135), both documenting
deliberate Bedrock-only behaviour and correct as written.

Fixed in this pass: the parsed `item_interaction` (embedded item use) was dropped, so
eat / block / bow / place from `PlayerAuthInput` never reached the item use code.

## Remaining gaps

| file:line | vanilla / Geyser reference | gap | suggested fix | size |
|---|---|---|---|---|
| player_auth_input.rs:186 `await_teleport` | `ServerGamePacketListenerImpl.handleAcceptTeleportation` / Bedrock `Teleport` ack struct (HANDLED_TELEPORT) | SPlayerAuthInput has no `Teleport` field (transaction id + tick), so confirmation is inferred from position proximity with a 20-tick resend; a client that stops moving near but not on the target loops resends | add the `Teleport` struct to the protocol packet and clear `awaiting_teleport` on transaction id match, keep proximity as fallback | M |
| player_auth_input.rs:69-79 flags `StartJumping` (31) | `ServerboundPlayerCommandPacket` START_RIDING_JUMP; `Player::start_riding_jump` exists and is only reachable from the Java path (`net/java/play/player_command.rs:25`) | a saddled horse/pig on Bedrock never jumps because the flag is ignored | on `StartJumping` while riding a `Mob` with a saddle, call `player.start_riding_jump(server, scale)` with the charge scale derived from held `JumpDown` ticks | M |
| player_auth_input.rs:317-357 `apply_mount_move` | `ServerboundMoveVehiclePacket` (vehicle position/rotation/isOnGround) | `packet.vehicle_rotation` and `vehicle_unique_id` are parsed and ignored; the mount is moved from the rider's step and rotated to the rider's yaw, so a boat/camera-relative mount rotation never reaches the server | accept `vehicle_rotation` as the mount yaw when present and verify `vehicle_unique_id` matches the controlled mount before applying the step | M |
| player_auth_input.rs:36-38 `on_ground` | `ServerboundMovePlayerPacket` explicit `onGround` byte; Bedrock only sends `VerticalCollision` | requires Collision *and* `client_delta.y < 0`, so a stationary tick on the ground (zero delta) reports airborne, which feeds `apply_client_move` fall/ground state | keep the delta check for landing detection but fall back to the previous `on_ground` when the delta is exactly zero | S |
| player_auth_input.rs:50-56 sprint | `ServerboundPlayerCommandPacket` START/STOP_SPRINTING | only the edge flags are read; `SprintDown`/`Sprinting` state flags are ignored, so a client that reports held sprint without edges keeps vanilla sprint state unsynced | fall back to `SprintDown` when neither edge flag is set | S |
| player_auth_input.rs:43 `store_input_state` | vanilla `LastKinetic` / knockback recovery | `packet.delta` is stored unvalidated and later layered into entity pushes (`entity/player/client_push.rs:130`) without the `MAX_STEP_SQUARED` clamp used in `apply_mount_move` | clamp the stored delta to the same 100.0 squared-step budget | S |
| player_auth_input.rs:66-67 start/stop crawling | n/a (Bedrock has no crawl) | correct to ignore, kept as documentation | none | - |
| player_auth_input.rs:diff `MissedSwing` (39) / `StartUsingItem` (53) flags | Geyser `BedrockPlayerAuthInputTranslator` maps both to `ServerboundSwingPacket` / `ServerboundUseItemPacket` | the flags are a duplicate transport for effects already handled by `SAnimate` and the embedded item interaction; only worth mapping if a client is observed without those packets | none for now | - |
