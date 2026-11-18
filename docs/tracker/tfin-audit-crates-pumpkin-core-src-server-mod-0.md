# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-server-mod.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: mod.rs

| 4 | `server/mod.rs:1614` (`update_tick_times`) | vanilla `MinecraftServer` debug sample sends `SampleType.TICK_TIME` with the raw ring entry (worst/current tick), not `tick_duration.max(target)` | mirror the vanilla 4-value sample layout field by field | S - STILL OPEN: vanilla debug sample source not decompiled here |
- rows 1-4, 7: STILL OPEN.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
