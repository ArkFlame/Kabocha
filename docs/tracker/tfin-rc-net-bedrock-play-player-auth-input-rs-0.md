# Final rows from docs/tracker/tpar-rates-pumpkin-core-src-net-bedrock-play-player-auth-input-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

No row is STILL OPEN. Every row in both referenced audit docs was re-verified against

## Result (final pass)

Every row is closed. All 13 rows across the two referenced audit docs were re-verified
this pass against decompiled vanilla in `.vendor/vanilla-decompiled/net/minecraft/` and
against the Rust in `crates/pumpkin-core/src/net/bedrock/play/`. Every fix and its pure
unit test is already present in `player_auth_input.rs`, `inventory_action.rs` and
`player_action.rs`. No source file needed an edit this pass, so there was nothing to
rustfmt. No generated `pumpkin-data` field, WIT change or missing subsystem is required,
so no row is BLOCKED-DATA. No cargo/git run (forbidden by the unit); the orchestrator
compiles once after all workers finish.

Vanilla files checked this pass (line numbers confirmed):
- `world/entity/player/Player.java:382-390` `getDesiredPose` returns `SPIN_ATTACK` only
  from `isAutoSpinAttack` (389-390).
- `server/level/ServerPlayer.java:3777-3780` `getLastClientMoveIntent` builds
  `(leftIntent, 0, forwardIntent)`; `world/entity/Entity.java:2471-2481` `getInputVector`
  maps LEFT to world +X at yaw 0. `camera_relative_move` sets LEFT for +X, sign-correct.
- `server/network/ServerGamePacketListenerImpl.java:2947-2950` `START_RIDING_JUMP`
  gates on a controlled jumping vehicle, `canJump()` and `data > 0`.
- `ServerGamePacketListenerImpl.java:591,606-607,704-710` `handleMoveVehicle` gates on
  `getControllingPassenger() == player` and snaps the mount to `targetYRot`/`targetXRot`,
  matching `vehicle_id_matches` + `mount_rotation` (vehicle_rotation x=pitch, y=yaw).
- `ServerGamePacketListenerImpl.java:857` `handleAcceptTeleportPacket` correlates the ack
  by id; the `HandledTeleport` flag / echoed `server_position` substitute, proximity
  and a 20-input resend are the fallback.
- `ServerGamePacketListenerImpl.java:2921-2922,2935-2938` `START/STOP_SPRINTING`.

| row | outcome | evidence |
|---|---|---|
| spin attack flags (S) | VERIFIED FIXED | Both Bedrock handlers call `update_player_pose()`, never `set_pose(SpinAttack)` from a client flag (`Player.java:382-390`). |
| camera-relative movement (S) | VERIFIED FIXED | `camera_relative_move` projects `raw_move` with LEFT set for +X at yaw 0 (`ServerPlayer.java:3777-3780`, `Entity.java:2471-2481`); two tests. |
| sneak raw flags (S) | FIXED (Bedrock-only) | `sneaking_from_flags` toggle edge + raw fallbacks, edges win; four unit tests. No vanilla/Geyser source for the Bedrock flags; only precedence verified. |
| analog deadzone (S) | FIXED (Bedrock-only) | `ANALOG_DEADZONE` 0.1 in `analog_move`; threshold not from a decompiled source; test present. |
| `await_teleport` (M) | VERIFIED FIXED | `SPlayerAuthInput.teleport` echo + `HandledTeleport` flag + `player_action.rs` clearing on ack; proximity/20-input resend fallback. `handleAcceptTeleportPacket` (857) id correlation substituted by flag/echo; three tests. |
| `InputData::StartJumping` (M) | VERIFIED FIXED | Held `JumpDown` charge capped at `MAX_JUMP_CHARGE_TICKS` (90) swapped into `start_riding_jump` (`ServerGamePacketListenerImpl.java:2947-2950`). |
| `apply_mount_move` (M) | VERIFIED FIXED | `get_controlling_passenger` check, `vehicle_id_matches`, `mount_rotation` from `vehicle_rotation` (x=pitch, y=yaw); three tests (`handleMoveVehicle` 591,606-607,704-710). |
| vehicle branch boats/minecarts (L) | FIXED (MVP) | `apply_mount_move` takes `&Arc<dyn EntityBase>` and moves any controlled vehicle; paddle flags applied; minecarts steer via server-side input. |
| `inventory_action.rs` `ReleaseItem` (adjacent, S) | VERIFIED FIXED | Calls `player.stop_using_item()` (line 552), clearing active hand and `using_item`, matching the Java path. |
| `player_action.rs` `// TODO` catch-all (adjacent, M) | VERIFIED FIXED | `HandledTeleport`/`ChangeDimensionAck` clear pending state (17-33); remaining sneak/item-use/swing actions documented intentional no-ops (auth-input applies effects each tick; `animate.rs` broadcasts swings). |
| on_ground from zero delta (S) | FIXED (Bedrock-only) | `is_on_ground` keeps stored ground state on zero delta, lands on downward delta; three tests. |
| sprint `SprintDown` fallback (S) | VERIFIED FIXED | Edges win, then held `SprintDown` state (`START/STOP_SPRINTING` 2921-2922,2935-2938). |
| unclamped stored delta (S) | VERIFIED FIXED | `store_input_state` zeroes non-finite/over-budget deltas via `MAX_CLIENT_STEP_SQUARED`, shared with `apply_mount_move`. |

Skipped: no source edits (all rows already fixed), no build/test (orchestrator compiles).
Risk: Bedrock-only field conventions (`camera_orientation`/`raw_move` signs, sneak raw
flags, 0.1 analog deadzone) have no Geyser/protocol fixture or live client in `.vendor`;
only the vanilla Java input mapping was verified mechanically.
