# Tracker: entities / Oak Boat (RE-REVIEW-UNTRACKED)

Re-review date: 2026-11-17. Branch `kab/tentities-oak-boat-5c8be7`.
Prior review: `.pool-1794893548750269673/docs/tracker/tentities-oak-boat.md` and
`docs/tracker/tentities-oak-boat.md` (line numbers there are stale: boat logic was split into
`entity/vehicle/boat/{physics,passengers,dismount,block_effects}.rs`). No decompiled vanilla sources
exist in `.vendor/` in this worktree, so each row names the vanilla member it must match and "
verify" marks items that need the decompiled method before changing code.

## Source map (current)

- `crates/pumpkin-core/src/entity/vehicle/boat.rs` (451 lines): entity state, tick, damage, interact,
  seat attachment, NBT (chest inventory), `BoatEntity::new` chest detection.
- `crates/pumpkin-core/src/entity/vehicle/boat/physics.rs`: `get_status` :44, `is_underwater` :62,
  `check_in_water` :89, `get_water_level_above` :114, `get_ground_friction` :137, `float_boat` :185,
  `move_boat` :245, `move_from_client` :267.
- `crates/pumpkin-core/src/entity/vehicle/boat/block_effects.rs`: `Entity.checkInsideBlocks` sweep
  (`record_movement` :24, `apply_effects_from_blocks` :49, 16-step / 100-movement caps, differential
  tests at :412+).
- `crates/pumpkin-core/src/entity/vehicle/boat/{passengers,dismount}.rs`: rider positioning and the
  vanilla dismount search (fallback, horizontal escape vector, standing/crouching/swimming poses).
- `crates/pumpkin-core/src/item/items/boat.rs`: item -> entity (:35), entity -> pick result (:53),
  id list (:85), placement raycast and space checks (vanilla `BoatItem.use`).
- `crates/pumpkin-core/src/net/java/play/{move_vehicle,paddle_boat}.rs`: vehicle packets.

## Oak wiring (re-verified, correct)

- `generated/entity_type.rs:7595` `OAK_BOAT`: size 1.375 x 0.5625, eye height 0.5625,
  `BoatSeats { ride_height: 0.1875, single_passenger_x_offset: 0.0 }`.
- Constructor dispatch `entity/type.rs:334-355` covers every boat/raft id; `OAK_BOAT` maps through it.
- `boat.rs:41-53` `is_chest_boat` excludes `OAK_BOAT`, so it gets two seats and no inventory.
- Item round trip `item/items/boat.rs:35/53/85`; pick result goes through `entity_to_item`.
- Prior fixes are all still present: controlling-passenger check `boat.rs:182`, no eye-in-water
  boarding gate `boat.rs:375`, creative break drops nothing `boat.rs:325`, chest contents scattered on
  break, container-open game event, two-passenger chest boats, client vertical delta tracking.

## Remaining gaps

| # | file:line | vanilla | gap / suggested fix | size |
|---|---|---|---|---|
| 1 | `boat.rs:286` `tick`, `physics.rs:185` `float_boat`, `generated/tracked_data.rs` `ID_BUBBLE_TIME` | `AbstractBoat.tick`/`floatBoat` bubble-column branch, `DATA_ID_BUBBLE_TIME` | No boat-specific bubble-column handling: `ID_BUBBLE_TIME` is generated but never written, and the only effect is the generic `bubble_column::push_entity` velocity write, which `float_boat` then multiplies by 0.9 and adds gravity to. Boats do not rise/fall at vanilla speed and never show the bubble wobble. Track `is_above_bubble_column`/`bubble_column_direction_is_down` plus a bubble timer; apply 0.06 (0.1 at surface) acceleration with 0.7/1.8 caps and -0.03/-0.3 downward before `float_boat`; send `ID_BUBBLE_TIME` while the timer runs. | M |
| 2 | `passengers.rs:34` `rider_position`, `boat.rs:201` `passenger_attachment_point` | `AbstractBoat.clampRotation` | No rider-rotation clamp: vanilla clamps a mob rider's yaw to +/-105 deg around the boat yaw and syncs head yaw. Players stay client-authoritative, so only non-player riders need the hook. Add a vehicle rider-rotation hook and the boat override. | M |
| 3 | `physics.rs:185-243` `float_boat` air -> water branch | `AbstractBoat.floatBoat` transition | Vanilla snaps the boat to the surface Y before the collision test; Rust returns a zero offset and never snaps when the target box is blocked, so a boat embedded in blocks does not pop to the water level. *verify* ordering in the decompiled method, then return the computed offset unconditionally and gate only the velocity/status writes. | S (verify) |
| 4 | `physics.rs:218` `vertical_speed = -self.get_gravity()` | `AbstractBoat.floatBoat` gravity constant | `get_gravity()` honours `no_gravity`, vanilla applies the constant in `floatBoat` regardless (verify). Confirm no boat can carry `NoGravity`, otherwise use `DEFAULT_GRAVITY`. | S (verify) |
| 5 | `net/java/play/move_vehicle.rs:73` `set_rotation(packet.yaw, packet.pitch)` | `ServerGamePacketListenerImpl.handleMoveVehicle` | Vanilla wraps the packet yaw/pitch with `Mth.wrapDegrees` before `absMoveTo`; the Rust handler stores raw values (the player-move path already wraps, `client_move.rs:431`). Wrap both here (and in `correct_client_position` send path only if needed). *verify* against the decompiled handler. | S (verify) |
| 6 | `net/java/play/paddle_boat.rs:6-10` | `ServerGamePacketListenerImpl.handlePaddleBoat` | Handler reads `player.get_entity().get_vehicle()` and forwards to any vehicle; vanilla reads the **root** vehicle and only acts when it is an `AbstractBoat`. Also no check that the sender is the controlling passenger. Route through `get_root_vehicle()` and boat-downcast. | S |
| 7 | `physics.rs:267` `move_from_client` | n/a (dead code) | The boat-specific MoveVehicle path is no longer called from anywhere (`grep` finds only the definition); the generic handler in `net/java/play/move_vehicle.rs` supersedes it. Delete it, or wire it back if `has_new_boat_collision` is meant to guard vehicle moves (vanilla server handler does not do that check, so deletion is the parity option). | S |
| 8 | `server/builtin_game_tests/` | runtime parity | No boat gametest coverage (only the dispenser boat test), so the rewritten water physics, 60-tick underwater ejection and chest-boat drop are unverified at runtime. Add one gametest covering float-up after a fall, ejection after 60 underwater ticks, and 27-slot scatter on break. | S |

## This pass

No code change: every candidate fix above is either M-sized or needs the decompiled
`AbstractBoat` (`floatBoat` ordering/gravity, `handleMoveVehicle` wrapping) to satisfy WORKER.md
rule 12; the worktree has no `.vendor/` reference to check against.

## Gap status
- 1 bubble time publish (boat.rs:286, physics.rs:185): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 2 rider position clamp (passengers.rs:34): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 3 float_boat air-to-water transition (physics.rs:185-243): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 4 floatBoat gravity constant (physics.rs:218): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 5 MoveVehicle rotation (move_vehicle.rs:73): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 6 paddle boat handler (paddle_boat.rs:6-10): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 7 dead move_from_client path (physics.rs:267): STILL OPEN (dead-code cleanup, size S, physics excluded)
- 8 no boat gametest coverage: STILL OPEN (needs gametest, not in scope)


## Result

Verified against decompiled vanilla at .vendor/vanilla-decompiled (symlink to
/home/linsaftw/.claude/jobs/9615f3b2/tmp/rel-mc-backup), specifically
net/minecraft/server/network/ServerGamePacketListenerImpl.java (handleMoveVehicle :578,
handlePaddleBoat :2464) and net/minecraft/world/entity/Entity.java (getControlledVehicle :6661).

- Row 1 (bubble time publish, boat.rs/physics.rs/generated tracked_data): LEFT OPEN -
  file outside this unit plus needs generated tracked-data write; one-line reason: requires
  boat.rs, physics.rs and generated data edits outside this unit.
- Row 2 (rider position clamp, passengers.rs/boat.rs): LEFT OPEN - files outside this
  unit (boat.rs already carries an AbstractBoat.clampRotation hook at on_rider_positioned,
  but that file is not editable here).
- Row 3 (float_boat air-to-water snapping, physics.rs): LEFT OPEN - physics.rs is
  outside this unit.
- Row 4 (floatBoat gravity constant, physics.rs): LEFT OPEN - physics.rs is outside
  this unit.
- Row 5 (MoveVehicle rotation wrap, move_vehicle.rs): VERIFIED FIXED, no change needed.
  Vanilla handleMoveVehicle uses targetYRot = Mth.wrapDegrees(movingTo.yRot()) and
  targetXRot = Mth.wrapDegrees(movingTo.xRot()) and passes them to vehicle.absSnapTo.
  Pumpkin already wraps at move_vehicle.rs:43-44 into target_yaw/target_pitch and applies
  them via set_rotation at :81; client_move.rs:418 etc. also wrap. No edit required.
- Row 6 (paddle boat handler, paddle_boat.rs): FIXED. Vanilla handlePaddleBoat:
  if (player.getControlledVehicle() instanceof AbstractBoat boat)
  boat.setPaddleState(packet.getLeft(), packet.getRight()).
  getControlledVehicle returns the immediate vehicle only when the player is its
  controlling passenger. Rewrote handle_paddle_boat to: require the vehicle be its
  controlling passenger (is_controlling via get_controlling_passenger entity-id match),
  downcast with cast_any to BoatEntity, then set_paddle_state. This stops arbitrary
  vehicles receiving paddle state and stops non-controlling passengers paddling.
  Note: the doc suggested get_root_vehicle; decompiled vanilla uses getControlledVehicle
  (immediate vehicle), so the implementation matches vanilla, not that suggestion.
- Row 7 (dead move_from_client path, physics.rs): LEFT OPEN - physics.rs is outside
  this unit.
- Row 8 (boat gametest coverage, server/builtin_game_tests): LEFT OPEN - gametest
  files are outside this unit.

No pure unit-test seam exists for these packet handlers (they require a live Player,
world and vehicle graph), so none was added. rustfmt --edition 2024 run on the one
touched Rust file (paddle_boat.rs).


## Result (re-review 2026-11-18)

Re-verified in the main checkout against decompiled vanilla at
/home/linsaftw/.claude/jobs/9615f3b2/tmp/rel-mc-backup (ServerGamePacketListenerImpl.java
handleMoveVehicle :578 / handlePaddleBoat :2464, Entity.getControlledVehicle :6661).

- Row 1 (bubble time publish): STILL OPEN - needs boat.rs, physics.rs and generated tracked_data
  edits, all outside this unit.
- Row 2 (rider position clamp): STILL OPEN - needs passengers.rs/boat.rs edits, outside this unit.
- Row 3 (float_boat air-to-water snapping): STILL OPEN - physics.rs outside this unit.
- Row 4 (floatBoat gravity constant): STILL OPEN - physics.rs outside this unit.
- Row 5 (MoveVehicle rotation wrap, move_vehicle.rs): ALREADY FIXED / VERIFIED - vanilla wraps with
  Mth.wrapDegrees (:606-607); Pumpkin wraps at move_vehicle.rs:43-44 and applies at :81. No change.
- Row 6 (paddle boat handler, paddle_boat.rs): ALREADY FIXED / VERIFIED - vanilla uses
  getControlledVehicle() instanceof AbstractBoat (:2466); Pumpkin checks the immediate vehicle's
  controlling passenger, downcasts to BoatEntity and sets paddle state; matches vanilla. No change.
- Row 7 (dead move_from_client path): STILL OPEN - physics.rs outside this unit.
- Row 8 (boat gametest coverage): STILL OPEN - gametest files outside this unit.

No code changes this pass: the two in-scope rows were already fixed by a prior pass and verified
here. No pure unit-test seam exists for these packet handlers (they need a live Player/world/vehicle
graph), so none was added. No rustfmt run (no .rs file touched).


## Result (verify pass 2026-11-18, main checkout)

Re-checked every STILL OPEN row against decompiled vanilla at
/home/linsaftw/.claude/jobs/9615f3b2/tmp/rel-mc-backup/net/minecraft/
(ServerGamePacketListenerImpl.java handleMoveVehicle :578 / handlePaddleBoat :2464,
Entity.java getControlledVehicle :6661).

- Row 1 (bubble time publish): STILL OPEN - needs boat.rs, physics.rs and generated tracked_data
  edits, all outside this unit.
- Row 2 (rider position clamp): STILL OPEN - needs passengers.rs/boat.rs, outside this unit.
- Row 3 (float_boat air-to-water snapping): STILL OPEN - physics.rs outside this unit.
- Row 4 (floatBoat gravity constant): STILL OPEN - physics.rs outside this unit.
- Row 5 (MoveVehicle rotation wrap, move_vehicle.rs): VERIFIED FIXED, no change needed. Vanilla
  wraps with Mth.wrapDegrees (:606-607) before absSnapTo; Pumpkin wraps at move_vehicle.rs:43-44 and
  applies via set_rotation at :81. client_move.rs also wraps (:392-428). No edit required.
- Row 6 (paddle boat handler, paddle_boat.rs): VERIFIED FIXED, no change needed. Vanilla
  getControlledVehicle (:6661) returns the immediate vehicle only when it is the controlling
  passenger; handlePaddleBoat (:2466) requires that vehicle be an AbstractBoat. Pumpkin reads the
  immediate vehicle, checks its controlling passenger, downcasts to BoatEntity and sets paddle state.
  Matches vanilla (the doc's get_root_vehicle suggestion is not what decompiled vanilla does).
- Row 7 (dead move_from_client path): STILL OPEN - physics.rs outside this unit.
- Row 8 (boat gametest coverage): STILL OPEN - gametest files outside this unit.

No code changes: both in-scope rows were already fixed by a prior pass and re-verified here. No
pure unit-test seam exists for these packet handlers (they need a live Player/world/vehicle graph).
No rustfmt run (no .rs file touched). No compile run (shell denied to workers).
