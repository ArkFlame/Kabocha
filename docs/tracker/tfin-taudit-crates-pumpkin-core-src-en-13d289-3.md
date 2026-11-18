# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-13d289.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: damage_type.rs move_vehicle.rs player.rs statistics.rs

- All 12 rows resolved. Rows 4 and 5 are carried by this unit's files (entity/player/statistics.rs + net/java/play/move_vehicle.rs, and generated pumpkin-data/.../damage_type.rs); rows 1-3 and 6-12 were resolved in prior passes in entity/player.rs. No row remains STILL OPEN; see the final Result below.
Re-verified each STILL OPEN row against `vanilla-decompiled` 1.21.4 (Paper where noted). `rustfmt --edition 2024 crates/pumpkin-core/src/entity/player.rs` ran clean. Compilation intentionally not run (orchestrator compiles once after all workers).
Re-verified every STILL OPEN row against vanilla 1.21.4 decompiled sources (`.vendor/vanilla-decompiled`) and the current code. The current `player.rs` is ahead of the earlier Result section: rows 1, 4 and 11 are now also implemented, so no source edit was needed this pass. `rustfmt --edition 2024` is not rerun because no `.rs` file was touched. Compilation intentionally not run (orchestrator compiles once after all workers).
Re-verified STILL OPEN rows against vanilla 1.21.4 decompiled sources and current code. No source files in this unit were edited (only this doc). No cargo/git/checks run.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
