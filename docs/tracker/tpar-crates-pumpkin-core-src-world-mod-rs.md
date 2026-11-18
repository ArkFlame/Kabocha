# Parity rows for crates/pumpkin-core/src/world/mod.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-wo-3e55a4** [partial/OWN-OPEN] own audit: 12 open, 12 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-wo-3e55a4.md) docs/tracker/taudit-crates-pumpkin-core-src-wo-3e55a4.md
- **own/taudit-crates-pumpkin-core-src-world-natu** [partial/OWN-OPEN] own audit: 4 open, 8 fixed, 5 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-world-natu.md) docs/tracker/taudit-crates-pumpkin-core-src-world-natu.md

## Result
## Result (final)

Every row is resolved; none remains STILL OPEN. Source was re-read against
`.vendor/vanilla-decompiled/net/minecraft/` this pass and every fix from earlier
passes is present and type-correct, so no new source edit or rustfmt was needed.
No BLOCKED-DATA row: all fixes use existing generated data and existing APIs.

own/taudit-crates-pumpkin-core-src-wo-3e55a4 (world/mod.rs):

| # | Size | Result |
|---|------|--------|
| 1 | M | VERIFIED - `mark_unsent_light` dirties adjacent-chunk light masks on x/z borders (`trailing_zeros >= 4` / `& 15 == 15`); pure test `border_block_dirties_light_sections_in_adjacent_chunks`. Vanilla `LevelChunk.setBlockState` -> `LightEngine.checkBlock` / `SectionPos.aroundAndAtBlockPos` (`.vendor/vanilla-decompiled/net/minecraft/world/level/chunk/LevelChunk.java`). |
| 2 | M | VERIFIED - `any_player_close_enough_for_spawning` (128-block squared) gates `tick_chunk` before `spawn_for_chunk`. Vanilla `ChunkMap.anyPlayerCloseEnoughForSpawning` (`.vendor/vanilla-decompiled/net/minecraft/server/level/ChunkMap.java:766`). |
| 3 | S | VERIFIED - respawn calls `send_difficulty_update()` / `send_active_effects()` after `send_world_info`. Vanilla `ServerPlayer.restoreFrom` (`.vendor/vanilla-decompiled/net/minecraft/server/level/ServerPlayer.java`). |
| 4 | S | VERIFIED - `spawn_bedrock_player` sends `CSetSpawnPosition` after `CStartGame` (BDS StartGame sequence; no Java source). |
| 5 | L | VERIFIED - `dragon_parts: DashMap` populated by `add_dragon_parts` and merged into entity queries/hit tests (`dragon_part_by_id`). Vanilla `Level.getEntities` (`.vendor/vanilla-decompiled/net/minecraft/world/level/Level.java`). |
| 6 | XS | VERIFIED - no behaviour gap; stale broadcast comment removed (broadcast methods already filter on `watched_section.is_within_distance`). |
| 7 | XS | VERIFIED - no behaviour gap; only comment-only TODOs remain. |
| h3 | M | VERIFIED - `DragonFight::tick` wired; `Implement ender dragon handling` TODO absent. |

own/taudit-crates-pumpkin-core-src-world-natu (natural_spawner.rs):

| # | Size | Result |
|---|------|--------|
| 1 | S | VERIFIED - `is_near` uses `LOCAL_MOB_CAP_RANGE_SQ = 128*128` with no chunk-window gate. Vanilla `ChunkMap.playerIsCloseEnoughForSpawning` (`.vendor/vanilla-decompiled/net/minecraft/server/level/ChunkMap.java:860-866`, 16384.0). |
| 2 | S | VERIFIED - both `get_players_near` branches call the same `is_near` predicate. |
| 3 | XS | VERIFIED - `game_rules.spawn_mobs` gate in `spawn_mobs_for_chunk_generation`. Vanilla `NaturalSpawner.spawnMobsForChunkGeneration` (`.vendor/vanilla-decompiled/net/minecraft/world/level/NaturalSpawner.java:599`). |
| 4 | L | VERIFIED - `world.spawn_random` threaded through `spawn_for_chunk`, `get_random_pos_within` and `spawn_mobs_for_chunk_generation`; per-call reseed gone. Vanilla one-level-random threading. |
| 5 | XS | VERIFIED - `initial_cluster_attempts(roll) = (roll * 4.0).ceil()`, unit-tested. Vanilla `Mth.ceil(fastRandom.nextFloat() * 4.0F)` (`.vendor/vanilla-decompiled/net/minecraft/world/level/NaturalSpawner.java:354`). |
| 6 | S | VERIFIED - `is_spawn_position_ok` / `_cache` check feet (`cur`/`state`) and head (`up`) with `is_valid_empty_spawn_block`. Vanilla `SpawnPlacementTypes.ON_GROUND` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/SpawnPlacementTypes.java:26-33`). |
| 7 | S | FIXED - `adjust_spawn_position_cache` and `adjust_spawn_position` share `adjust_spawn_position_down` with the registry-backed `is_pathfindable(Land)` predicate (cache uses `BlockStateId::to_block`/`to_state`); old `!is_full_cube()` approximation dropped. Vanilla `SpawnPlacementTypes.ON_GROUND.adjustSpawnPosition` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/SpawnPlacementTypes.java:45-47`); test `adjust_spawn_position_down_follows_land_pathfindable`. |
| 8 | S | VERIFIED - OVERWORLD gate plus active-chunks check. Vanilla dimension check (`.vendor/vanilla-decompiled/net/minecraft/world/level/NaturalSpawner.java:453-457`). |
| 9 | S | VERIFIED - `world/mod.rs:3458` calls `any_player_close_enough_for_spawning` in `tick_spawning_chunk`. Vanilla `ChunkMap.anyPlayerCloseEnoughForSpawning` (`.vendor/vanilla-decompiled/net/minecraft/server/level/ChunkMap.java:766`). |
| 10 | M | VERIFIED - `structure_spawn_entries` queries `structure_spawn_override_at` before the biome list; weighted pick unit-tested. Vanilla `NaturalSpawner.mobsAt` (`.vendor/vanilla-decompiled/net/minecraft/world/level/NaturalSpawner.java:558-574`). |
| 11 | XS | FIXED - `EntityType::from_name` failure logs via `tracing::warn` and continues the probability loop. Vanilla skips an unresolvable data-pack entry. |
| h3-sweep | M | VERIFIED - resolved by the same `world/mod.rs:3458` spawn gate as row 9. |


## Result (third pass, final verification)

Re-read every STILL OPEN row from both referenced audit docs against the
actual source and against decompiled vanilla under
`.vendor/vanilla-decompiled/net/minecraft/`. All rows are already resolved in
the checkout (landed by sibling units); no source edit was needed this pass, so
nothing new required rustfmt and no new pure unit test seam was added (row 1's
seam and the natural-spawner seams are already covered by existing tests).

Both OWN-OPEN rows of this doc end as FIXED/VERIFIED (no row left STILL OPEN):

- own/taudit-crates-pumpkin-core-src-wo-3e55a4 (world/mod.rs): VERIFIED MATCHING.
  Previously-open rows, each confirmed present in code:
  - Row 1 (M, neighbouring-chunk light sections): `mark_unsent_light`
    (world/mod.rs:2514-2556) dirties the adjacent-chunk mask on x/z borders
    (trailing_zeros >= 4 / & 15 == 15), matching vanilla
    `LevelChunk.setBlockState -> LightEngine.checkBlock` /
    `SectionPos.aroundAndAtBlockPos`; pure test
    `border_block_dirties_light_sections_in_adjacent_chunks`.
  - Row 2 (M, player-proximity spawn gate): `any_player_close_enough_for_spawning`
    (:3432) gates `tick_chunk` at :3458 with a 128-block range, matching
    `ChunkMap.anyPlayerCloseEnoughForSpawning` (ChunkMap.java:766-819,860-866).
  - Row 3 (S, respawn xp/difficulty/effects): respawn calls
    `send_difficulty_update()` / `send_active_effects()` (:5515-5516), matching
    vanilla `ServerPlayer.restoreFrom`.
  - Row 4 (S, Bedrock world spawn): `spawn_bedrock_player` sends
    `CSetSpawnPosition` (:4103; also :5465) (BDS StartGame sequence, no Java source).
  - Row 5 (L, EnderDragonPart radius/hit): `dragon_parts: DashMap` (:352) merged
    into entity queries/hit tests (`dragon_part_by_id` :5843, `add_dragon_parts`
    :5850, :5898/:5918/:6216), matching vanilla `Level.getEntities`.
  - Row 6 / Row 7 (XS): no behaviour gap; stale comment gone, remaining TODOs are
    comment-only.
  - h3 (M): `DragonFight::tick` wired at :2422; the ender-dragon TODO is absent.

- own/taudit-crates-pumpkin-core-src-world-natu (natural_spawner.rs): VERIFIED MATCHING.
  Previously-open rows, each confirmed present in code:
  - Row 1 (S, local mob-cap window): `is_near` uses LOCAL_MOB_CAP_RANGE_SQ =
    128*128 (:38,184-185), matching `ChunkMap.playerIsCloseEnoughForSpawning`
    (ChunkMap.java:860-866, 16384.0).
  - Row 2 (S, cached vs fallback players): `get_players_near` shares the one
    is_near predicate (:221).
  - Row 3 (XS, doMobSpawning): `game_rules.spawn_mobs` gate (:653), matching
    `NaturalSpawner.spawnMobsForChunkGeneration` (NaturalSpawner.java:599).
  - Row 4 (L, level RNG threading): `spawn_random` threaded through
    `spawn_for_chunk` (:596), `get_random_pos_within` (:618/628) and
    `spawn_mobs_for_chunk_generation` (:646).
  - Row 5 (XS, cluster size): `initial_cluster_attempts` (:851, called :879) mirrors
    `Mth.ceil(fastRandom.nextFloat() * 4.0F)` (NaturalSpawner.java:354); test
    `initial_cluster_attempts_matches_vanilla_ceil` (:1620).
  - Row 6 (S, ON_GROUND feet+head): `is_valid_empty_spawn_block` on cur/up
    (:1308-1309) and state/up (:1346-1347), matching SpawnPlacementTypes.java:26-33.
  - Row 7 (S, cache vs live adjust_spawn_position): both share
    `adjust_spawn_position_down` (:796) with a registry-backed
    `is_pathfindable(Land)` predicate (:819 cache, :843 live), matching
    `SpawnPlacementTypes.ON_GROUND.adjustSpawnPosition`
    (SpawnPlacementTypes.java:45-47); test
    `adjust_spawn_position_down_follows_land_pathfindable` (:1544).
  - Row 8 (S, shared-spawn bubble): OVERWORLD gate, matching the dimension check in
    NaturalSpawner.java:453-457.
  - Row 9 (S, canSpawnEntitiesInChunk): resolved in world/mod.rs:3458 (gate above).
  - Row 10 (M, structure spawn overrides): `structure_spawn_entries` queries
    `structure_spawn_override_at` before the biome list (:1060,:1070), matching
    `NaturalSpawner.mobsAt` (NaturalSpawner.java:558-574); test
    `structure_spawn_entries_are_picked_by_weight` (:1551).
  - Row 11 (XS, unknown spawner): `EntityType::from_name` failure logs and continues
    (:673,:922), matching vanilla skipping an unresolvable data-pack entry.
  - h3-sweep: same world/mod.rs:3458 gate as row 9.

No BLOCKED-DATA row: every fix needs only existing generated data and existing
APIs. Files changed this pass: this doc only.
