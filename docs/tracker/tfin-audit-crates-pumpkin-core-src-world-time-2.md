# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-world-time.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: mod.rs time.rs

- Row 7 (overworld clock id hardcoded to 0): STILL OPEN. The world-clock registry is absent from `assets/` and `pumpkin-data`, so the id cannot be read (needs Extractor, blocked).
All three STILL OPEN rows were re-checked against decompiled vanilla at
| 3 | STILL OPEN | `ServerClockManager.resolveTimerMove` (row 3) is driven by `ClockTimeMarker`/timeline data and the sleep-skip call site is `world/mod.rs` (now lines 2663/2671), outside this unit; needs timeline data support (L). |
| 5 | STILL OPEN | Clock registry + `/time set <timemarker>` + `custom:clock` need `command/commands/time.rs` (outside unit), a WORLD_CLOCK registry absent from `pumpkin-data`, and timeline data (cross-crate refactor, L). |

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
