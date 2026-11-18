# Tracker: entities / Spruce Boat (RE-REVIEW-UNTRACKED)

Re-review date: 2026-11-17. Branch `kab/tentities-spruce-boat-8ca801`. The earlier review
(`4df90a24e`) described a boat that still wrote no bubble time and had no damage game event;
both landed since, so this pass re-reads the current source. No decompiled vanilla sources exist in
`.vendor/` in this worktree, so each row names the vanilla member it must match.

`SPRUCE_BOAT` / `SPRUCE_CHEST_BOAT` still share the generic `BoatEntity`: the only per-variant
data is `EntityType.boat` (`BoatSeats { ride_height: 0.1875, single_passenger_x_offset: 0.0 }`),
item -> entity `item/items/boat.rs:25-26`, id list `item/items/boat.rs:75-76`, constructor
dispatch `entity/type.rs:348-349`, and `is_chest_boat` (`boat.rs:46-58`) which excludes
`SPRUCE_BOAT` (2 seats) and includes `SPRUCE_CHEST_BOAT` (1 seat, 27 slots).

## Source map (current)

- `entity/vehicle/boat.rs`: state :82, tick :317, tick_underwater (bubble time + 60-tick eject)
  :179, damage_with_context :361, interact :421, seat attachment :232, chest NBT :276.
- `entity/vehicle/boat/physics.rs`: get_status :44, is_underwater :62, check_in_water :89,
  get_water_level_above :114, get_ground_friction :142, float_boat :185, move_boat :235,
  move_from_client :257 (dead).
- `entity/vehicle/boat/{passengers,dismount,block_effects}.rs`: rider seating, vanilla dismount
  search, `checkInsideBlocks` sweep.
- `entity/vehicle/vehicle.rs`: hurt/damage decay, wobble metadata, damage/destroy + game event.
- `net/java/play/{paddle_boat,move_vehicle}.rs`: client input packets.

## Fixed this pass

- `entity/mod.rs:1422` `EntityBase::kill`: `/kill` on a non-living entity went straight to
  `Entity::remove`, which drops nothing, so a `SPRUCE_CHEST_BOAT` (and every chest minecart) lost
  its 27 slots instead of scattering them. Vanilla `AbstractChestBoat.remove` /
  `AbstractMinecartContainer.remove` call `Containers.dropContents` whenever
  `reason.shouldDestroy()` (KILLED/DISCARDED), independent of `doEntityDrops`; the kill path now
  scatters `get_container_inventory()` at the entity position before removal. The damage path keeps
  its `claim_drops` guard, so a destroyed boat still scatters exactly once.

## Verified, not gaps

- `boat.rs:179-194` `tick_underwater` now syncs `ID_BUBBLE_TIME` from the underwater tick count
  and ejects at 60, matching `AbstractBoat.tick` (`outOfControlTicks`, `DATA_ID_BUBBLE_TIME`).
- Bubble columns are handled generically: `Entity::on_inside_bubble_column` /
  `on_above_bubble_column` (`entity/mod.rs:661-678`) apply `bubble_column` constants 0.06/0.7 up,
  0.1/1.8 at the surface, -0.03/-0.3 down (`block/blocks/bubble_column.rs:19-24`), and
  `float_boat` then applies its per-tick friction exactly like `AbstractBoat.floatBoat`. Nothing
  boat-specific is missing.
- `physics.rs:142-183` `get_ground_friction`: generated block data has `slipperiness: 0.6` for
  normal blocks (`pumpkin-data/src/generated/block.rs:548722`), i.e. the vanilla
  `Block.getFriction()` value, so `land_friction` is right; the `-1 / ceil` scan bounds only add
  a ring that cannot intersect the contact box.
- `physics.rs:185-233` `float_boat`: gravity -0.04, 0.9/0.45/7.0e-4 branches, buoyancy
  `(v + e * gravity/0.65) * 0.75`, air -> water snap with `last_yd` reset - matches
  `AbstractBoat.floatBoat`.
- `boat.rs:361-419` damage: invulnerability first, creative = no drops, explosion skips the item
  drop, custom name copied - matches `AbstractBoat.hurt`/`destroy`.
- `boat.rs:421-460` `interact`: 60-tick gate, eye-in-water gate, secondary use opens the chest
  only, plain use boards a free seat - matches `AbstractBoat.interact` + `AbstractChestBoat`.
- `passengers.rs:34-65` and `boat.rs:232-264` agree on the seat offsets (0.2/-0.6, animal +0.2)
  and rotation sign; `passenger_attachment_point` is what the generic `position_rider` uses.

## Remaining gaps

| # | file:line | vanilla | gap / suggested fix | size |
|---|---|---|---|---|
| 1 | `entity/mod.rs:1570` `position_rider`, `boat.rs:232` `passenger_attachment_point` | `AbstractBoat.clampRotation` | No rider-rotation clamp for non-player riders: vanilla sets body yaw to the boat yaw, clamps the rider yaw delta to +/-105 deg and syncs head yaw after every `positionRider`. Add an `EntityBase::clamp_rotation` hook called at the end of `position_rider` (default no-op) and the boat override. Players are client-authoritative, so only mob riders change. Needs the decompiled method to pin the `yRotO` adjustment. | M |
| 2 | `physics.rs:257-297` `move_from_client` + `correct_client_position` | n/a | Dead code: the boat MoveVehicle path is unreachable (the generic `net/java/play/move_vehicle.rs` + `has_new_boat_collision` supersede it). Delete both methods and the now-unused `Player`/`CMoveVehicle` imports; `position_rider` stays (reached from `passengers.rs:12`/`entity/mod.rs:6079`). | S | **FIXED** (dead code removed)
| 3 | `server/builtin_game_tests/` | runtime parity | No boat gametest (only the dispenser boat test). Add one covering the air -> water snap, ejection after 60 submerged ticks, and chest scatter on break + on `/kill` (the last one guards the fix above). | S |

## h6 status
- Rider yaw clamp (AbstractBoat.clampRotation, +/-105 deg, body and head sync): FIXED bd8b9bf97 in entity/vehicle/boat/passengers.rs. Unit test: rider_yaw_is_clamped_to_105_degrees_around_boat_yaw. Delta-rotation accumulation not added: UNVERIFIED, STILL OPEN.


## Result

Re-reviewed 2026-11-17 against `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java`: `positionRider` :696, `clampRotation` :740, `onPassengerTurned` :762, `handleBubbleColumnEffect` :399. Decompiled vanilla is present in this worktree (the "no vanilla sources" note at the top is stale). None of the STILL OPEN work lands in this unit's editable files (block.rs, bubble_column.rs, move_vehicle.rs), so no source file was changed and no test was added.

- Row 1 / h6 "delta-rotation accumulation" (STILL OPEN): already implemented in the current checkout. `entity/vehicle/boat.rs:718-752` defines `rider_yaw_with_delta` and `clamp_rider_yaw` with unit tests `rider_yaw_clamps_to_boat_heading_window` and `rider_yaw_follows_boat_yaw_delta_then_clamps`; `yaw_delta` is stored at :439 and consumed by `on_rider_positioned` at :673. Cross-checked against `positionRider` :696-704 (delta applied before clamp, +/-105 window): correct. No longer a gap in this checkout. The one remaining vanilla detail is `onPassengerTurned`'s `passenger.yRotO += yDelta` (AbstractBoat.java:765), which needs `entity/vehicle/boat.rs` and/or `entity/mod.rs`, both outside this unit. Left open.

- Row 2 (dead `move_from_client` / `correct_client_position`): already FIXED before this pass and outside this unit. No action.

- Row 3 (boat gametest): requires `crates/pumpkin-core/src/server/builtin_game_tests/` (only the dispenser boat test exists at `dispensers_and_droppers_extended.rs:335`), outside this unit. Left open.

Fixed: 0. Open: 2. Files changed: this doc only. No .rs file touched, so rustfmt not run.

## Result (2026-11-18)

Re-verified each STILL OPEN / remaining-gap row against
`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java`
(`positionRider` :696, `clampRotation` :740, `onPassengerTurned` :762) and `BubbleColumnBlock`.
None of the open rows land in this unit's editable source files (block.rs, bubble_column.rs,
dispensers_and_droppers_extended.rs, move_vehicle.rs), so no source file was changed and no test
was added.

- Row 1 / h6 "delta-rotation accumulation" (STILL OPEN): now fully implemented in files owned by
  other workers. `entity/vehicle/boat.rs` stores `yaw_delta` (:232/:577), applies the boat yaw delta
  before the +/-105 clamp in `on_rider_positioned` (:846-889), and `on_passenger_turned` (:891+)
  folds `rider_clamp_adjustment` (:1008) into the rider's previous yaw, matching
  `AbstractBoat.onPassengerTurned` :763-765 (`passenger.yRotO += yDelta`). `EntityBase` exposes the
  hook at `entity/mod.rs:801`. Unit tests `rider_yaw_clamps_to_boat_heading_window` and
  `rider_yaw_follows_boat_yaw_delta_then_clamps` cover the clamp/delta math. Cross-checked correct;
  the last open nuance from the prior pass (the `yRotO` fold) is now implemented. Files outside this
  unit, so no action.
- Row 2 (dead `move_from_client` / `correct_client_position`): already removed before this pass, in
  `physics.rs` (outside this unit). No action.
- Row 3 (boat gametest): left open. Needs a new `pub(super)` test function registered in
  `server/builtin_game_tests/mod.rs` (manual `register_function_test` list at :1203-1290), which is
  outside this unit; an unregistered function would trigger dead_code under `-D warnings`. The
  air/water snap, 60-tick eject and chest-scatter assertions also need runtime boat entity access
  that the current `GameTestHelper` does not expose. Reason: requires files outside this unit.
- Bubble column and MoveVehicle paths in this unit's files match vanilla (constants 0.06/0.7/0.1/1.8
  and -0.03/-0.9/-0.3, and the generic MoveVehicle packet supersedes the deleted boat path).
  No boat-specific gap found. No action.

Fixed: 0. Open: 3. Files changed: this doc only. No .rs file touched, so rustfmt not run.


## Result (2026-11-18, third pass)

Re-verified the STILL OPEN rows against decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/` (present in this worktree; the "no vanilla sources"
note at the top is stale). None of the open rows land in this unit's editable files, so no
source file was changed and no test was added.

- Row 1 / h6 rider rotation (STILL OPEN): fully implemented in `entity/vehicle/boat.rs` (outside
  this unit). `yaw_delta` (:232/:588), delta-applied clamp in `on_rider_positioned` (:851-894),
  `on_passenger_turned` folding the clamp adjustment into `rider.y_rot_o` (:896-944), and unit
  tests `rider_yaw_clamps_to_boat_heading_window`, `rider_yaw_follows_boat_yaw_delta_then_clamps`
  and `rider_clamp_adjustment_feeds_previous_yaw` (:1033-1055) match
  `AbstractBoat.positionRider`/`clampRotation`/`onPassengerTurned` (AbstractBoat.java:696-765,
  `passenger.yRotO += yDelta` at :765). No longer a gap; no action possible from this unit.
- Row 2 (dead code): already removed in `physics.rs` (outside this unit). No action.
- Row 3 (boat gametest): left open. Needs a new function registered in the manual
  `register_function_test` list in `server/builtin_game_tests/mod.rs` (:1272), outside this unit;
  an unregistered `pub(super)` function fails `-D warnings` dead_code. Reason: requires files
  outside this unit.
- In-scope files match vanilla: `bubble_column.rs` constants (0.06/0.7, 0.1/1.8, -0.03/-0.3,
  -0.9 surface cap) match `Entity.handleOnAboveBubbleColumn` / `handleOnInsideBubbleColumn`
  (Entity.java:4149-4199) and already have nine pure unit tests. `move_vehicle.rs` is the generic
  MoveVehicle handler (the boat-specific path was removed earlier) and matches the vanilla
  clamp/wrap/controlling-passenger flow in `ServerGamePacketListenerImpl.handleMoveVehicle`
  (:578-710). No boat-specific gap found.

Fixed: 0. Open: 1 (Row 3, out of scope). Files changed: this doc only. No .rs file touched, so
rustfmt not run.


## Result (2026-11-18, second pass, expanded scope)

Re-verified every STILL OPEN / remaining-gap row against decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/`, now with permission to edit any file a row needs.

- Row 1 / h6 rider rotation (STILL OPEN): already fully implemented in `entity/vehicle/boat.rs`
  (outside this unit): `yaw_delta`, delta-applied +/-105 clamp in `on_rider_positioned`,
  `on_passenger_turned` folding the clamp adjustment into `rider.y_rot_o`, plus three pure unit
  tests. Matches `AbstractBoat.positionRider`/`clampRotation`/`onPassengerTurned`
  (AbstractBoat.java:696-765). Verified correct; no action needed.
- Row 2 (dead code): already removed in `physics.rs` before this pass. No action.
- Row 3 (boat gametest): the seven existing `boat_block_effects.rs` tests already cover the
  air->water snap, 60-tick submerged eject, and chest scatter on break. The one uncovered piece
  was chest scatter on `/kill` (the guard for the `EntityBase::kill` container-scatter fix at
  `entity/mod.rs:1491-1512`). Confirmed against decompiled vanilla:
  `AbstractChestBoat.remove` (:71-76) calls `Containers.dropContents` whenever
  `reason.shouldDestroy()`, and vanilla `Entity.kill` removes with `RemovalReason.KILLED`. Added
  `test_chest_boat_scatters_slots_on_kill` to `boat_block_effects.rs` (summon oak chest boat with
  3 NBT slots, `/kill` it, assert 3 diamond item entities scatter) and registered it in
  `server/builtin_game_tests/mod.rs` (id list + `register_function_test`). This closes Row 3.

Files changed: `boat_block_effects.rs`, `builtin_game_tests/mod.rs`, this doc. No unwrap/expect/panic,
no printf, no em dashes; new test copies the existing in-file pattern. rustfmt run by orchestrator.
Not compiled by this worker (per task).

Fixed: 1 (Row 3 /kill guard gametest). Open: 0.


## Result (2026-11-18, third pass, final verification)

Re-verified every remaining row against decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/`. The prior pass left Open: 0; this pass confirms each
row's final disposition and changes no source file (all rows already resolved in this checkout).

- Row 1 / h6 rider rotation: VERIFIED. `entity/vehicle/boat.rs` stores `yaw_delta` (:232/:593),
  applies the delta before the +/-105 clamp in `on_rider_positioned` (:883-898), and
  `on_passenger_turned` folds the clamp adjustment into `rider.y_rot_o` (:948), matching
  `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java`
  `clampRotation` :740-750 (`Mth.clamp(delta, -105.0F, 105.0F)`) and `onPassengerTurned` :763-765
  (`passenger.yRotO += yDelta`). Three pure unit tests cover the clamp/delta math. No action.
- Row 2 (dead `move_from_client` / `correct_client_position`): VERIFIED removed. `grep` for
  `move_from_client|correct_client_position|CMoveVehicle` in `boat/physics.rs` returns no matches;
  the generic MoveVehicle packet supersedes the boat path. No action.
- Row 3 (boat gametest): FIXED and present. `test_chest_boat_scatters_slots_on_kill` exists in
  `boat_block_effects.rs:386` and is registered in `builtin_game_tests/mod.rs` (:280 id list,
  :1869/:1873 `register_function_test`). Vanilla `AbstractChestBoat.remove` :71-76 calls
  `Containers.dropContents` on `RemovalReason.KILLED`; the test guards that scatter. No action.
- In-scope files match vanilla: `bubble_column.rs` constants (up 0.06/0.7, surface up 0.1/1.8,
  down -0.03/-0.3, surface down -0.9) match `Entity.handleOnAboveBubbleColumn` :4149-4163 and
  `handleOnInsideBubbleColumn` :4185-4198 in
  `.vendor/vanilla-decompiled/net/minecraft/world/entity/Entity.java`, with nine pure unit tests.
  `move_vehicle.rs` is the generic MoveVehicle handler; no boat-specific gap. No action.

Fixed: 0 (nothing new; all rows already fixed/verified). Open: 0. Files changed: this doc only.
No .rs file touched, so rustfmt not run.
