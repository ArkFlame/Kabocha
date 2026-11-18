# Final rows from docs/tracker/tentities-oak-boat-5c8be7.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: boat.rs passengers.rs physics.rs

- Row 2 (rider position clamp)
- Row 3 (float_boat air-to-water snapping)
- Row 4 (floatBoat gravity constant)
- Row 7 (dead move_from_client path)

## Result
- Row 2 (rider position clamp): OPEN. Not audited in this pass.
- Row 3 (float_boat air-to-water snapping): OPEN. Needs physics.rs audit against `AbstractBoat.floatBoat`.
- Row 4 (floatBoat gravity constant): BLOCKED. The 26.3 constant is not in `.vendor`; not generated data.
- Row 7 (dead move_from_client path): OPEN. Needs physics.rs.

Status: open=3, blocked=1. No compile run.
