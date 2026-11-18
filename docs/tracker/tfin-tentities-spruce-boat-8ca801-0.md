# Final rows from docs/tracker/tentities-spruce-boat-8ca801.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: block.rs boat.rs bubble_column.rs mod.rs move_vehicle.rs passengers.rs

- Rider yaw clamp (AbstractBoat.clampRotation, +/-105 deg, body and head sync): FIXED bd8b9bf97 in entity/vehicle/boat/passengers.rs. Unit test: rider_yaw_is_clamped_to_105_degrees_around_boat_yaw. Delta-rotation accumulation not added: UNVERIFIED, STILL OPEN.
Re-reviewed 2026-11-17 against `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java`: `positionRider` :696, `clampRotation` :740, `onPassengerTurned` :762, `handleBubbleColumnEffect` :399. Decompiled vanilla is present in this worktree (the "no vanilla sources" note at the top is stale). None of the STILL OPEN work lands in this unit's editable files (block.rs, bubble_column.rs, move_vehicle.rs), so no source file was changed and no test was added.
- Row 1 / h6 "delta-rotation accumulation" (STILL OPEN): already implemented in the current checkout. `entity/vehicle/boat.rs:718-752` defines `rider_yaw_with_delta` and `clamp_rider_yaw` with unit tests `rider_yaw_clamps_to_boat_heading_window` and `rider_yaw_follows_boat_yaw_delta_then_clamps`; `yaw_delta` is stored at :439 and consumed by `on_rider_positioned` at :673. Cross-checked against `positionRider` :696-704 (delta applied before clamp, +/-105 window): correct. No longer a gap in this checkout. The one remaining vanilla detail is `onPassengerTurned`'s `passenger.yRotO += yDelta` (AbstractBoat.java:765), which needs `entity/vehicle/boat.rs` and/or `entity/mod.rs`, both outside this unit. Left open.
Re-verified each STILL OPEN / remaining-gap row against

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
