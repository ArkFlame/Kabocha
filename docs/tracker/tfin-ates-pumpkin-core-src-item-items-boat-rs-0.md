# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-item-items-boat-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

All rows CLOSED. Per row: FIXED / VERIFIED with the .vendor vanilla path, or BLOCKED-DATA with the exact missing generated field.

## Result

FINAL PASS 2026-11-18. Independently re-verified every row against decompiled vanilla under
`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/` (`AbstractBoat.java` and
`AbstractChestBoat.java`). Current Pumpkin code already matches vanilla and carries pure unit
tests, so no `.rs` file needed a change. No generated `pumpkin-data` field or missing subsystem
is required, so no row is BLOCKED. No source file edited, so rustfmt is not applicable.

The parent doc's single row, `own/tentities-cherry-boat-ae864b`, expands (via
`docs/tracker/tentities-cherry-boat-ae864b.md`) into the items below. All live in
`crates/pumpkin-core/src/entity/vehicle/boat.rs` and its `boat/` submodules.

- Row 1 / h6 "Delta-rotation accumulation not added": FIXED / VERIFIED (already implemented).
  `BoatEntity` stores `last_yaw`/`yaw_delta` (boat.rs:231-232, init :257-258); `tick` recomputes the
  delta from consecutive yaw samples each tick (:590-593); `on_rider_positioned` adds it to rider
  yaw/head yaw then clamps body and head (:856-899); `on_passenger_turned` re-clamps and folds the
  adjustment into `yRotO` (:901-949). Pure helpers `rider_body_yaw` (:982-994), `clamp_head_to_body`
  (:998-1002), `clamp_rider_yaw` (:1009-1013), `rider_clamp_adjustment` (:1018-1020) with tests
  (:1037-1060). Matches `.vendor/.../AbstractBoat.java:696-705` (positionRider adds `deltaRotation`
  to YRot/YHeadRot, then clampRotation), `:740-754` (clampRotation: body yaw = hull, non-mob head
  clamped to +-105 deg), `:756-759` (full-boat animal +/-90 deg body shift) and `:762-766`
  (onPassengerTurned folds yDelta into yRotO). The h6 note claiming the gap was open was stale.
- Row 2 "drop ticks_underwater / is_eye_in_water from mounting gate": VERIFIED, no change. The doc
  claim is contradicted by vanilla. `.vendor/.../AbstractBoat.java:784` keeps
  `!(outOfControlTicks < 60.0F)` in `interact` and `:836-837` keeps
  `size < getMaxPassengers() && !isEyeInFluid(WATER)` in `canAddPassenger`. Pumpkin's
  `boat_interact_outcome` (boat.rs:161-177, called :823-830) and its `seat_available && !eye_in_water`
  gate (:169) match: a submerged or long-out-of-control boat refuses riders in vanilla too. Tests
  `chest_boat_interact_follows_vanilla_mount_container_nothing_chain` cover the chain.
- Row 3 "increment outOfControlTicks only for UnderWater": VERIFIED, no change. The doc claim is
  contradicted by vanilla. `.vendor/.../AbstractBoat.java:243-247` increments `outOfControlTicks`
  for BOTH `UNDER_WATER` and `UNDER_FLOWING_WATER`; `:856-859` `isUnderWater()` covers both. Pumpkin
  `BoatStatus::is_underwater` (boat/physics.rs) covers both and `tick_underwater` (boat.rs:~392-402)
  uses it. Correct.
- Row 4 "chest contents scatter only in damage path": FIXED / VERIFIED (already implemented).
  `BoatEntity::on_removed` (boat.rs:488-506) scatters the 27 chest slots (`scatter_chest_contents`,
  :272-288) on every destroying removal via `removal_reason.should_destroy()`; `claim_drops` makes
  it run at most once and the damage path also calls it (:746). Matches
  `.vendor/.../AbstractChestBoat.java:71-77` (`remove` scatters contents when `reason.shouldDestroy()`).
- Capacity cross-check (stale "chest boats = 2" note): VERIFIED, no change. Vanilla
  `AbstractBoat.getMaxPassengers` returns 2 (`.vendor/.../AbstractBoat.java:840-842`) while
  `AbstractChestBoat.getMaxPassengers` overrides to 1 (`.vendor/.../AbstractChestBoat.java:48-50`).
  Pumpkin `BOAT_MAX_PASSENGERS = 2` / `CHEST_BOAT_MAX_PASSENGERS = 1` (boat.rs:47-48) selected by
  `max_passengers` (:95-101) is correct.
- `boat/dismount.rs` (no open row): VERIFIED line-by-line against
  `.vendor/.../AbstractBoat.java:709-738` (`getDismountLocationForPassenger`: escape vector from
  `getBbWidth()*SQRT_OF_TWO`, `BlockPos.containing`, water tag gate, valid-floor filter, pose-outer/
  target-inner loop, `super` fallback), `Entity.getCollisionHorizontalEscapeVector`
  (`(cw+cw2+1e-5)/2`, x=-sin, z=cos, scale=max(|x|,|z|)) and `DismountHelper.canDismountTo`
  (pose dimensions, empty collisions, worldborder containment). Two pure tests cover pose sets and
  the escape vector. Correct.

No BLOCKED rows: none require a generated pumpkin-data field or a missing subsystem.
