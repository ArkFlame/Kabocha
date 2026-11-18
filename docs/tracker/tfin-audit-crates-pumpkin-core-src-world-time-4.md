# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-world-time.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: mod.rs time.rs

Re-verified the single remaining STILL OPEN row against decompiled vanilla at
| 5 | STILL OPEN (outside unit) | In-unit `ServerClockManager` keeps one `ClockInstance` per clock name (`server_clock_manager_keeps_clocks_independent`) and is wired by `world/mod.rs:624-633` and `command/commands/time.rs:205-219,375`; `/time set <timemarker>` and `of <clock>` already exist in `command/commands/time.rs` (outside unit). Residue needs full datapack timeline-attribute data (L, cross-crate). |
Re-verified the only remaining STILL OPEN row (5) against decompiled vanilla at
No `STILL OPEN` rows remain. No new unit test added: every pure seam already has

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
