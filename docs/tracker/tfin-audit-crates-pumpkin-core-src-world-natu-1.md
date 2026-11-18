# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-world-natu.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: mod.rs natural_spawner.rs

All 11 gap rows plus the h3-sweep row are closed in the final Result below. The
in-scope files (natural_spawner.rs and world/mod.rs) are present and type-correct;
cross-file rows were verified read-only against the decompiled vanilla source under
.vendor/vanilla-decompiled/net/minecraft/. The PRIOR-OPEN h3-sweep line in the parent
audit Status section is superseded by that final Result.

## Result (final pass, unit closed)

All 11 gap rows plus the h3-sweep row from the parent audit were re-checked
against `.vendor/vanilla-decompiled/net/minecraft/`. Every row ends FIXED or
VERIFIED MATCHING; no row remains PRIOR-OPEN or LEFT OPEN. No source edit was
needed in this pass: the prior-pass fixes in `natural_spawner.rs` and
`world/mod.rs` are present and type-correct. The PRIOR-OPEN / LEFT OPEN markers
in the parent doc's earlier Result sections and its h3 Status line are superseded
by this result.

- Row 1 (local mob-cap player window): VERIFIED MATCHING. Vanilla
  `ChunkMap.getPlayersCloseForSpawning` uses a 16384.0 squared horizontal radius
  and rejects spectators (`.vendor/.../server/level/ChunkMap.java:846,860-866`).
  Pumpkin's `LOCAL_MOB_CAP_RANGE_SQ` = 128*128 with `is_near`
  (natural_spawner.rs:38,181-186); `for_world` filters spectators when building
  the active snapshot (149-161). No change.
- Row 2 (cached vs fallback player paths): VERIFIED MATCHING. Both branches of
  `get_players_near` call the same `is_near` predicate
  (natural_spawner.rs:221-246), so can_spawn and add_mob/remove_mob see the same
  player set. No change.
- Row 3 (chunk-gen honours doMobSpawning): VERIFIED MATCHING. Vanilla
  `spawnMobsForChunkGeneration` gates on GameRules.SPAWN_MOBS
  (`.vendor/.../world/level/NaturalSpawner.java:599`); Pumpkin checks
  `game_rules.spawn_mobs` (line 653). No change.
- Row 4 (level RandomSource threading): VERIFIED MATCHING. Vanilla threads one
  level.random through getRandomPosWithin / spawnCategoryForChunk
  (NaturalSpawner.java:236,577-584); Pumpkin threads `world.spawn_random` through
  spawn_for_chunk / get_random_pos_within / spawn_mobs_for_chunk_generation
  (596-649), with no per-call reseed. No change.
- Row 5 (initial cluster size): VERIFIED MATCHING. Vanilla uses
  `Mth.ceil(fastRandom.nextFloat() * 4.0F)` (NaturalSpawner.java:354); Pumpkin's
  `initial_cluster_attempts` (851-853) mirrors it and has a unit test. No change.
- Row 6 (ON_GROUND head-block check): VERIFIED MATCHING. Vanilla validates the
  floor plus feet and head emptiness
  (`.vendor/.../world/entity/SpawnPlacementTypes.java:26-41`); Pumpkin checks
  down/cur/up on both cache and live paths. No change.
- Row 7 (chunk-gen vs live adjust_spawn_position): FIXED (prior pass),
  RE-VERIFIED. Vanilla `adjustSpawnPosition` calls `state.isPathfindable(LAND)`
  (SpawnPlacementTypes.java:45-47; default `!isCollisionShapeFullBlock`,
  BlockBehaviour.java:130-136). Both Pumpkin paths share
  `adjust_spawn_position_down` with the registry-backed `is_pathfindable(.., Land)`
  predicate (796-846); `BlockStateId::to_block/to_state` and the registry
  signature exist, so the fix type-checks. No change.
- Row 8 (shared-spawn bubble / ticking chunk): VERIFIED MATCHING. Vanilla applies
  the respawn check only when `respawnData.dimension() == level.dimension()`
  (NaturalSpawner.java:446,453-457); Pumpkin's OVERWORLD gate plus active-chunks
  check is a conservative match. No change.
- Row 9 (canSpawnEntitiesInChunk gate): VERIFIED MATCHING (in world/mod.rs,
  handled by an earlier pass). Vanilla gates per-tick spawning on
  `ChunkMap.anyPlayerCloseEnoughForSpawning` (ChunkMap.java:766-819); Pumpkin's
  `tick_spawning_chunk` calls `any_player_close_enough_for_spawning` (3432-3459),
  a 128-block non-spectator XZ test. No change.
- Row 10 (structure spawn overrides): VERIFIED MATCHING. Vanilla `mobsAt`
  consults structure overrides before the biome list
  (NaturalSpawner.java:558-574); Pumpkin's `structure_spawn_entries` (1060-1081)
  checks `structure_spawn_override_at` first, with a weighted-pick unit test.
  No change.
- Row 11 (unknown spawner aborts chunk-gen): FIXED (prior pass), RE-VERIFIED.
  Vanilla stores resolved entity types in the spawner list
  (NaturalSpawner.java:609-611); Pumpkin's `EntityType::from_name` failure logs
  via `tracing::warn` and continues the loop (673-678), skipping the bad entry.
  No change.
- h3-sweep row (canSpawnEntitiesInChunk TODO, world/mod.rs): VERIFIED resolved by
  the same `any_player_close_enough_for_spawning` gate at line 3458. No change.

Existing pure unit tests cover the seams: local_mob_cap_uses_128_block_horizontal_range,
after_spawn_updates_category_counts, adjust_spawn_position_down_follows_land_pathfindable,
structure_spawn_entries_are_picked_by_weight, initial_cluster_attempts_matches_vanilla_ceil.
No new seam was added, so no new pure unit test. Files changed this pass: this
doc only.
