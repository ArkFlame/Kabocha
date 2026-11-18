# entities / Pale Oak Boat — re-review

Source: `crates/pumpkin-core/src/item/items/boat.rs`, `crates/pumpkin-core/src/entity/vehicle/boat.rs`
(and submodules `physics.rs`, `passengers.rs`, `dismount.rs`, `block_effects.rs`). Last structural change
touching this row: 570d3ea1f "feat: improve boat water physics (#4073)", which rewrote the boat tick,
floating and collision code. Reference: vanilla `net.minecraft.world.entity.vehicle.boat.BoatEntity`
(+ `AbstractBoat` seating/attachment members) for the current MC target. No decompiled sources in
`.vendor/`, so each row names the vanilla member it must match.

Pale oak is not a special-cased wood: it rides the shared boat entity and shared `BoatItem`. The audit
therefore covers (a) every place a pale-oak boat must be registered, and (b) the shared boat behaviour
that the 570d3ea1f rewrite changed.

## Wiring verified (no gaps)

| Where | Check |
|---|---|
| item/items/boat.rs:39-40 | `PALE_OAK_BOAT` / `PALE_OAK_CHEST_BOAT` items map to their entity types |
| item/items/boat.rs:89-90 | both ids registered through `ItemMetadata::ids`, and `mod.rs:158` registers one `BoatItem` for the whole list |
| entity/type.rs:344-345 | both entity types construct `BoatEntity` |
| entity/vehicle/boat.rs:52-53 | `is_chest_boat` includes `PALE_OAK_CHEST_BOAT` (27-slot inventory, 1 passenger) |
| entity/vehicle/boat.rs:43-55 | non-chest pale oak boat keeps 2 passengers |
| net/java/play/pick_item.rs:100 | middle-click pick block falls back to `BoatItem::entity_to_item`, which resolves pale oak |
| entity/player.rs:4375,4385 | `boat_one_cm` statistic includes both pale oak types |
| block/blocks/redstone/dispenser.rs:326-329 | dispenser boat path iterates `BoatItem::ids()`, so pale oak is dispensed |
| entity/vehicle/boat/passengers.rs:67-69,71-79 | single-rider offset 0.15 for chest boats, ride height `height/3` (raft exception) matches vanilla `AbstractBoat.getPassengerAttachmentPos` / `getRideHeight` |
| entity/vehicle/boat.rs:347-357 | boat break drop resolves the item from the entity registry key; `pumpkin-data` has no `entities/<boat>` loot table, so this matches vanilla `BoatEntity` dropping its item rather than a loot table |

## Remaining gaps

| # | Where | Vanilla member | Gap | Fix | Size |
|---|-------|----------------|-----|-----|------|
| 1 | entity/vehicle/boat.rs:377-380 `interact` | `BoatEntity.interact` chest-boat branch | The container-open result is discarded (`let _ = self.open_custom_inventory_screen(player); return true;`). A failed open (out of reach, plugin denied, screen already open) is reported to the client as success, so no fallback happens and the interaction is silently swallowed | Return the result of `open_custom_inventory_screen` (and let the caller fall through to riding when it is `false`); confirm against the decompiled chest-boat `interact` whether a failed open should return `PASS` | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 2 | entity/vehicle/boat.rs:365-366 `interact` | `BoatEntity.interact` mount gate | Mounting is additionally blocked when the player's eye is inside a water block (`!self.is_eye_in_water()`). Vanilla gates only on `ticksUnderwater < 60`, "not already riding" and seat availability, so a player whose eye is submerged next to an empty boat cannot board it here but can in vanilla | Drop the `is_eye_in_water()` term from `can_add_passenger` (or move the check to where vanilla actually uses it) | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 3 | entity/vehicle/boat.rs:327-332 `damage_with_context` | `BoatEntity.damage` / `dropItems` | Break detection is a pre-computed prediction (`will_break`) that must still agree with `VehicleEntity::damage_with_context` (vehicle.rs:243). They currently agree (`damage + amount*10 > 40`), but the coupling is implicit: any change to the vehicle threshold silently stops boat + chest contents from dropping. Vanilla decides once, inside `damage` | Have `vehicle.damage_with_context` return whether it destroyed the entity, or drop from a single post-damage check (`is_removed() && !creative`) instead of the pre-computed flag | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 4 | entity/vehicle/boat/physics.rs:172-213 `get_ground_friction` | `AbstractBoat.getGroundFriction` | `friction / count as f32` yields `NaN` when no block under the boat has a collision shape; it is only harmless because entity/vehicle/boat.rs:84 tests `friction > 0.0`. Vanilla sums then divides only inside the block loop's non-empty case | Guard `count == 0` and return `0.0` explicitly; add a unit test for the empty case | S, FIXED 080a2e01e (`average_friction` returns 0.0 on empty count; unit test; UNVERIFIED, no decompile) |
| 5 | entity/vehicle/boat.rs:288-289 tick | `BoatEntity.tick` client-controlled branch | A client-controlled boat has its velocity zeroed every tick (`entity.velocity.store(Vector3::default())`) but `last_yd` is never updated from the client move, so `get_water_level_above` (physics.rs:146) keeps using a stale `last_yd` after a client move until the boat becomes server-controlled (`move_from_client`, physics.rs:319, does store it) | Store `last_yd` in the client-controlled branch from the accepted `move_from_client` delta as well, and confirm the vanilla client-authoritative path | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |

## Not gaps (checked)

Pale oak tag membership in `minecraft:boat`/`c_boats` statistics paths, chest boat inventory size 27,
`TIME_TO_EJECT` 60-tick underwater ejection, custom-name preservation on both placement
(`spawn_egg::apply_entity_variant`, spawn_egg.rs:138-146) and break drop (boat.rs:351-355), wobble
metadata pairing (vehicle.rs:167-188), seat indices 0.2/-0.6 for two passengers, dismount spot search,
`blocks_building = true` on construction, and dispenser spawning.

## h6 status
- get_ground_friction NaN with no supporting block: FIXED 426efffdc (returns 0.0). No unit test: needs a world fixture.
- Row 1 (chest interact result discarded), row 2 (eye-in-water mount gate), row 3 (damage prediction coupling), row 5 (client velocity vs last_yd): STILL OPEN, not attempted this pass, needs decompiled interact ordering or a shared vehicle destroy return value.


## Result

All four STILL OPEN rows require edits to files outside this unit's only editable source file
(`spawn_egg.rs`), and no decompiled boat source exists under `.vendor/vanilla-decompiled/` to ground
a change, so each is left open with a one-line reason. No source file was edited, so no rustfmt run.

- Row 1 (chest-boat interact result discarded): OPEN - code is in `entity/vehicle/boat.rs`
  (`interact` / `open_custom_inventory_screen`), outside this unit's editable file (`spawn_egg.rs`);
  no decompiled source to confirm vanilla chest-boat interact ordering.
- Row 2 (eye-in-water mount gate): OPEN - code is in `entity/vehicle/boat.rs` (`is_eye_in_water`
  gate in `interact`), outside this unit's editable file (`spawn_egg.rs`); no decompiled source to
  confirm the vanilla mount gate.
- Row 3 (damage prediction coupling): OPEN - code is in `entity/vehicle/boat.rs` and
  `entity/vehicle.rs` (`damage_with_context`); cross-file refactor outside this unit's editable
  file (`spawn_egg.rs`).
- Row 5 (client velocity vs last_yd): OPEN - code is in `entity/vehicle/boat.rs` and
  `entity/vehicle/boat/physics.rs` (`last_yd`), outside this unit's editable file (`spawn_egg.rs`);
  no decompiled source to confirm the vanilla client-authoritative path.

Row 4 was already FIXED in a prior pass.

## Result (re-review)

Re-verified against decompiled vanilla now present under `.vendor/vanilla-decompiled/net/minecraft/`
(`world/entity/vehicle/boat/AbstractBoat.java`, `AbstractChestBoat.java`). This unit's only editable
source file is `spawn_egg.rs`, which handles custom-name preservation on placement
(`apply_entity_variant`, line 146), already confirmed working. None of the STILL OPEN rows map to
`spawn_egg.rs`, so no source file was edited and no rustfmt/test run was needed. fixed=0 open=0.

All four STILL OPEN rows have ALREADY been resolved in the current `boat.rs` by concurrent workers
since this doc was last updated (the doc's old line numbers no longer match the file):

- Row 1 (chest-boat interact result discarded): RESOLVED - `boat.rs:822` now returns
  `self.open_custom_inventory_screen(player)` directly, matching vanilla `AbstractChestBoat.interact`
  which returns the `interactWithContainerVehicle` result.
- Row 2 (eye-in-water mount gate): NON-GAP - decompiled `AbstractBoat.canAddPassenger` is
  `getPassengers().size() < getMaxPassengers() && !isEyeInFluid(FluidTags.WATER)`, so vanilla also
  blocks mounting when the eye is in water; the doc's "vanilla gates only on ticksUnderwater" premise
  was wrong. Current `boat.rs:816` checks `is_eye_in_water(&self.vehicle.entity)` (the boat's own eye),
  matching vanilla. No change needed.
- Row 3 (damage prediction coupling): RESOLVED - `boat.rs:729-732` now drops from a post-damage
  `was_alive && self.vehicle.entity.is_removed()` check instead of a pre-computed `will_break` flag,
  so it cannot drift from the vehicle destroy threshold.
- Row 5 (client velocity vs last_yd): RESOLVED - `boat.rs:602` now stores `last_yd = 0.0` in the
  client-controlled branch, matching vanilla `AbstractBoat.checkFallDamage` storing `lastYd` each tick.

Row 4 remains FIXED from a prior pass.

## Result (wave 3)

Re-checked each STILL OPEN row against `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java`,
`AbstractChestBoat.java` and `vehicle/VehicleEntity.java`. All four are already satisfied by the current
`crates/pumpkin-core/src/entity/vehicle/boat.rs`; no code in this unit's editable file (`spawn_egg.rs`)
is involved, so no source edit, rustfmt or test run was needed.

- Row 1 VERIFIED: vanilla `AbstractChestBoat.interact` (AbstractChestBoat.java:90-96) returns
  `interactWithContainerVehicle`'s result, and `ContainerEntity.interactWithContainerVehicle`
  (ContainerEntity.java:93-100) returns `PASS` when `player.openMenu` fails. Pumpkin `boat.rs:822`
  returns `open_custom_inventory_screen(player)` (false on a failed open), matching PASS. No change.
- Row 2 NOT A GAP: vanilla `AbstractBoat.canAddPassenger` (AbstractBoat.java:836-838) is
  `getPassengers().size() < getMaxPassengers() && !this.isEyeInFluid(FluidTags.WATER)`, so the
  eye-in-water gate is vanilla. Pumpkin gates the boat's own eye in `boat_interact_outcome` (boat.rs:169)
  and reads it at boat.rs:816. The doc's "vanilla gates only on ticksUnderwater" premise was wrong.
- Row 3 VERIFIED: `VehicleEntity.hurtServer` (VehicleEntity.java:35-79) calls `discard`/`destroy`
  itself; Pumpkin's boat `damage_with_context` (boat.rs:720-752) drops from the post-hit
  `was_alive && is_removed() && drops_item_when_destroyed(creative, named)` state, so the tail follows
  the vehicle threshold with no separate `will_break` prediction to drift. No change.
- Row 5 VERIFIED: vanilla `AbstractBoat.tick` (AbstractBoat.java:274-277) sets
  `setDeltaMovement(Vec3.ZERO)` on the client-authoritative branch and `checkFallDamage` (:808-809)
  stores `lastYd = getDeltaMovement().y`, i.e. 0 while a player steers. Pumpkin stores
  `last_yd = 0.0` in the same branch (boat.rs:596-602). No change.
- Row 4 remains FIXED 080a2e01e (`average_ground_friction` guard plus its unit test, physics.rs:264-281).

Status: fixed=0 open=0 verified=4 (rows 1, 2, 3, 5).


## Result (scope: dismount.rs + spawn_egg.rs, re-review)

Independently re-verified every row against decompiled vanilla under
.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/ and
world/entity/. This unit's editable source files are dismount.rs and
spawn_egg.rs; both are already complete and match vanilla, so no source edit,
rustfmt run, or test run was needed.

Rows 1, 2, 3 and 5 map to boat.rs / physics.rs / vehicle.rs, files outside
this unit's editable scope, and are already resolved in the current code:

- Row 1 VERIFIED: boat.rs:816 returns open_custom_inventory_screen(player)
  directly, matching vanilla AbstractChestBoat.interact returning the
  interactWithContainerVehicle result (PASS on a failed open).
- Row 2 NOT A GAP: boat.rs:810 gates mounting on is_eye_in_water of the boat;
  vanilla AbstractBoat.canAddPassenger also blocks when the eye is in water,
  so the doc's ticksUnderwater-only premise was wrong.
- Row 3 VERIFIED: boat.rs:714-725 drops from a post-hit
  was_alive && is_removed() && drops_item_when_destroyed(...) check, so there
  is no will_break prediction that can drift from the vehicle threshold.
- Row 5 VERIFIED: boat.rs:602 stores last_yd = 0.0 in the client-controlled
  branch, matching vanilla AbstractBoat.checkFallDamage storing lastYd each
  tick.

Row 4 remains FIXED (average_ground_friction empty-count guard plus its unit
test in physics.rs); that file is outside this unit.

Editable files checked against vanilla (no gaps found):
- dismount.rs get_collision_horizontal_escape_vector matches vanilla
  Entity.getCollisionHorizontalEscapeVector (Entity.java:6645) exactly.
  get_dismount_location_for_passenger matches AbstractBoat
  .getDismountLocationForPassenger (AbstractBoat.java:709-738): water check on
  the block below, floored target, valid floor height (finite and < 1.0),
  per-pose canDismountTo with the boat-top fallback. Two pure unit tests present.
- spawn_egg.rs apply_variant preserves the custom name (lines 152-154) and
  applies entity variants before entity_data, matching the vanilla order.

fixed=0 open=0 verified=5 files=<doc only: docs/tracker/tentities-pale-oak-boat.md>

## Result (dismount.rs + spawn_egg.rs, wave re-review)

Re-verified each STILL OPEN row against `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java`,
`AbstractChestBoat.java` and the current `crates/pumpkin-core/src/entity/vehicle/boat.rs`. All four rows live in
`boat.rs` (or `physics.rs`/`vehicle.rs`), outside this unit's editable files, and are already satisfied there. No
source edit, rustfmt run or test run was needed; no new seam exists in `dismount.rs`/`spawn_egg.rs`.

- Row 1 VERIFIED: `AbstractChestBoat.interact` (AbstractChestBoat.java:86) opens the container only when
  `canAddPassenger(player) && !player.isSecondaryUseActive()`; Pumpkin `boat.rs:827` returns
  `open_custom_inventory_screen(player)` directly, so a failed open now yields `false` (vanilla PASS).
- Row 2 NOT A GAP: `AbstractBoat.canAddPassenger` (AbstractBoat.java:836-837) is
  `getPassengers().size() < getMaxPassengers() && !isEyeInFluid(FluidTags.WATER)`; Pumpkin models exactly that in
  `boat_interact_outcome` (boat.rs:169) and feeds it `is_eye_in_water` at boat.rs:821.
- Row 3 VERIFIED: `boat.rs:713-735` drops from a post-hit `was_alive && is_removed() && drops_item_when_destroyed(..)`
  check, with no pre-computed `will_break` left to drift from `VehicleEntity`'s destroy threshold.
- Row 5 VERIFIED: `boat.rs:613` stores `last_yd = 0.0` in the client-controlled branch, matching vanilla
  `AbstractBoat.tick` zeroing the delta movement and `checkFallDamage` storing `lastYd` from it.
- Row 4 remains FIXED (physics.rs empty-count guard plus its unit test).

Editable files re-checked against vanilla, no gaps: `dismount.rs` matches `AbstractBoat.getDismountLocationForPassenger`
(AbstractBoat.java:709-738) including the water-below early fallback, floored target, `isBlockFloorValid` (finite,
< 1.0) filtering and per-pose `canDismountTo`; `spawn_egg.rs` `apply_variant` (line 151-154) still preserves the custom
name before entity variants and entity_data.

Status: fixed=0 open=0 verified=5 files=<doc only: docs/tracker/tentities-pale-oak-boat.md>
