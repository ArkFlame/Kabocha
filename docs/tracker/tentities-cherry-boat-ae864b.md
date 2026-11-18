# entities / Cherry Boat - audit (2026-11-17, re-review)

Tracker row: `entities | Cherry Boat | RE-REVIEW-UNTRACKED`.
The shared `BoatEntity` source changed since the 2026-11-16 review: physics, block effects and
dismount handling now live in `crates/pumpkin-core/src/entity/vehicle/boat/{physics,block_effects,dismount,passengers}.rs`.
Cherry boats use that shared code plus the wiring below.

## Wiring (complete)

| Concern | Rust |
|---|---|
| item -> entity | `crates/pumpkin-core/src/item/items/boat.rs:37`, id list `:87` |
| entity -> item | `item/items/boat.rs:53` (`entity_to_item`, `Boat.getPickResult`) |
| entity construction | `entity/type.rs:350-351` (cherry), `:352-353` (poplar) |
| dimensions / seats | generated `pumpkin-data/entity_type.rs:2136`, `boat` seats used in `passengers.rs` |
| chest inventory | `boat.rs:41 is_chest_boat`, `:37` 27 slots, `:227-237` NBT |
| boat_one_cm stat | `entity/player.rs:4376`, `:4386` (chest) |
| buoyancy / status / friction | `boat/physics.rs:215-273 float_boat`, reset/eject `boat.rs:136-159` |
| block collision effects | `boat/block_effects.rs:49 apply_effects_from_blocks` |
| dismount search | `boat/dismount.rs:18-75` |

Resolved since the last review: server-side `floatBoat` (water gravity/buoyancy, land friction,
`Boat.Status` transitions) and chest-content scatter on the damage destruction path (`boat.rs:307-357`).

## Fixed in this pass

- `boat.rs:92-94` - chest boats were capped at 1 passenger (`get_max_passengers` returned 1).
  Vanilla `AbstractBoat.canAddPassenger` returns `passengers.size() < 2` for every boat; `ChestBoat.interact`
  only opens the container when sneaking or when no seat is free. Fixed: constant `MAX_PASSENGERS = 2`
  (`boat.rs:39`, used at `boat.rs:363`).

## Remaining gaps (shared `BoatEntity`, visible on a cherry boat)

1. `boat/passengers.rs:12-16, 34-65` - rider rotation is never clamped to the boat. Vanilla
   `Boat.positionRider` does `passenger.setYHeadRot(passenger.getYHeadRot() + this.deltaRotation)` and then
   `clampRotation(passenger)` (body yaw = boat yaw, head yaw wrapped to +-105 deg relative to the boat).
   Pumpkin has no `delta_rotation` or `clamp_rotation` (grep: no hits under `crates/pumpkin-core/src`), so a
   cherry-boat rider can look backwards and the rider's head does not follow the hull while it turns.
   Fix: track `deltaRotation` in `BoatEntity` and add `EntityBase::clamp_rotation` called from both
   `position_rider` and `passenger_attachment_point`. Size M.

2. `boat.rs:359-362` - `interact` refuses mounting while `ticks_underwater >= TIME_TO_EJECT` or the boat's
   eye is in water. Vanilla `Boat.interact` only rejects secondary use and then relies on
   `startRiding` (`canAddPassenger`, player not already riding, boarding cooldown); the underwater eject
   lives in `Boat.tick` (`outOfControlTicks >= 60`). A submerged cherry boat accepts riders in vanilla.
   Fix: drop `ticks_underwater < TIME_TO_EJECT` and `!self.is_eye_in_water()` from `can_add_passenger`
   (then delete the now-unused `physics.rs:45-72 is_eye_in_water`). Size S.

3. `boat.rs:136-142 tick_underwater` - ejection counter increments for `UnderWater` and
   `UnderFlowingWater` (`BoatStatus::is_underwater`, `physics.rs:24-27`). Vanilla `AbstractBoat.floatBoat`
   only increments `outOfControlTicks` in the `UNDER_WATER` case; flowing water does not count. Fix:
   increment only for `BoatStatus::UnderWater`. Size S.

4. `boat.rs:307-357` + `entity/vehicle/vehicle.rs:190-214` - chest contents scatter only inside the boat
   damage path. `Entity::remove` (`entity/mod.rs:4281`) carries no `RemovalReason`, so `/kill`, plugin-driven
   removal or any other destroy path calls `kill_and_drop_self`, which drops only the loot table and never
   `scatter_inventory`. Vanilla `ChestBoat.remove(RemovalReason)` scatters the 27 slots whenever
   `reason.shouldDestroy()`. Fix: add a destroy hook (or pass `RemovalReason`) so `ChestBoat` scatters once
   per destroying removal. Size M.

## h6 status
- Rider yaw clamp (AbstractBoat.clampRotation, +/-105 deg, body and head sync): FIXED bd8b9bf97 in entity/vehicle/boat/passengers.rs. Unit test: rider_yaw_is_clamped_to_105_degrees_around_boat_yaw. Delta-rotation accumulation not added: UNVERIFIED, STILL OPEN.

## Result

- Row "Delta-rotation accumulation not added: STILL OPEN" (h6 status): VERIFIED ALREADY FIXED, no code change needed. Current code implements it: BoatEntity.yaw_delta (boat.rs:169, init :193) is recomputed every tick from consecutive yaw samples (boat.rs:436-439 wrap_degrees(yaw - last_yaw)), and on_rider_positioned (boat.rs:666-677) applies rider_yaw_with_delta(rider.yaw, boat.yaw, yaw_delta) to non-player riders, syncing body/head yaw. Pure-function unit test rider_yaw_follows_boat_yaw_delta_then_clamps exists (boat.rs:742-752). This matches decompiled AbstractBoat.positionRider (AbstractBoat.java:696-705: add deltaRotation to YRot/YHeadRot, then clampRotation) and clampRotation (AbstractBoat.java:740-754: body yaw = boat yaw, head clamped to +-105 deg). The doc h6 note is stale: the clamp test was renamed to rider_yaw_clamps_to_boat_heading_window (boat.rs:735) and the delta logic lives in boat.rs, not passengers.rs. position_rider (entity/mod.rs:1648) invokes the hook once per tick via ride_tick_riders (world/mod.rs:2248-2252).
- Cross-check of the two S gaps whose doc claims are contradicted by the decompiled vanilla - left unchanged on purpose: (a) gap 2 "drop ticks_underwater < TIME_TO_EJECT": vanilla AbstractBoat.interact (AbstractBoat.java:784) DOES keep !(this.outOfControlTicks < 60.0F), so boat.rs:621 is correct. (b) gap 3 "increment only for UnderWater": vanilla AbstractBoat.tick (AbstractBoat.java:243-247) increments outOfControlTicks for BOTH UNDER_WATER and UNDER_FLOWING_WATER, so boat.rs:279-283 using is_underwater() is correct.
- No .rs file modified this pass; rustfmt not applicable.


## Result (re-verification 2026-11-18)

All four STILL OPEN rows checked against decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java`.
No source file is in this unit's edit allowlist (basenames: none), so no .rs file was touched.

- Row 1 / h6 "Delta-rotation accumulation not added: STILL OPEN": VERIFIED ALREADY FIXED.
  `BoatEntity` holds `last_yaw`/`yaw_delta` AtomicCells (boat.rs:231-232); `tick` recomputes the
  delta every tick from consecutive yaw samples (boat.rs:574-577); `on_rider_positioned`
  applies it to non-player riders and mob heads (boat.rs:873-888). Pure tests
  `rider_yaw_follows_boat_yaw_delta_then_clamps`, `rider_yaw_clamps_to_boat_heading_window`
  and `rider_clamp_adjustment_feeds_previous_yaw` exist (boat.rs:1027-1050). Matches
  `AbstractBoat.positionRider` (adds `deltaRotation`, then `clampRotation` to +-105 deg).
  Doc note is stale (no `rider_yaw_with_delta` fn; logic lives in boat.rs, not passengers.rs).
- Row 2 "drop ticks_underwater / is_eye_in_water from mounting gate": LEFT OPEN, doc claim
  CONTRADICTED by vanilla. `AbstractBoat.interact` (AbstractBoat.java:784) keeps
  `!(outOfControlTicks < 60.0F)` and `AbstractBoat.canAddPassenger` (:836-838) returns
  `size < getMaxPassengers() && !isEyeInFluid(WATER)`. Pumpkin's `boat_interact_outcome`
  inputs at boat.rs:813-820 (ticks_underwater >= TIME_TO_EJECT, is_eye_in_water,
  has_vehicle, seat_available) are correct; a submerged boat does NOT accept riders in vanilla.
- Row 3 "increment outOfControlTicks only for UnderWater": LEFT OPEN, doc claim CONTRADICTED.
  `AbstractBoat.tick` (AbstractBoat.java:243-247) increments for BOTH UNDER_WATER and
  UNDER_FLOWING_WATER. Pumpkin `BoatStatus::is_underwater` (boat/physics.rs:22-23) matches
  both, and `tick_underwater` (boat.rs:397) uses it: correct, no change.
- Row 4 "chest contents scatter only in damage path": VERIFIED ALREADY FIXED via destroy hook.
  `BoatEntity::on_removed` (boat.rs:488-490) calls `scatter_chest_contents`, invoked from the
  central `World::remove_entity` (world/mod.rs:6615) for every removal reason (`/kill`, plugin,
  discard, void, break). `claim_drops` guard (boat.rs:276) makes the 27-slot scatter run at
  most once, and the damage path also calls it (boat.rs:736). Matches vanilla
  `AbstractChestBoat.remove(RemovalReason)` scattering whenever `reason.shouldDestroy()`.
- No code change was warranted: two rows were already implemented and two doc claims were
  proven wrong against vanilla. rustfmt not applicable (no .rs file edited).

## Result (dismount.rs verification 2026-11-18)

Unit allowlist is `dismount.rs` only. No row in this doc is marked STILL OPEN against that
file: the four "Remaining gaps" live in passengers.rs/boat.rs/physics.rs and the only literal
STILL OPEN note (h6 delta-rotation) is already covered by the two prior Result sections and
resides in boat.rs. Verified `boat/dismount.rs` line-by-line against decompiled vanilla:
- `AbstractBoat.getDismountLocationForPassenger` (AbstractBoat.java:709-738): escape vector
  from `getBbWidth()*SQRT_OF_TWO` + passenger yaw, target X/Z, `BlockPos.containing(..., maxY, ...)`
  (`floored_v(fallback+direction)`), `belowBlockPos` water gate (`get_fluid_and_fluid_state`
  + MINECRAFT_WATER tag), valid-floor filter (`is_finite() && <1.0` vs `isBlockFloorValid`),
  pose-outer/target-inner loop, and the `super` fallback (`Vec3(x, maxY, z)` = `fallback`) all match.
- `Entity.getCollisionHorizontalEscapeVector` (Entity.java:6645-6651): `(cw+cw2+1e-5)/2`,
  x=-sin, z=cos, scale=max(|x|,|z|), matches `get_collision_horizontal_escape_vector` exactly.
- `DismountHelper.canDismountTo` (DismountHelper.java:42-53): bounds from rider pose dimensions
  moved to location, empty block collisions, worldborder containment all match `can_dismount_to`.
- `dismount_poses` matches `Player.getDismountPoses` (Standing/Crouching/Swimming) vs
  `LivingEntity` (Standing); two pure unit tests cover pose sets and escape vector.
- Minor vanilla nuance only: vanilla calls `setPose(dismountPose)` for every passenger while
  Rust does so only for players; non-player dismount poses are exclusively Standing (no-op),
  so behaviour is observably equivalent. Not changed to avoid untestable churn in a file with
  no open row. No code change; no compile needed (no .rs file edited).
