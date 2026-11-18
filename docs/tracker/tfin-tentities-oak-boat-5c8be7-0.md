# Final rows from docs/tracker/tentities-oak-boat-5c8be7.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: boat.rs passengers.rs physics.rs

- Row 1 (bubble time publish)
- Row 2 (rider position clamp)
- Row 3 (float_boat air-to-water snapping)
- Row 4 (floatBoat gravity constant)

## Result
- Row 1 (bubble time publish): VERIFIED. `boat.rs` `tick_bubble_column` (~:429-474) publishes `ID_BUBBLE_TIME` on every countdown change, matching `AbstractBoat.tickBubbleColumn` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java` :351-387). No generated tracked-data field is missing.
- Row 2 (rider position clamp): OPEN. Not audited in this pass (passengers.rs / boat.rs clamp path).
- Row 3 (float_boat air-to-water snapping): OPEN. Needs physics.rs audit against `AbstractBoat.floatBoat`; the vendored tree is a Paper-era version (see oak-boat-0).
- Row 4 (floatBoat gravity constant): BLOCKED. The vendored `floatBoat` uses `getGravity()`; the 26.3 constant cannot be confirmed from `.vendor` (not generated data).

Status: verified=1, open=2 (rows 2, 3), blocked=1 (row 4). No compile run.
