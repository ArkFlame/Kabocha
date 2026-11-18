# Parity rows for crates/pumpkin-core/src/net/bedrock/play/player_auth_input.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-ne-967170** [partial/OWN-OPEN] own audit: 4 open, 37 fixed, 5 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-ne-967170.md) docs/tracker/taudit-crates-pumpkin-core-src-ne-967170.md
- **own/taudit-crates-pumpkin-core-src-net-bedroc** [partial/OWN-OPEN] own audit: 17 open, 2 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-net-bedroc.md) docs/tracker/taudit-crates-pumpkin-core-src-net-bedroc.md

## Result (third pass, final)

No row is STILL OPEN. Every row in both referenced audit docs was re-verified against
`.vendor/vanilla-decompiled/net/minecraft/`. All fixes are present in
`crates/pumpkin-core/src/net/bedrock/play/player_auth_input.rs` with pure unit tests;
no source file needed an edit this pass, so there was nothing to rustfmt. No cargo/git
run (forbidden by the unit); the orchestrator compiles once after all workers finish.

Vanilla references confirmed this pass:
- `world/entity/player/Player.java:382-393` `getDesiredPose` returns `SPIN_ATTACK` only
  from `isAutoSpinAttack` (389-390).
- `server/level/ServerPlayer.java:3777-3781` `getLastClientMoveIntent` builds
  `(leftIntent, 0, forwardIntent)`; `world/entity/Entity.java:2471-2481` `getInputVector`
  maps that to world `(x*cos - z*sin, z*cos + x*sin)`, so LEFT -> +X and FORWARD -> +Z at
  yaw 0. `camera_relative_move` sets LEFT for +X and is sign-correct.
- `server/network/ServerGamePacketListenerImpl.java:2947-2953` `START_RIDING_JUMP`
  gates on controlling vehicle, `canJump()` and data > 0.
- `server/network/ServerGamePacketListenerImpl.java:578-613` `handleMoveVehicle`
  checks `getControllingPassenger() == player` (591) and uses `targetYRot`/`targetXRot`
  (606-607), matching `vehicle_id_matches` + `mount_rotation`.
- `server/network/ServerGamePacketListenerImpl.java:857-884`
  `handleAcceptTeleportPacket` correlates by id (859); Bedrock's `HandledTeleport`
  flag / echoed `server_position` substitute for that id, proximity as fallback.

| row | outcome | evidence |
|---|---|---|
| spin attack flags (S) | VERIFIED FIXED | Both Bedrock handlers call `update_player_pose()`, never `set_pose(SpinAttack)` from a client flag (`Player.java:382-393`). |
| camera-relative movement (S) | VERIFIED FIXED | `camera_relative_move` projects `raw_move` with `left_intent`; LEFT set for +X (`ServerPlayer.java:3777-3781`, `Entity.java:2471-2481`). |
| sneak raw flags (S) | VERIFIED FIXED (Bedrock-only) | `sneaking_from_flags` toggle edge + raw fallbacks, edges win; four unit tests. No vanilla/Paper/Geyser source in `.vendor`, so only flag precedence is verified. |
| analog deadzone (S) | VERIFIED FIXED (Bedrock-only) | `ANALOG_DEADZONE` 0.1 in `analog_move`; threshold not from a decompiled source. |
| `await_teleport` (M) | VERIFIED FIXED | `SPlayerAuthInput.teleport` echo + `HandledTeleport` flag + `player_action.rs` clearing on ack; proximity/20-input resend fallback. `handleAcceptTeleportPacket` (857-884), no cross-crate protocol change needed. |
| `InputData::StartJumping` (M) | VERIFIED FIXED | Held `JumpDown` charge capped at `MAX_JUMP_CHARGE_TICKS` (90) swapped into `start_riding_jump` (`ServerGamePacketListenerImpl.java:2947-2953`). |
| `apply_mount_move` (M) | VERIFIED FIXED | `get_controlling_passenger` check, `vehicle_id_matches`, `mount_rotation` from `vehicle_rotation` (x=pitch, y=yaw); tests present (`handleMoveVehicle` 578-613). |
| vehicle branch boats/minecarts (L) | VERIFIED FIXED (MVP) | `apply_mount_move` takes `&Arc<dyn EntityBase>` and moves any controlled vehicle; paddle flags applied; minecarts steer via server-side input. |
| `inventory_action.rs` `ReleaseItem` (adjacent, S) | VERIFIED FIXED | Calls `player.stop_using_item()`, which clears active hand, matching the Java path. |
| `player_action.rs` `// TODO` catch-all (adjacent, M) | VERIFIED FIXED | Teleport acks clear pending state; remaining sneak/item-use/swing actions are documented intentional no-ops (auth-input applies effects each tick; `animate.rs` broadcasts swings). |
| on_ground from zero delta (S) | VERIFIED FIXED (Bedrock-only) | `is_on_ground` keeps stored ground state on zero delta and lands on downward delta; tests present. |
| sprint `SprintDown` fallback (S) | VERIFIED FIXED | Edges win, then held `SprintDown` state (`START/STOP_SPRINTING`, 2935-2940). |
| unclamped stored delta (S) | VERIFIED FIXED | `store_input_state` zeroes non-finite/over-budget deltas via `MAX_CLIENT_STEP_SQUARED`. |

Skipped: no source edits (all rows already fixed), no build/test (orchestrator compiles).
Risk: Bedrock-only field conventions (`camera_orientation`/`raw_move` signs, sneak raw
flags, 0.1 analog deadzone) have no Geyser/protocol fixture or live client in `.vendor`;
only the vanilla Java input mapping was verified mechanically.
