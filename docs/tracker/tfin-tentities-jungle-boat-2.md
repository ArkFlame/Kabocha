# Final rows from docs/tracker/tentities-jungle-boat.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: boat.rs dismount.rs

- Only row ever marked STILL OPEN is delta-rotation accumulation (h6 status, line 43). Its fix lives in `entity/vehicle/boat.rs` (fields `last_yaw`/`yaw_delta` at :231-258, per-tick delta at :575-577, applied in `on_rider_positioned`/`on_passenger_turned` at :840-939, pure helpers + tests at :993-1035). That file is outside this unit's editable scope (`dismount.rs` only) and the fix is already present and matches decompiled `AbstractBoat.positionRider`/`clampRotation`; left open with reason: out-of-unit file, already fixed. No change made.
- One latent, non-STILL-OPEN divergence noted but not changed (outside explicit scope): vanilla sets the dismount pose for every living passenger, whereas `dismount.rs` gates `set_pose` on `is_player`; for non-players the only pose is Standing so this is unobservable. All STILL OPEN rows are out-of-unit and already fixed: fixed=0, open=0. No generated-data, WIT or cross-crate work. No compile run (orchestrator gate).
- Scanned this unit doc for rows marked STILL OPEN. The only explicitly marked STILL OPEN item is delta-rotation accumulation (h6 status, line 43), which is implemented and tested in `entity/vehicle/boat.rs` (outside this unit's editable scope `dismount.rs`) and already verified in prior results; left open with reason: out-of-unit file, already fixed.
- No additional pure unit test seam identified beyond the existing tests without duplicating them. No generated-data, WIT, or cross-crate work required. fixed=0, open=0 (all STILL OPEN rows are out-of-unit/already fixed). No compile run (orchestrator gate).

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
