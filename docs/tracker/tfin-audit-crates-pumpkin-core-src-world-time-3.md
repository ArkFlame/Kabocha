# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-world-time.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: set_time.rs time.rs

| 7 | STILL OPEN | `WorldClocks.bootstrap` fixes overworld clock id 0 (THE_END is 1), so the hardcoded 0 is currently correct; replacing it with a registry lookup needs the WORLD_CLOCK registry, which is absent from `assets/`/`pumpkin-data` (blocked, needs Extractor). |
All three STILL OPEN rows were re-checked against decompiled vanilla at
| 5 | STILL OPEN (outside unit) | In-unit `ServerClockManager` now keeps one `ClockInstance` per clock name (`server_clock_manager_keeps_clocks_independent`); `/time set <timemarker>`, `of <clock>` and the timemarker resolver already exist in `command/commands/time.rs` (outside this unit). No remaining work in `time.rs` or `set_time.rs`. |
| 5 | STILL OPEN (outside unit) | Clock registry, `/time set <timemarker>` and `of <clock>` already live in `command/commands/time.rs`; the in-unit `ServerClockManager` keeps one `ClockInstance` per clock name and `SleepWakeMarker` resolves the marker, so the residue needs command + full timeline-attribute data (L, outside unit). |

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
