# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-world-spaw.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- Both STILL OPEN rows resolved; open=0.
- No source edits needed; no code risk. The other four "Remaining gaps" rows are not marked STILL OPEN

## Result
The requested path `.vendor/vanilla-decompiled/net/minecraft/` is absent in this checkout; verification used the decompiled Paper/Canvas reference this unit was scoped against: `/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft/server/level/PlayerSpawnFinder.java` (the parent doc's stated reference tree).

- `player_spawn_finder.rs:99-109` stored-Y path: FIXED / VERIFIED. Vanilla `findSpawn` (`PlayerSpawnFinder.java:50-68`) has no stored-Y early return and no raw-suggestion early return. With `RESPAWN_RADIUS=0` the single candidate (squareSide 1, candidateCount 1, coprime/offset 0, deltas 0,0) resolves through `getLevelRespawnPos` (column surface); the suggestion's Y is used only as the final fallback after all candidates fail (`scheduleNext` else-branch at :82-86). Current `get_safe_player_spawn_position` (`player_spawn_finder.rs:89-141`) has no standable/early-return branch: `spawn_search_radius` yields 0, `spawn_candidate_count(0)=1`, and `spawn_candidate_offsets(0,0)` yields the single `(0,0)` candidate resolved by `check_spawn_column` to the column surface, with `fixup_spawn_height_at(suggestion)` only after the loop. Covering test `zero_radius_uses_the_column_surface_not_the_stored_height` (`spawn_tests.rs:273`) present.

- `player_spawn_finder.rs:110-117` adventure gate: FIXED / VERIFIED. Vanilla `findSpawn` at `PlayerSpawnFinder.java:51` reads `level.serverLevelData.getGameType() == GameType.ADVENTURE`, i.e. the level's own level.dat game type, and returns `fixupSpawnHeight(level, spawnSuggestion)` at :52. Current code reads `self.level_info.load().game_type == GameMode::Adventure` (`player_spawn_finder.rs:98`); `level_info` is `Arc<ArcSwap<LevelData>>` (`world/mod.rs:344`), the level's own data, whose `game_type` is persisted as NBT `GameType` in level.dat (`pumpkin-world/src/world_info/anvil.rs:270-274` read, :354 write). Adventure mode runs no SPAWN_SEARCH candidate loop and calls `fixup_spawn_height_at(suggestion)`. Covering test `adventure_mode_fixes_height_on_the_suggestion_not_the_column` (`spawn_tests.rs:405`) present. No cross-crate field change needed.

Both rows end FIXED / VERIFIED; open=0. No source edits were needed this pass (both rows were resolved by earlier passes and re-verified here against the decompiled source); rustfmt not required on untouched files. The other four "Remaining gaps" rows were never marked STILL OPEN and remain out of scope.
