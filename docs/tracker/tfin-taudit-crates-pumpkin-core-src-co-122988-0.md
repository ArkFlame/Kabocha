# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-co-122988.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: time.rs

| 6 | Bedrock has no `commands.time.rate` key, so `/time rate` from a Bedrock client uses `commands.time.set` ("Set the time to 0.5") | `time.rs:300-304` | Bedrock has no rate command | leave the fallback or add a Pumpkin-specific key | S - STILL OPEN: needs a Pumpkin-specific key decision for Bedrock fallback |
- `/time rate` Bedrock message (`time.rs:289-316`): STILL OPEN (not re-read this pass).
- 1. `/time set <timemarker>` markers not implemented: STILL OPEN (M; no `time_marker` symbol in `time.rs`).
- 2. Foreign clock id mutates single `LevelTime`: STILL OPEN (M).

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
