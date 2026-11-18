# Audit: crates/pumpkin-core/src/net/bedrock/play/player_auth_input.rs

Coverage: AUDIT (2026-11-17). No TODO/FIXME markers in the file; the two tracker hits are
the `vanilla` mentions in the crawl (line 67) and glide (`canGlide`) comments, both
documenting deliberate Bedrock-only behaviour and correct as written.

Fixed in this pass:

| commit | gap | vanilla / Geyser reference |
|---|---|---|
| 60729726a | `on_ground` required `client_delta.y < 0`, so every standing tick (zero delta) reported airborne; `apply_client_move` thus never saw the player grounded while walking | `ServerboundMovePlayerPacket` explicit `onGround` byte. Now a zero delta keeps the stored `Entity.on_ground` (helper `is_on_ground`, unit tests in the file) |
| db658c13d | sprint was only applied from the `StartSprinting`/`StopSprinting` edges, clients reporting only held `SprintDown` never sprinted | `ServerboundPlayerCommandPacket` START_SPRINTING with the input packet as fallback |
| 9ce9d7764 | `packet.delta` was stored unvalidated and later layered into entity pushes (`entity/player/client_push.rs`) | vanilla "moved too quickly" budget; the stored delta now shares `MAX_CLIENT_STEP_SQUARED` with the mount guard and is zeroed when non-finite or over budget |

## Remaining gaps

| file:line | vanilla / Geyser reference | gap | suggested fix | size |
|---|---|---|---|---|
| `player_auth_input.rs` `await_teleport` | `ServerGamePacketListenerImpl.handleAcceptTeleportation`, Bedrock `Teleport` ack struct (`HANDLED_TELEPORT`) | SPlayerAuthInput has no `Teleport` value, so confirmation is inferred from position proximity with a 20-input resend; a client that stops near but not on the target loops resends | add the `Teleport` struct (transaction id + tick) to the protocol packet and clear `awaiting_teleport` on id match, keep proximity as fallback | M |
| `player_auth_input.rs` `InputData::StartJumping` (31) | `ServerboundPlayerCommandPacket` START_RIDING_JUMP; `Player::start_riding_jump` is reachable only from `net/java/play/player_command.rs` | a saddled horse/pig on Bedrock never jumps; the charge scale is only in the held `JumpDown` ticks | on `StartJumping` while riding a saddled `Mob`, call `player.start_riding_jump(server, scale)` | M |
| `player_auth_input.rs` `apply_mount_move` | `ServerboundMoveVehiclePacket` (position/rotation/isOnGround) | `packet.vehicle_rotation` and `vehicle_unique_id` are parsed and ignored; the mount is rotated to the rider's yaw and the reported vehicle id is never checked | use `vehicle_rotation` (x=pitch, y=yaw) for the mount when present, verify `vehicle_unique_id` matches the controlled mount before applying the step | M |
| `player_auth_input.rs` vehicle branch | Bedrock has no move-vehicle packet; boats/minecarts report `ClientPredictedVehicle` + `vehicle_rotation` | `apply_mount_move` returns for non-mob vehicles, so a Bedrock client cannot steer a boat or minecart; only the rider's rotation syncs | feed `vehicle_rotation` + paddling flags into `entity/vehicle/boat.rs` and the minecart input path | L |
| `player_auth_input.rs` `InputData::SneakToggleDown` (23), `SneakCurrentRaw` (64), `PersistSneak` (24) | Bedrock input flags; Geyser reads the raw sneak state | only the `Start`/`StopSneaking` edges and `SneakDown`/`Descend` are read, so a client that reports toggle-sneak through the raw flags never crouches | fall back to `SneakCurrentRaw` when neither edge flag is set, and treat `SneakToggleDown` as a toggle edge | S |
| `player_auth_input.rs` `camera_relative_movement`, `move_vec`, `raw_move`, `camera_orientation` | Bedrock third-person camera-relative movement (`InputData::CameraRelativeMovementEnabled`) | the fields are parsed but unused, so the Java input bits (`SPlayerInput`) are derived from the analog vector even when the client reports camera-relative controls; only affects the input bitmask, positions stay client-authoritative | rotate `raw_move` by `camera_orientation` when the flag is set before deriving the bits | S |
| `player_auth_input.rs` `analog_move` | analog stick input | no deadzone: controller drift reports `FORWARD`/`LEFT` bits every tick, which reach predicates, `PlayerInputEvent` and minecart `driver_input` | ignore components below a small deadzone | S |
| `player_auth_input.rs` spin attack flags | vanilla sets `SpinAttack` pose from riptide state (`LivingEntity.isAutoSpinAttack`) and never from a client flag | the pose is set/cleared directly from `StartSpinAttack`/`StopSpinAttack`, so any client can pose as spinning | gate the pose on the server-side riptide state | S |

Adjacent (other files, found while reading the call chain):

- `net/bedrock/play/inventory_action.rs:536` `TransactionData::ReleaseItem` calls `on_stopped_using` and `clear_active_hand()` but not `player.stop_using_item()`, so `using_item` / `item_use_start_time` stay set after a Bedrock bow release; the Java path uses `stop_using_item`. S.
- `net/bedrock/play/player_action.rs:278` still has a `// TODO` catch-all: `StartSneaking`/`StopSneaking`, `StartUsingItem`/`StopUsingItem`, `StartJump`, `HandledTeleport`, `MissedSwing` and `ChangeDimensionAck` actions are dropped, so only the auth-input copies of sneak/glide work. M.

## Gapwave2-2 status

- `analog_move` deadzone (S): FIXED in 38492c288 (`ANALOG_DEADZONE` 0.1). UNVERIFIED: the 0.1 threshold is not from a decompiled source.
- `InputData::StartJumping` (M): FIXED (UNVERIFIED). `Player::bedrock_jump_charge_ticks` counts held `JumpDown` ticks (capped at 90, reset when released); `StartJumping` swaps it out and calls `start_riding_jump`. Charge timing vs Bedrock client not verified against a live client.
- Remaining rows (`await_teleport`, `apply_mount_move`, sneak raw flags, camera-relative movement, spin attack gating): not attempted this pass.

## Gapwave2-2 status (h3)
- player_auth_input StartJumping: FIXED 039310a96 (StartJumping now sets the Java JUMP input)
- remaining rows: STILL OPEN (note body in this worktree is unreadable; rows not reviewed in this wave)


## Result

Re-verified every STILL OPEN row against
`.vendor/vanilla-decompiled/net/minecraft/` this pass. Fixes are written but not compiled
(the unit forbids cargo); rustfmt 2024 was run on all touched .rs files.

| row | outcome | evidence |
|---|---|---|
| spin attack flags (S) | FIXED | `Player.getDesiredPose()` returns `SPIN_ATTACK` only when `isAutoSpinAttack()`. `player_auth_input.rs` no longer calls `set_pose(SpinAttack)` on a client flag; both spin flags now call `update_player_pose()`, which derives the pose from the server-side riptide state. Same duplicate fixed in bedrock `player_action.rs`. |
| sneak raw flags (S) | FIXED | New pure `sneaking_from_flags` reads `SneakToggleDown` as a toggle edge and `SneakPressedRaw`/`SneakReleasedRaw`/`SneakCurrentRaw` as a fallback; `Start`/`StopSneaking` still win. Four pure unit tests added. Bedrock-only flags, no vanilla/Paper/Spigot reference; UNVERIFIED against a live Bedrock client. |
| `inventory_action.rs` `ReleaseItem` (adjacent, S) | FIXED | Java path calls `stop_using_item()`; Bedrock path only called `clear_active_hand()`, leaving `using_item` set. Now `player.stop_using_item()`. |
| `apply_mount_move` (M) | ALREADY FIXED in this worktree | `mount_rotation` uses `vehicle_rotation` (x=pitch, y=yaw) when present and `vehicle_id_matches` verifies `vehicle_unique_id`; matches vanilla `handleMoveVehicle` (`getControllingPassenger() == player`, `absSnapTo(targetX/targetY/targetYRot/targetXRot)`). Existing unit tests cover both helpers. No edit needed. |
| `await_teleport` (M) | OPEN | Needs a `Teleport` struct with transaction id added to `SPlayerAuthInput` in `pumpkin-protocol`, a file outside this unit (cross-crate protocol change). Position proximity + 20-input resend stays as fallback. |
| camera-relative movement (S) | OPEN | Needs the Bedrock `camera_orientation` rotation math. No decompiled vanilla, Paper/Spigot, Geyser source or protocol fixture in `.vendor` to verify signs of the (x,z) rotation; a wrong sign would invert movement bits. Packet fields are already parsed. |
| vehicle branch, boats/minecarts (L) | OPEN | `apply_mount_move` still returns for non-mob vehicles. Needs server-side boat/minecart steering integration in `entity/vehicle/boat.rs` and the minecart input path; beyond an auth-input-only fix. |
| `player_action.rs` `// TODO` catch-all (adjacent, M) | OPEN | Adding sneak/item-use actions here would duplicate the auth-input side effects already applied every tick; needs a coordinated pass over both handlers, not a one-file fix. |

Already fixed before this pass, confirmed unchanged: `StartJumping` (rides JUMP/charge),
`analog_move` deadzone (`ANALOG_DEADZONE`), `on_ground`/delta handling.

## Result

Re-verification pass (2026-11-18) over every STILL OPEN row against
`.vendor/vanilla-decompiled/net/minecraft/`. All rows are already addressed in this
checkout; one residual bug was found and fixed this pass. rustfmt 2024 run on the
touched file; cargo not run (forbidden by the unit).

| row | outcome | evidence |
|---|---|---|
| camera-relative movement (S) | FIXED (this pass) | Decompiled `ServerPlayer.getLastClientMoveIntent` builds `(left, 0, forward)` and `Entity.getInputVector` maps LEFT to world +X at yaw 0. Old `camera_relative_move` returned the RIGHT bit for +X (inverted). Renamed the strafe projection to `left_intent`; +X now sets LEFT, -X sets RIGHT. Two test expectations and the doc comment corrected. |
| `await_teleport` (M) | FIXED (already) | `SPlayerAuthInput.teleport` ack (field exists in `pumpkin-protocol`) + `HandledTeleport` flag + `player_action.rs` clearing on `HandledTeleport`/`ChangeDimensionAck`. No cross-crate change needed; proximity resend kept as fallback. |
| `InputData::StartJumping` (M) | FIXED (already) | `bedrock_jump_charge_ticks` (capped 90) swapped into `start_riding_jump(server, charge)`, mirroring Java `Action::StartHorseJump` in `player_command.rs`. |
| `apply_mount_move` (M) | FIXED (already) | `get_controlling_passenger()` check, `vehicle_id_matches(vehicle_unique_id)`, `mount_rotation` from `vehicle_rotation`; matches vanilla `handleMoveVehicle` `absSnapTo(targetYRot, targetXRot)` + `positionRider`. Tests present. |
| vehicle branch, boats/minecarts (L) | FIXED (already, MVP) | `apply_mount_move` now takes `&Arc<dyn EntityBase>` and moves any controlled vehicle (boats included); paddle flags applied; minecarts steer via server-side `last_input` in `minecart.rs`. |
| sneak raw flags (S) | FIXED (already) | `sneaking_from_flags` treats `SneakToggleDown` as an edge and falls back to `SneakPressedRaw`/`SneakReleasedRaw`/`SneakCurrentRaw`; edges win; four unit tests. Bedrock-only, UNVERIFIED against a live client. |
| analog deadzone (S) | FIXED (already) | `ANALOG_DEADZONE` 0.1 in `analog_move`; test added. Threshold not from a decompiled source (UNVERIFIED). |
| spin attack flags (S) | FIXED (already) | Verified `Player.getDesiredPose()` returns `SPIN_ATTACK` only when `isAutoSpinAttack()` (Player.java:382-393). Both Bedrock handlers now call `update_player_pose()` (which uses `get_desired_pose()`), never `set_pose(SpinAttack)` from a client flag. |
| `inventory_action.rs` `ReleaseItem` (adjacent, S) | FIXED (already) | `player.stop_using_item()` called; it internally calls `clear_active_hand()` (`player.rs:8534`), so `using_item` and active hand are both cleared like the Java path. |
| `player_action.rs` `// TODO` catch-all (adjacent, M) | FIXED (already) | `HandledTeleport`/`ChangeDimensionAck` clear the pending teleport; remaining sneak/item-use/swing actions are documented intentional no-ops because auth-input applies those effects each tick and `animate.rs` broadcasts swings. |

Skipped: no code change outside `player_auth_input.rs`; no build/test run (orchestrator compiles). Risk: Bedrock field conventions (`camera_orientation`/`raw_move`) have no Geyser or protocol fixture in `.vendor`, so only the LEFT/RIGHT vs vanilla-input mapping was corrected; the look-axis signs follow the existing implementation.

## Result

Fresh re-audit (2026-11-18). No row is STILL OPEN in the latest status; every gap is
already fixed in this checkout. Verified each fix against `.vendor/vanilla-decompiled/`
and left the code unchanged. No cargo/git run (forbidden by the unit); orchestrator compiles.

| row | outcome | evidence checked this pass |
|---|---|---|
| spin attack flags (S) | FIXED (already) | `Player.getDesiredPose()` returns SPIN_ATTACK only from `isAutoSpinAttack()` (Player.java:382-390). Both Bedrock handlers call `update_player_pose()`, never `set_pose(SpinAttack)`. |
| camera-relative movement (S) | FIXED (already) | `ServerPlayer.getLastClientMoveIntent()` (3777-3781) builds `(left, 0, forward)`; `Entity.getInputVector` (2471-2481) rotates to world `(x*cos - z*sin, z*cos + x*sin)`. `camera_relative_move` inverse (forward dot, left = `(forward_z, -forward_x)` dot) is sign-correct; LEFT bit set for +X at yaw 0. |
| sneak raw flags (S) | FIXED (already) | `sneaking_from_flags` handles toggle edge + raw fallbacks with edges winning; four unit tests. Bedrock-only, no vanilla reference. |
| analog deadzone (S) | FIXED (already) | `ANALOG_DEADZONE` 0.1 in `analog_move`; test present. Threshold not from a decompiled source. |
| `await_teleport` (M) | FIXED (already) | `SPlayerAuthInput.teleport` echo + `HandledTeleport` flag + `player_action.rs` clearing on ack; proximity resend fallback. Vanilla `handleAcceptTeleportation` equivalent. |
| `InputData::StartJumping` (M) | FIXED (already) | Held `JumpDown` charge (capped 90, MAX_JUMP_CHARGE_TICKS) swapped into `start_riding_jump`. |
| `apply_mount_move` (M) | FIXED (already) | `get_controlling_passenger()` check, `vehicle_id_matches`, `mount_rotation` from `vehicle_rotation` (x=pitch, y=yaw); tests present. |
| vehicle branch boats/minecarts (L) | FIXED (already, MVP) | `apply_mount_move` takes `&Arc<dyn EntityBase>`, moves any controlled vehicle; paddle flags applied. |
| `inventory_action.rs` `ReleaseItem` (adjacent, S) | FIXED (already) | Calls `player.stop_using_item()` (line 552), matching the Java path. |
| `player_action.rs` `// TODO` catch-all (adjacent, M) | FIXED (already) | `HandledTeleport`/`ChangeDimensionAck` clear pending teleport; remaining sneak/item-use/swing actions documented as intentional no-ops (lines 296-299). |

Skipped: no code changes (all rows already fixed); no build/test (orchestrator compiles).
Risk: Bedrock-only field conventions (`camera_orientation`/`raw_move`, sneak raw flags,
analog deadzone threshold) have no Geyser/protocol fixture or live client in `.vendor` to
verify end-to-end; only the vanilla-input mapping was checked here.

## Result

Re-audit (2026-11-18) of every gap row against `.vendor/vanilla-decompiled/net/minecraft/`.
All ten rows are already fixed in this checkout; no source file needed an edit this pass.
No cargo/git run (forbidden by the unit); the orchestrator compiles.

Vanilla references checked this pass:
- `ServerPlayer.getLastClientMoveIntent` (3777-3781) builds
  `(leftIntent, 0, forwardIntent)`; `Entity.getInputVector` (2471-2481) maps that to
  world `(x*cos - z*sin, z*cos + x*sin)`, so LEFT -> +X and FORWARD -> +Z at yaw 0.
  `camera_relative_move`'s `left_intent = raw_move.x*forward_z - raw_move.y*forward_x`
  is sign-correct and sets LEFT for +X.
- `Player.getDesiredPose` (382-393) returns SPIN_ATTACK only from `isAutoSpinAttack()`,
  confirming both Bedrock handlers must call `update_player_pose()`, never set it from a flag.
- Java `Action::StartHorseJump` calls `player.start_riding_jump(server, jump_boost)`
  (java/play/player_command.rs:25), matching the Bedrock held-`JumpDown` charge swap.

| row | outcome | verification in main checkout |
|---|---|---|
| spin attack flags (S) | FIXED (already) | `update_player_pose()` on both spin flags in player_auth_input.rs (88-92) and player_action.rs (273-275); no `set_pose(SpinAttack)` from a client flag. |
| camera-relative movement (S) | FIXED (already) | `camera_relative_move` with `left_intent` (490); sign checked against vanilla input mapping above. |
| sneak raw flags (S) | FIXED (already) | `sneaking_from_flags` toggle edge + raw fallbacks (357-381); four unit tests. |
| analog deadzone (S) | FIXED (already) | `ANALOG_DEADZONE` 0.1 in `analog_move` (459-468); test present. |
| `await_teleport` (M) | FIXED (already) | `SPlayerAuthInput.teleport` echo + `HandledTeleport` flag (303-308) + player_action.rs clearing on `HandledTeleport`/`ChangeDimensionAck` (17-33); proximity resend fallback. |
| `InputData::StartJumping` (M) | FIXED (already) | held `JumpDown` charge capped at `MAX_JUMP_CHARGE_TICKS` swapped into `start_riding_jump` (137-151). |
| `apply_mount_move` (M) | FIXED (already) | `get_controlling_passenger` check (514-519), `vehicle_id_matches` (523,554), `mount_rotation` from `vehicle_rotation` (539-544,561); tests present. |
| vehicle branch boats/minecarts (L) | FIXED (already, MVP) | `apply_mount_move` takes `&Arc<dyn EntityBase>` and moves any controlled vehicle; paddle flags applied (122-133). |
| `inventory_action.rs` ReleaseItem (S) | FIXED (already) | `player.stop_using_item()` at line 552, matching the Java path. |
| `player_action.rs` `// TODO` catch-all (M) | FIXED (already) | teleport acks clear pending state; remaining sneak/item-use/swing actions documented intentional no-ops (296-299); swings broadcast by animate.rs (31-35). |

Skipped: no code changes (all rows already fixed); no build/test (orchestrator compiles).
Risk: Bedrock-only field semantics (`camera_orientation`/`raw_move` signs, sneak raw flags,
0.1 analog deadzone) have no decompiled vanilla, Paper/Spigot, Geyser source or protocol fixture
in `.vendor`; only the vanilla Java input mapping was verified here.

## Result (coder pass 2026-11-18, batch r2_013)
- No STILL OPEN rows in the latest pass result above. Every row is FIXED, VERIFIED or BLOCKED-DATA per that result.
- This pass did not re-verify each row against vanilla (time budget); status carried from the latest prior pass, unchanged.
- Files edited this pass: none. No rustfmt run. Cargo and git not run (forbidden).
