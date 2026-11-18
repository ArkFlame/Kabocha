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


## Result (verify pass 2026-11-18, main checkout, re-check)

Re-verified the two in-scope rows against decompiled vanilla at
.vendor/vanilla-decompiled/net/minecraft/ (ServerGamePacketListenerImpl.java
handleMoveVehicle :578 with Mth.wrapDegrees at :606-607 / handlePaddleBoat :2464-2468,
Entity.java getControlledVehicle :6661-6662).

- Row 5 (MoveVehicle rotation wrap, net/java/play/move_vehicle.rs): VERIFIED FIXED, no change.
  Vanilla wraps yaw/pitch with Mth.wrapDegrees (:606-607) before absSnapTo; Pumpkin wraps at
  move_vehicle.rs:43-44 and applies via set_rotation at :81. client_move.rs also wraps
  (import :13, applied :392-428). No edit required.
- Row 6 (paddle boat handler, net/java/play/paddle_boat.rs): VERIFIED FIXED, no change.
  Vanilla getControlledVehicle (:6661-6662) returns the immediate vehicle only for its
  controlling passenger; handlePaddleBoat (:2466) requires that vehicle be an AbstractBoat.
  Pumpkin reads the immediate vehicle, checks its controlling passenger, downcasts to BoatEntity
  and sets paddle state. Matches vanilla (the doc's get_root_vehicle suggestion is not what
  decompiled vanilla does).

All STILL OPEN rows require files outside this unit, so each is left open:
- Row 1 (bubble time publish): boat.rs, physics.rs, generated tracked_data.
- Row 2 (rider position clamp): passengers.rs/boat.rs.
- Row 3 (float_boat air-to-water snapping): physics.rs.
- Row 4 (floatBoat gravity constant): physics.rs.
- Row 7 (dead move_from_client path): physics.rs.
- Row 8 (boat gametest coverage): server/builtin_game_tests.

No code changes this pass: both in-scope rows were already fixed by earlier passes and re-verified
here. No pure unit-test seam exists for these packet handlers (they need a live Player/world/vehicle
graph), so none was added. No rustfmt run (no .rs file touched). No compile run (shell denied to workers).


## Result (second pass 2026-11-18, cross-file edits now permitted, main checkout)

Re-verified every STILL OPEN row against decompiled vanilla at
/home/linsaftw/.claude/jobs/9615f3b2/tmp/rel-mc-backup/net/minecraft/
(AbstractBoat.java floatBoat :607-648, clampRotation :740-754,
positionRider :696-706, onAboveBubbleColumn :164-185,
tickBubbleColumn :351-387, handleBubbleColumnEffect :399-408;
ServerGamePacketListenerImpl.java handleMoveVehicle :578 / handlePaddleBoat :2464;
Entity.java getControlledVehicle :6661, getY(double) :6839).

All previously STILL OPEN code rows are already fixed in the main checkout; no new code
edit was needed in this pass:

- Row 1 (bubble time publish): VERIFIED FIXED, no change. boat.rs carries the
  bubble_column_time / bubble_column_drag_down / bubble_column_above fields (:237-239),
  on_above_bubble_column arms the 60-tick countdown and writes ID_BUBBLE_TIME (:665-716),
  tick_bubble_column decrements and republishes it and applies the launch/sink impulse
  (:429-474), and on_inside_bubble_column uses the capped entity push (:718-723).
  bubble_column_vertical_velocity (:994-996) matches handleBubbleColumnEffect (-0.7 / 0.6)
  and has a pure unit test (:1094-1098). Generated field ID_BUBBLE_TIME already exists.
- Row 2 (rider rotation clamp): VERIFIED FIXED, no change. on_rider_positioned (:851-894)
  and on_passenger_turned (:896-944) apply clampRotation to non-player, non-can-turn-in-boats
  riders; clamp_rider_yaw (:1027-1031), clamp_head_to_body (:1016-1020), rider_body_yaw
  (:1000-1012) and rider_clamp_adjustment (:1036-1038) mirror vanilla, with unit tests
  (:1057-1092). Players and CAN_TURN_IN_BOATS entities stay client authoritative.
- Row 3 (float_boat air-to-water snapping): VERIFIED CORRECT, no change needed. The doc's
  premise was inverted: vanilla floatBoat (:614-618) runs the noCollision test FIRST and
  only calls move when the target box is clear. physics.rs:196-211 does the same, returning
  the offset only when no_collision passes. No snap-while-blocked change required.
- Row 4 (floatBoat gravity constant): VERIFIED CORRECT, no change needed. Vanilla uses
  -getGravity() (:608) for vspeed and getDefaultGravity()/0.65 only for buoyancy (:645);
  physics.rs:214 uses -self.get_gravity() and physics.rs:235 uses DEFAULT_GRAVITY/0.65.
  get_gravity (boat.rs:982-988) honours no_gravity exactly like vanilla getGravity().
- Row 5 (MoveVehicle rotation wrap): VERIFIED FIXED, no change. move_vehicle.rs:49-50 wraps
  yaw/pitch with wrap_degrees and applies at :91; vanilla wraps with Mth.wrapDegrees
  (:606-607). client_move.rs also wraps.
- Row 6 (paddle boat handler): VERIFIED FIXED, no change. paddle_boat.rs reads the immediate
  vehicle, requires it be the controlling passenger, downcasts to BoatEntity and sets paddle
  state. Vanilla getControlledVehicle (:6661-6662) is the same immediate-vehicle check and
  handlePaddleBoat (:2466) requires an AbstractBoat.
- Row 7 (dead move_from_client path): VERIFIED FIXED, no change. physics.rs no longer defines
  move_from_client; the generic move_vehicle.rs handler supersedes it.
- Row 8 (boat gametest coverage): STILL OPEN - needs a builtin gametest plus a runtime run;
  gametest files are outside this unit and the worker shell is denied, so it stays open.

Compile-pitfall re-check: no source file was edited this pass, so no new import paths,
helper placement, struct initializers, clones, or clippy-pedantic issues were introduced.
No rustfmt run (no .rs file touched). No compile run (shell denied to workers).


## Result (third pass 2026-11-18, main checkout, final)

Full re-verify of all 8 rows against decompiled vanilla at
.vendor/vanilla-decompiled/net/minecraft/ (the task-given path, now present in this checkout).
Every row is in a terminal state; no STILL OPEN rows remain. No source file needed an edit this
pass because every fix was already present and verified below.

- Row 1 (bubble time publish, boat.rs): VERIFIED FIXED, matches vanilla
  .vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java
  onAboveBubbleColumn :165-172, tickBubbleColumn :351-386, handleBubbleColumnEffect :399-408.
  Pumpkin boat.rs arms the 60-tick countdown and writes ID_BUBBLE_TIME (:647-659),
  decrements and republishes it (:429-447), ejects riders and broadcasts BoatSink/BoatLaunch
  (:451-467), and applies the -0.7 / 0.6 vertical impulse via bubble_column_vertical_velocity
  (:472 and :994-996). Generated field ID_BUBBLE_TIME exists, so this is not BLOCKED-DATA.
- Row 2 (rider rotation clamp, boat.rs): VERIFIED FIXED, matches vanilla AbstractBoat.java
  positionRider :696-706 and clampRotation :740-754. Pumpkin on_rider_positioned (:856-894) and
  on_passenger_turned (:896-944) clamp non-player riders to +/-105 deg via clamp_rider_yaw and
  clamp_head_to_body (:1009-1038), with pure unit tests (:1039-1051). Players and
  CAN_TURN_IN_BOATS riders stay client authoritative, as in vanilla.
- Row 3 (float_boat air-to-water snap, physics.rs): VERIFIED CORRECT, matches vanilla
  AbstractBoat.java floatBoat :611-618. The noCollision test runs first (:201) and the surface
  snap offset is returned only when the target box is clear (:211). The doc premise was inverted;
  no snap-while-blocked change is required.
- Row 4 (floatBoat gravity, physics.rs): VERIFIED CORRECT, matches vanilla floatBoat :608
  (-getGravity) and :645 (getDefaultGravity/0.65). Pumpkin physics.rs:214 uses -get_gravity() and
  :235 uses DEFAULT_GRAVITY/0.65; get_gravity honours no_gravity just like vanilla getGravity().
- Row 5 (MoveVehicle rotation wrap, move_vehicle.rs): VERIFIED FIXED, matches vanilla
  .vendor/vanilla-decompiled/net/minecraft/server/network/ServerGamePacketListenerImpl.java
  handleMoveVehicle :606-607 (Mth.wrapDegrees). Pumpkin wraps at move_vehicle.rs:49-50 and applies
  via set_rotation at :91; client_move.rs also wraps.
- Row 6 (paddle boat handler, paddle_boat.rs): VERIFIED FIXED, matches vanilla
  ServerGamePacketListenerImpl.java handlePaddleBoat :2466 and Entity.java getControlledVehicle
  :6661-6662 (immediate vehicle plus controlling-passenger check). Pumpkin reads the immediate
  vehicle, checks its controlling passenger, downcasts to BoatEntity and sets paddle state.
- Row 7 (dead move_from_client path, physics.rs): VERIFIED FIXED, no move_from_client definition
  remains; the generic move_vehicle.rs handler supersedes it.
- Row 8 (boat gametest coverage): VERIFIED FIXED, coverage present in
  crates/pumpkin-core/src/server/builtin_game_tests/boat_block_effects.rs:
  test_boat_floats_up_after_fall (:135-169), test_boat_ejects_passenger_after_60_underwater_ticks
  (:173-229) and test_chest_boat_scatters_27_slots_on_break (:233-278). These are exactly the
  float-up, 60-tick ejection and 27-slot scatter scenarios the row asked for, so the old "no boat
  gametest coverage" premise is stale; no duplicate test was added. Runtime execution is the
  orchestrator gate because the worker shell is denied.

Compile-pitfall re-check: no .rs file was edited this pass, so no new import paths, helper
placement, struct initializers, clones or clippy-pedantic issues were introduced. No rustfmt run
(no .rs file touched). No cargo or git command run (shell denied to workers).

Status: PASS fixed=0 open=0 files=docs/tracker/tentities-oak-boat-5c8be7.md
