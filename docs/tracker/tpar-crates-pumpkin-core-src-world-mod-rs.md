# Parity rows for crates/pumpkin-core/src/world/mod.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-wo-3e55a4** [partial/OWN-OPEN] own audit: 12 open, 12 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-wo-3e55a4.md) docs/tracker/taudit-crates-pumpkin-core-src-wo-3e55a4.md
- **own/taudit-crates-pumpkin-core-src-world-natu** [partial/OWN-OPEN] own audit: 4 open, 8 fixed, 5 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-world-natu.md) docs/tracker/taudit-crates-pumpkin-core-src-world-natu.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)


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
