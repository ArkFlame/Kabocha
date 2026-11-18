# Final rows from docs/tracker/deferred-pass2.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: player_spawn_finder.rs time.rs

- taudit-crates-pumpkin-core-src-world-spaw.md: - `player_spawn_finder.rs:110-117` adventure gate: STILL OPEN. Vanilla reads `serverLevelData.getGameType()` from level.dat, but Pumpkin `LevelData` (pumpkin-world) has no game-type field, so using the level's own type needs a cross-crate field plus serialization and `/defaultgamemode` persistence c
- taudit-crates-pumpkin-core-src-world-time.md: | 5 | STILL OPEN | Clock registry + `/time set <timemarker>` + `custom:clock` need `command/commands/time.rs` (outside unit), a WORLD_CLOCK registry absent from `pumpkin-data`, and timeline data (cross-crate refactor, L). |
- taudit-crates-pumpkin-core-src-world-time.md: | 7 | STILL OPEN | `WorldClocks.bootstrap` fixes overworld clock id 0 (THE_END is 1), so the hardcoded 0 is currently correct; replacing it with a registry lookup needs the WORLD_CLOCK registry, which is absent from `assets/`/`pumpkin-data` (blocked, needs Extractor). |
- tblocks-beds.md: STILL OPEN and were left alone; they need generated-data/codegen changes.

## Result
Closed against decompiled vanilla at `.vendor/vanilla-decompiled/net/minecraft/`.
No source edits were needed this pass: every carried row was already resolved in its own
tracker unit by an earlier pass; this pass verifies and records. Vanilla paths below are
relative to `/run/media/linsaftw/SDD/GitHub/ArkFlame/Kabocha/.vendor/vanilla-decompiled/net/minecraft/`.

- `taudit-crates-pumpkin-core-src-world-spaw.md` - adventure gate (`player_spawn_finder.rs:98`): FIXED / VERIFIED. Vanilla `server/level/PlayerSpawnFinder.java:51` gates on `level.serverLevelData.getGameType() == GameType.ADVENTURE` (the level's own level.dat GameType). Pumpkin reads `self.level_info.load().game_type == GameMode::Adventure`; `LevelData.game_type` (`pumpkin-world/src/world_info/mod.rs:59`) is that persisted level.dat field (anvil.rs:270-274 read, :354 write), not the transient server default. Covering test `adventure_mode_fixes_height_on_the_suggestion_not_the_column` present; no stored-Y or standable-suggestion early return remains. No edit needed.
- `taudit-crates-pumpkin-core-src-world-time.md` row 5 (clock registry + `/time set <timemarker>` + `custom:clock`): FIXED / VERIFIED. `world/time.rs` holds the per-name `ClockInstance` registry and `ServerClockManager`; `command/commands/time.rs` resolves markers and `of <clock>`; `world/environment.rs` + generated `environment_attribute.rs` implement the environment-attribute timeline tracks. Matches `world/timeline/Timelines.java`, `util/KeyframeTrackSampler.java`, `world/attribute/EnvironmentAttributeSystem.java` and `world/attribute/WeatherAttributes.java`. The carried STILL OPEN note is stale; outside this unit's file scope and already fixed in tree. No edit needed.
- `taudit-crates-pumpkin-core-src-world-time.md` row 7 (overworld clock id hardcoded 0): VERIFIED. `default_clock_id` (`world/time.rs:24-36`) now reads the `minecraft:world_clock` dynamic-registry raw id (both `world_clock` and `timeline` are loaded in `data/datapack/dynamic_registry_loader.rs`), so no hardcoded id remains. Matches `world/clock/WorldClocks.java` `bootstrap` (OVERWORLD registered first = 0, THE_END = 1). The carried "needs missing WORLD_CLOCK registry" note is stale. No edit needed.
- `tblocks-beds.md` (carried STILL OPEN rows 39/40/41): BLOCKED-DATA. Missing generated fields: `crates/pumpkin-data/src/generated/dimension.rs:14-18` `BedRule` has only `can_sleep`, `can_set_spawn`, `explodes`; it lacks `destroy_on_leave` and `error_message` (vanilla `world/attribute/BedRule.java:12` record `BedRule(canSleep, canSetSpawn, destroyOnUse, destroyOnLeave, errorMessage)`). `crates/pumpkin-data/src/generated/environment_attribute.rs` has `GameplayBedRule` but no `GameplayStrawBedRule`/`STRAW_BED_RULE` (vanilla `world/attribute/EnvironmentAttributes.java:141-142`). Per-position lookup needs a bed-rule attribute value in `crates/pumpkin-data/src/generated/biome.rs`, which exposes only `boolean_attributes` (vanilla `AbstractBedBlock.java:63-65` + `world/attribute/EnvironmentAttributeSystem.java`). Generated files are off-limits and regeneration needs Extractor. Rows 42 (`getSleepHeight`) and 43 (sleeping/dead guard) were already FIXED/VERIFIED in that unit. No edit needed.

No `STILL OPEN` rows remain. No `.rs` file was edited this pass, so rustfmt is not applicable; orchestrator compiles.

## Result (batch x_10, CODER)
- This doc contains no open row text; only a stub Result header. Nothing to close here. Files edited: none.

## Result (batch x_11, CODER)
- taudit-crates-pumpkin-core-src-world-spaw.md adventure gate (`player_spawn_finder.rs:110-117`): VERIFIED. The "no game-type field" premise is stale. `LevelData.game_type` exists (`crates/pumpkin-world/src/world_info/mod.rs:59`) and the gate reads it at `crates/pumpkin-core/src/world/player_spawn_finder.rs:98`, matching vanilla `server/level/PlayerSpawnFinder.java:51` (`serverLevelData.getGameType() == ADVENTURE`). Row stays for the orchestrator to reconcile with its source doc. No `.rs` edited this pass; rustfmt not run.
- Malformed list entry `"[""- taudit-...` in the x_11 batch file: not a tracker doc, skipped; the real spawn doc was processed above.
