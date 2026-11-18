# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-13d289.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: damage_type.rs move_vehicle.rs statistics.rs

All 12 rows resolved; none remain STILL OPEN. Re-verified the current code against vanilla 1.21.4 decompiled sources under `.vendor/vanilla-decompiled/`. No `.rs` file needed an edit this pass: the three unit files (`statistics.rs`, `move_vehicle.rs`, `damage_type.rs`) already carry the row 4 and row 5 fixes, and row 8 is upstream parity rather than a bug. `rustfmt` not rerun (no `.rs` touched); compilation intentionally left to the orchestrator.
No `## Result` section left any row STILL OPEN, and re-reading the current source confirms every row is FIXED or VERIFIED against vanilla 1.21.4 decompiled sources under `.vendor/vanilla-decompiled/`. No `.rs` file was changed this pass, so rustfmt was not needed; compilation is left to the orchestrator.
Files edited this pass: none. No remaining STILL OPEN rows; no BLOCKED-DATA dependency outstanding.
Files edited this pass: `docs/tracker/taudit-crates-pumpkin-core-src-en-13d289.md` only (stale gap status resolved + this Result). No source file needed an edit; no remaining STILL OPEN or BLOCKED-DATA rows.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
