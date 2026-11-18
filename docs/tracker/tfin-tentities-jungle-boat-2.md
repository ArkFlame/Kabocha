# Final rows from docs/tracker/tentities-jungle-boat.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: boat.rs dismount.rs

- Delta-rotation accumulation (former h6 row, line 43): code present in `entity/vehicle/boat.rs` (`last_yaw`/`yaw_delta` ~:231-258, per-tick delta ~:575-577, applied in `on_rider_positioned`/`on_passenger_turned` ~:840-939, pure helpers and tests ~:993-1035), matching decompiled `AbstractBoat.positionRider`/`clampRotation`.
- Non-row note: vanilla sets the dismount pose for every living passenger, while `dismount.rs` gates `set_pose` on `is_player`. Unobservable for non-players (their only pose is Standing), left unchanged.

## Result
- Delta-rotation accumulation: VERIFIED (prior pass; code present at boat.rs as above, vanilla `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java` `positionRider`/`clampRotation`). Not re-derived this pass; no edit.

Status: fixed=0 open=0. No compile run (orchestrator gate).
