# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-world-spaw.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: anvil.rs player_spawn_finder.rs spawn_tests.rs

- `player_spawn_finder.rs:99-109` stored-Y path: STILL OPEN (deliberate, commented in code at :119-123; needs the keep-or-drop decision).
- `player_spawn_finder.rs:110-117` adventure gate on server defaultgamemode: STILL OPEN (confirmed in code; S, needs level game type accessor, not changed).
- `player_spawn_finder.rs:110-117` adventure gate: STILL OPEN. Vanilla reads `serverLevelData.getGameType()` from level.dat, but Pumpkin `LevelData` (pumpkin-world) has no game-type field, so using the level's own type needs a cross-crate field plus serialization and `/defaultgamemode` persistence changes, outside this unit.
- player_spawn_finder.rs:110-117 adventure gate: FIXED / VERIFIED. Current code at :98 reads self.level_info.load().game_type == GameMode::Adventure. LevelData.game_type is the level own level.dat GameType (serialized as GameType in pumpkin-world/src/world_info/anvil.rs :354, deserialized :274, written by /defaultgamemode), so it matches vanilla serverLevelData.getGameType() at PlayerSpawnFinder.findSpawn :51. The earlier STILL OPEN / no game-type field note is stale. Added deterministic test adventure_mode_fixes_height_on_the_suggestion_not_the_column in spawn_tests.rs: sets game_type to Adventure and asserts a void suggestion resolves near the floor (fixupSpawnHeight) instead of the column surface. rustfmt --edition 2024 run on spawn_tests.rs.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
