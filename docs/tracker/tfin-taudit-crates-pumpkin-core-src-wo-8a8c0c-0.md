# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-wo-8a8c0c.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: player_spawn_finder.rs spawn_tests.rs

FIXED: NoiseBasedChunkGenerator and Custom spawn height uses sea_level + 1 (world/player_spawn_finder.rs:264-265). STILL OPEN: scan range row 2 and can_be_collided_with row 3 not re-read.
- STILL OPEN row 2 (scan range, `player_spawn_finder.rs:293`): VERIFIED MATCHING, no change. Vanilla
- STILL OPEN row 3 (`can_be_collided_with`, `player_spawn_finder.rs:63-83`): VERIFIED MATCHING, no change. A
it is not a STILL OPEN row and `spawn_tests.rs:404` encodes the current value, so correcting it needs a

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
