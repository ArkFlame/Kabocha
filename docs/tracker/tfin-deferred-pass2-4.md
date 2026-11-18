# Final rows from docs/tracker/deferred-pass2.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: player_spawn_finder.rs time.rs

- taudit-crates-pumpkin-core-src-world-spaw.md: - `player_spawn_finder.rs:110-117` adventure gate: STILL OPEN. Vanilla reads `serverLevelData.getGameType()` from level.dat, but Pumpkin `LevelData` (pumpkin-world) has no game-type field, so using the level's own type needs a cross-crate field plus serialization and `/defaultgamemode` persistence c
- taudit-crates-pumpkin-core-src-world-time.md: | 5 | STILL OPEN | Clock registry + `/time set <timemarker>` + `custom:clock` need `command/commands/time.rs` (outside unit), a WORLD_CLOCK registry absent from `pumpkin-data`, and timeline data (cross-crate refactor, L). |
- taudit-crates-pumpkin-core-src-world-time.md: | 7 | STILL OPEN | `WorldClocks.bootstrap` fixes overworld clock id 0 (THE_END is 1), so the hardcoded 0 is currently correct; replacing it with a registry lookup needs the WORLD_CLOCK registry, which is absent from `assets/`/`pumpkin-data` (blocked, needs Extractor). |
- tblocks-beds.md: STILL OPEN and were left alone; they need generated-data/codegen changes.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
