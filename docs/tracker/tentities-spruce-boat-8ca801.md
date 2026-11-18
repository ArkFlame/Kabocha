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
