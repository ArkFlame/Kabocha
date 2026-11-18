# Parity rows for crates/pumpkin-core/src/block/blocks/bubble_column.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/tentities-mangrove-boat** [partial/OWN-OPEN] own audit: 3 open, 1 fixed, 0 verified, 0 blocked-data (docs/tracker/tentities-mangrove-boat.md) docs/tracker/tentities-mangrove-boat.md -- out of scope for this unit (tracked in its own doc, not here).

## Result

Verified 2026-11-18 (third pass) against decompiled vanilla in
.vendor/vanilla-decompiled/net/minecraft/. All rows are closed; no STILL OPEN
rows remain.

- Row 1 (bubble-column physics / bubble time): FIXED/VERIFIED. The boat-side
  implementation is present in crates/pumpkin-core/src/entity/vehicle/boat.rs:
  bubble_column_time / bubble_column_drag_down / bubble_column_above fields,
  on_above_bubble_column (arms the 60-tick countdown and writes ID_BUBBLE_TIME),
  tick_bubble_column (decrements, ejects riders and broadcasts BOAT_SINK /
  BOAT_LAUNCH on expiry) and bubble_column_vertical_velocity. This matches
  .vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java
  :165-172 (onAboveBubbleColumn), 351-386 (tickBubbleColumn) and 399-408
  (handleBubbleColumnEffect: dragDown y-0.7, updraft y=0.6); the splash
  sound/particle at 174-183 also matches. ID_BUBBLE_TIME exists as a generated
  field (pumpkin_data::tracked_data::boat::ID_BUBBLE_TIME), so no generated data
  is missing.
- Row 2 (interact mounting): VERIFIED, current code is already correct. Earlier
  passes wrongly concluded the !is_eye_in_water gate should be removed. Vanilla
  AbstractBoat.interact (AbstractBoat.java:779-787) has no eye check itself, but
  it calls player.startRiding(this), which invokes canAddPassenger; vanilla
  AbstractBoat.canAddPassenger (AbstractBoat.java:836-837) returns
  getPassengers().size() < getMaxPassengers() && !isEyeInFluid(FluidTags.WATER).
  The eye-in-water gate is therefore REQUIRED for parity and must be KEPT.
  boat.rs:826 is_eye_in_water(&self.vehicle.entity) is correct; do NOT remove it.
- Row 3 (passenger attachment point): FIXED/VERIFIED. passengers.rs calls
  super::passenger_attachment (boat.rs:204-222), which reads the generated
  BoatSeats ride_height / single_passenger_x_offset from pumpkin-data
  entity_type.rs:63-65 (ride_height 0.1875 = height/3) instead of the old
  hardcoded 0.2/-0.6. Matches AbstractBoat.java:143-161 and Boat.java:14-17
  (rideHeight = dimensions.height()/3). No generated data is missing.
- In-unit file bubble_column.rs is correct and unchanged (push caps,
  is_open_above surface selection, particles, reconcile/tick logic) and matches
  .vendor/vanilla-decompiled/net/minecraft/world/level/block/BubbleColumnBlock.java
  :46-64 (entityInside), 76-120 (updateColumn/getColumnState/canOccupy),
  122-159 (animateTick particles).

No source file changed this third pass (boat.rs and passengers.rs were fixed in
earlier passes). No rustfmt or compile needed for a docs-only change; no
cargo/git run (forbidden).
