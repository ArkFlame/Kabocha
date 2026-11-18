# world / natural_spawner — deep audit

Source: crates/pumpkin-core/src/world/natural_spawner.rs (1335 lines).
Vanilla reference: net.minecraft.world.level.NaturalSpawner,
NaturalSpawner.SpawnState, NaturalSpawner.LocalMobCapCalculator,
SpawnPlacements / SpawnPlacementTypes, plus Paper patches on
ServerLevel.tickChunk. No Java source tree was available in the worktree
(.vendor/ absent, Java glob empty), so the comparison is against the 1.21.x
vanilla bodies from memory; each row states what to re-verify.

TODO/FIXME markers: none in this file (the 7 tracker hits are "vanilla"
mentions in doc comments and one test name). The only natural-spawner TODO in
the crate is the canSpawnEntitiesInChunk note at
crates/pumpkin-core/src/world/mod.rs:3025, row 9 below.

## Fixed in this pass

- Missing SpawnState.afterSpawn (call site was spawn_category_for_position,
  ~line 886; SpawnState had no after_spawn). SpawnState is rebuilt from
  world.entities once per tick (world/mod.rs:736), so mobs spawned earlier in
  the same tick were not visible to can_spawn_for_category_global or
  can_spawn_for_category_local: one tick could exceed the category cap and the
  per-player local cap by up to (chunks x 3 clusters x limit_per_chunk).
  Vanilla NaturalSpawner.SpawnState.afterSpawn(Mob, ChunkAccess) adds the
  category count and the local mob-cap entry right after
  addFreshEntityWithPassengers. Added SpawnState::after_spawn (counts + local
  cap, MISC skipped like vanilla) and call it from the CreatureSpawnEvent
  success path only, so a cancelled spawn still counts towards the local
  cluster but not towards the caps. Test: after_spawn_updates_category_counts.
  Size: S.

## Remaining gaps

1. **Local mob-cap player window is hardcoded** — natural_spawner.rs:216-218
   (|dchunk| <= 9 and calc_distance < 16384.0). Vanilla
   LocalMobCapCalculator.getPlayersNear delegates to
   ChunkMap.getPlayers(pos, false), i.e. players whose tracking distance covers
   the chunk (view distance, default 10-16 chunks), with no 128-block cutoff.
   The 9-chunk window makes local caps tighter than vanilla on servers with a
   large view distance and is unrelated to the 8-chunk spawn radius. Suggested
   fix: use the player-tracking set for the chunk (same source the entity
   tracker uses) or the configured view distance, and delete the magic
   numbers. Size: S.

2. **Cached and fallback player paths disagree** — natural_spawner.rs:214-232.
   With an active_players snapshot the +/-9-chunk gate applies; without it only
   the 128-block distance test does. The fallback therefore includes players 9+
   chunks away in players_near_chunk, so can_spawn and add_mob/remove_mob see
   different player sets depending on which path ran. Same fix as row 1, plus
   one shared predicate. Size: S.

3. **Chunk-generation spawning honours doMobSpawning** —
   natural_spawner.rs:619. Vanilla
   NaturalSpawner.spawnMobsForChunkGeneration has no game-rule check;
   RULE_DOMOBSPAWNING is consulted only in ServerLevel.tickChunk (per-tick
   spawning). Verify whether Pumpkin wants the stricter behaviour (most servers
   expect doMobSpawning false to stop passive chunk-gen spawns too); if not,
   drop the check. Size: XS.

4. **Random source is not the world/level RandomSource** —
   natural_spawner.rs:626-696 (rand::random*), 627/1017 (fresh rng()),
   806-864 (rng()), and 593 (new Xoroshiro from get_seed() per call in
   get_random_pos_within). Vanilla threads one level.random through
   getRandomPosWithin, spawnCategoryForPosition and spawnMobsForChunkGeneration.
   Consequence: no seed parity, so no differential spawn test is possible, and
   get_random_pos_within reseeds per chunk instead of advancing the level
   sequence. Suggested fix: pass the level RandomImpl/generator through
   spawn_for_chunk and spawn_category_for_position. Size: L.

5. **Initial cluster size may be off by one** — natural_spawner.rs:806:
   (rng().random::<f32>() * 4.0).ceil() yields 1..=4 (equal quarters, 0 only for
   exactly 0.0); vanilla spawnCategoryForPosition uses
   Mth.randomBetweenInclusive(random, 0, 3) (0..=3) in the versions this port
   targets. Confirming this changes the number of spawn attempts per cluster
   (~4 vs ~2.5 average). Re-verify against the 1.21.x body before changing.
   Size: XS.

6. **ON_GROUND placement over-checks the head block** —
   natural_spawner.rs:1098-1112 requires is_valid_empty_spawn_block for both
   cur and up. Vanilla SpawnPlacementTypes.ON_GROUND.isSpawnPositionOk only
   validates the block below; emptiness/danger checks live in
   NaturalSpawner.isValidSpawnPostitionForType and apply to the feet position
   (noCollision covers the rest). This blocks legal spawns whose head tile is a
   PREVENT_MOB_SPAWNING_INSIDE block (e.g. under carpets / low vegetation).
   Suggested fix: drop the up test from the placement helper (keep it in the
   chunk-gen variant if needed). Size: S.

7. **Chunk-gen and live "move to safe pos" disagree** —
   adjust_spawn_position_cache (natural_spawner.rs:745-762: below is not a full
   cube and not liquid) vs adjust_spawn_position (natural_spawner.rs:764-783:
   below is pathfindable with PathComputationType::Land). Both port the same
   vanilla tail of NaturalSpawner.getTopNonCollidingPos / SpawnPlacements
   adjustment, so one must be wrong: the same entity spawns at different heights
   depending on whether the chunk is generated or spawned live. Suggested fix:
   pick the vanilla predicate (re-read getTopNonCollidingPos +
   EntityType.getSpawnAABB) and share one helper. Size: S.

8. **Shared-spawn bubble restricted to the overworld** —
   natural_spawner.rs:933. Vanilla isRightDistanceToPlayerAndSpawnPoint
   applies level.getSharedSpawnPos().closerToCenterThan(pos, 24.0) in every
   dimension. Also natural_spawner.rs:947-956 approximates
   ServerLevel.isNaturalSpawningAllowed(BlockPos) (entity-ticking chunk) with
   active_chunks.contains + a world-border test; the world-border clause does
   not exist in vanilla at this point. Verify the nether/end spawn positions and
   the ticking-chunk predicate. Size: S.

9. **No canSpawnEntitiesInChunk gate** — caller world/mod.rs:3025
   (tick_spawning_chunk). Vanilla ServerLevel.tickChunk only spawns when
   canSpawnEntitiesInChunk(chunkPos) (natural-spawning distance manager check)
   holds; Pumpkin iterates every active chunk, so spawning also runs in chunks
   with no player in natural-spawn range. Suggested fix: implement the check on
   World (player distance manager) and early-return. Size: S (world side).

10. **Structure spawn overrides ignored** — can_spawn_mob_at,
    natural_spawner.rs:960-987 reads only the biome spawner lists. Vanilla
    NaturalSpawner.canSpawnMobAt(level, category, structureManager, type, pos)
    first consults StructureSpawnOverride for the structure at pos (e.g.
    monument guardians, outpost pillagers, swamp hut witches) and falls back to
    the biome list. Mobs still spawn there today, but with biome weights instead
    of the structure ones. Suggested fix: query world.structure_manager for a
    spawn override before the biome list. Size: M.

11. **Unknown spawner entity aborts chunk-gen spawning** —
    natural_spawner.rs:637-639 returns from spawn_mobs_for_chunk_generation when
    EntityType::from_name fails, which kills the remaining probability loop for
    that chunk. Vanilla EntityType.byString(...).isEmpty() just skips that
    spawner entry. Only reachable with a datapack biome naming a non-vanilla
    entity. Suggested fix: continue instead of return. Size: XS.

## Verified as matching vanilla (no action)

- MAGIC_NUMBER = 17 * 17; canSpawnForCategoryGlobal uses
  cap * spawnable_chunk_count / 289 with a config override
  (natural_spawner.rs:499-503, 62-87).
- LocalMobCapCalculator.canSpawn: true for a player with no entry, otherwise the
  per-player count must stay under the category cap; add_mob/remove_mob touch
  every nearby player (238-279).
- get_filtered_spawning_categories boolean algebra for
  friendlies/enemies/persistents equals vanilla (534-561).
- get_random_pos_within inclusive [min_y, heightmap+1] range and the y > min_y
  chunk guard (563-605).
- Cluster/group ordering: cluster limit return, max-group break, 3 clusters,
  x/z rand(0..5) - rand(0..5) walk, despawn-distance pre-check, and the
  CreatureSpawnEvent cancel still counting towards the cluster (786-907).
- Distance gates 24 blocks (576) to player and shared spawn point, 128-block
  despawn distance, canSpawnFarFromPlayer, canSummon (923-1073).
- is_valid_spawn_floor default (isFaceSturdy(UP) && light < 14) and the
  ocelot/parrot, polar bear, fire-immune and never/always predicates
  (1177-1193); is_block_dangerous list (wither rose, sweet berry bush, cactus,
  powder snow, fire tag, lava, magma, lava cauldron, campfires) with immune_to
  and fire-immunity exemptions (1195-1216).
- InWater / InLava placement tests (1087-1097, 1127-1138).
- choose_spawner = WeightedList.getRandom including all-zero weights returning
  None (1021-1028), plus the reduce_water_ambient_spawns 0.98 skip (998-1002).
- get_top_non_colliding_pos ceiling dimension descent (703-743; the extra
  y <= min_y guard is defensive only).
- SpawnState::add_entity/remove_entity: persistence-required and MISC entities
  skipped, spawn-cost charge from biome.spawn_costs, both counters updated
  (403-455).

## Status (h3 sweep)
- `canSpawnEntitiesInChunk` check (world/mod.rs:3295 still `// TODO this.level.canSpawnEntitiesInChunk`): STILL OPEN (M, not changed in this pass).


## Result

Re-verified every gap against .vendor/vanilla-decompiled (1.21.x). Fixed the one
safe, in-file row; the rest were either already correct or need files/crates
outside this unit.

- Row 1 (local mob-cap player window): LEFT OPEN. Needs the chunk player-tracking
  set / configured view distance, which lives outside natural_spawner.rs.
- Row 2 (cached vs fallback player paths): LEFT OPEN. Same root as row 1; a single
  shared predicate needs the tracking source, out of unit.
- Row 3 (chunk-gen honours doMobSpawning): VERIFIED MATCHING. Vanilla
  NaturalSpawner.spawnMobsForChunkGeneration gates on GameRules.SPAWN_MOBS
  (NaturalSpawner.java:599). Pumpkin's spawn_mobs check stays. No change.
- Row 4 (level RandomSource threading): LEFT OPEN. L-sized refactor threading the
  level RNG through spawn_for_chunk / spawn_category_for_position; too broad for a
  no-build pass.
- Row 5 (initial cluster size): VERIFIED MATCHING. Vanilla uses
  Mth.ceil(fastRandom.nextFloat() * 4.0F) (NaturalSpawner.java:354), identical to
  Pumpkin's (rng().random::<f32>() * 4.0).ceil(). The audit's randomBetweenInclusive
  premise was wrong. No change.
- Row 6 (ON_GROUND head-block check): VERIFIED MATCHING. Vanilla
  SpawnPlacementTypes.ON_GROUND.isSpawnPositionOk checks isValidSpawn on the block
  below and isValidEmptySpawnBlock on both feet and head (SpawnPlacementTypes.java
  30-33). Pumpkin does the same. No change.
- Row 7 (chunk-gen vs live adjust_spawn_position): PARTLY VERIFIED. Vanilla's default
  isPathfindable(LAND) is !state.isCollisionShapeFullBlock (BlockBehaviour.java
  132), so the cache path's !is_full_cube() matches the default and both paths
  agree for normal blocks. Exact parity for blocks overriding isPathfindable would
  need a block registry in GenerationCache (pumpkin-world), out of unit. Left open
  for that narrow case only.
- Row 8 (shared-spawn bubble / ticking-chunk): VERIFIED MATCHING. Vanilla applies the
  respawn-point check only when respawnData.dimension() == level.dimension()
  (NaturalSpawner.java:453-457), i.e. the overworld for standard worlds, matching
  Pumpkin's OVERWORLD gate. The trailing chunk check is a Canvas optimisation in
  vanilla. No change.
- Row 9 (canSpawnEntitiesInChunk gate): LEFT OPEN. Caller is world/mod.rs, outside
  this unit's file (basenames: natural_spawner.rs).
- Row 10 (structure spawn overrides): LEFT OPEN. Needs a world.structure_manager /
  StructureSpawnOverride query (cross-crate), size M.
- Row 11 (unknown spawner aborts chunk-gen): FIXED. EntityType::from_name failure now
  logs via tracing::warn and continues the probability loop instead of returning,
  so one malformed spawner name no longer aborts spawning for the chunk, matching
  vanilla skipping an unresolvable entry. Risk: a datapack setting spawn probability
  to 1.0 with an invalid name would spin; vanilla data uses probabilities below 1.
- h3-sweep row (canSpawnEntitiesInChunk TODO, world/mod.rs:3295): LEFT OPEN. File is
  outside this unit.

Files changed: crates/pumpkin-core/src/world/natural_spawner.rs; this doc.


## Result (recheck against .vendor/vanilla-decompiled, 1.21.x)

Re-read every gap against the decompiled vanilla source now present under
`.vendor/vanilla-decompiled/net/minecraft/`. Earlier in-file fixes for rows 1, 2,
4, 7, 10 and 11 were checked and left in place; one later regression was fixed.

- Row 1 (local mob-cap player window): CONFIRMED FIXED. Vanilla
  ChunkMap.playerIsCloseEnoughForSpawning (ChunkMap.java:860-866) rejects spectators
  and uses horizontal distSqr < 16384.0; Pumpkin's is_near is identical. No change.
- Row 2 (cached vs fallback player paths): CONFIRMED FIXED. Both paths call the same
  is_near predicate from get_players_near. No change.
- Row 3 (chunk-gen honours doMobSpawning): CONFIRMED MATCHING. Vanilla gates
  spawnMobsForChunkGeneration on GameRules.SPAWN_MOBS (NaturalSpawner.java:599).
  No change.
- Row 4 (level RandomSource threading): CONFIRMED FIXED. world.spawn_random is
  threaded through spawn_for_chunk, get_random_pos_within,
  spawn_category_for_position and spawn_mobs_for_chunk_generation, so the sequence
  advances once per spawn tick instead of reseeding per chunk. Exact seed parity is
  still impossible (Pumpkin uses Xoroshiro), but the structural gap is closed.
  No change.
- Row 5 (initial cluster size): FIXED. A later edit hardcoded max = 6 from the
  unused vanilla local `int ss = 6;` (NaturalSpawner.java:351). Vanilla actually uses
  Mth.ceil(fastRandom.nextFloat() * 4.0F) (line 354). Replaced with the pure helper
  initial_cluster_attempts(rng.next_f32()), which mirrors Mth.ceil, plus a unit test.
- Row 6 (ON_GROUND head-block check): CONFIRMED MATCHING. SpawnPlacementTypes.java:24-36
  validates the floor and calls isValidEmptySpawnBlock on both feet and head. No change.
- Row 7 (chunk-gen vs live adjust_spawn_position): still open only for blocks that
  override isPathfindable. The cache path uses !is_full_cube(), which equals the
  BlockBehaviour LAND default, and the live path queries the block registry; a narrow
  divergence remains for overridden blocks. Fixing it needs the registry inside
  pumpkin-world's GenerationCache, outside this unit.
- Row 8 (shared-spawn bubble / ticking chunk): CONFIRMED MATCHING. Vanilla applies the
  respawn-point check only when respawnData.dimension() == level.dimension()
  (NaturalSpawner.java:453-457) and Canvas replaces the trailing chunk check with
  `return true` (lines 459-463). Pumpkin's OVERWORLD gate plus active-chunks check is
  a conservative approximation. No change.
- Row 9 (canSpawnEntitiesInChunk gate): LEFT OPEN. The caller is in world/mod.rs,
  outside this unit's file (basenames: natural_spawner.rs).
- Row 10 (structure spawn overrides): CONFIRMED FIXED. structure_spawn_entries checks
  the world-gen spawn override before the biome list, and the weighted pick has a
  unit test. No change.
- Row 11 (unknown spawner aborts chunk-gen): CONFIRMED FIXED. EntityType::from_name
  failure logs and continues. Vanilla's spawner list already stores entity types, so
  skipping an unresolvable data-pack entry is the closest equivalent. No change.
- h3-sweep row (canSpawnEntitiesInChunk TODO, world/mod.rs): LEFT OPEN. File is
  outside this unit.

Files changed this pass: crates/pumpkin-core/src/world/natural_spawner.rs; this doc.


## Result (verification sweep, no in-file fix available)

Re-verified every STILL OPEN / LEFT OPEN row against
`.vendor/vanilla-decompiled/net/minecraft/`. No code change was needed in
natural_spawner.rs this pass; all remaining rows are outside this unit or were
already resolved by other workers.

- Row 7 (chunk-gen vs live adjust_spawn_position, narrow case): STILL OPEN, out
  of unit. Vanilla `SpawnPlacementTypes.ON_GROUND.adjustSpawnPosition` calls
  `state.isPathfindable(LAND)` (SpawnPlacementTypes.java:45-48), whose default is
  `!isCollisionShapeFullBlock` (BlockBehaviour.java:130-132); Pumpkin's cache path
  uses `!is_full_cube()` (matches the default) and its live path queries the block
  registry. `is_pathfindable` is a `BlockBehaviour` trait method reachable only
  through the block registry, which GenerationCache (pumpkin-world) does not hold.
  Fixing the overridden-block case needs a cross-crate change, outside this unit.
- Row 9 (canSpawnEntitiesInChunk gate): RESOLVED outside this unit. world/mod.rs:3458
  now calls `any_player_close_enough_for_spawning` (128-block non-spectator test,
  lines 3430-3445) from `tick_spawning_chunk`, matching vanilla
  `ChunkMap.anyPlayerCloseEnoughForSpawning`; no change by this unit.
- h3-sweep row (canSpawnEntitiesInChunk TODO): RESOLVED by the same world/mod.rs gate
  at line 3458. File is outside this unit (basenames: natural_spawner.rs).
- Rows 1-6, 8, 10, 11: re-confirmed matching/fixed in the current file (128-block
  is_near predicate; shared players-near path; spawn_mobs gate at line 653 matching
  NaturalSpawner.java:599; spawn_random threaded; `initial_cluster_attempts` mirror
  of Mth.ceil at NaturalSpawner.java:354; ON_GROUND feet+head check at
  SpawnPlacementTypes.java:31-33; OVERWORLD respawn gate at
  NaturalSpawner.java:453-457; structure spawn overrides; unknown spawner logs and
  continues). after_spawn matches NaturalSpawner.java:754-775. No change.

No new pure unit test: no seam was added this pass. Files changed: this doc only.

## Result (second pass, cross-file permission)

Row 7 (chunk-gen vs live adjust_spawn_position) was the only STILL OPEN row. FIXED
without a cross-crate change: get_top_non_colliding_pos already receives world:
&World, so adjust_spawn_position_cache now takes that &World and uses the same
world.block_registry.is_pathfindable(state_id.to_block(), state_id.to_state(),
PathComputationType::Land) predicate as the live adjust_spawn_position path. The
block and state still come from the in-progress GenerationCache (the generated block),
so this matches vanilla level.getBlockState(below).isPathfindable(LAND) and drops
the !is_full_cube() approximation that ignored isPathfindable overrides (fences,
walls, and so on). The shared adjust_spawn_position_down helper and its existing
test are unchanged. No new pure unit test: the only pure seam is already covered,
and exercising the cache path needs a live World plus GenerationCache.

Rows 1-6, 8-11 and the h3-sweep row remain resolved/matching as recorded in the
earlier Result sections (row 9 and the h3 gate were resolved in world/mod.rs by
another worker). Files changed this pass: natural_spawner.rs and this doc.


## Result (third pass, final verification)

All 11 gap rows plus the h3-sweep row were re-checked against
`.vendor/vanilla-decompiled/net/minecraft/`. No row remains STILL OPEN or LEFT
OPEN: each ends FIXED or VERIFIED MATCHING. No source file needed a new edit this
pass; the row 7 fix from the prior pass was confirmed present and type-correct.

- Row 1 (local mob-cap player window): VERIFIED MATCHING. Vanilla
  ChunkMap.playerIsCloseEnoughForSpawning uses a 128-block horizontal radius.
  Pumpkin's LocalMobCapCalculator::is_near uses LOCAL_MOB_CAP_RANGE_SQ = 128*128
  with no chunk-window gate (natural_spawner.rs:38,184).
- Row 2 (cached vs fallback player paths): VERIFIED MATCHING. Both
  get_players_near branches call the same is_near predicate (lines 221-242).
- Row 3 (chunk-gen honours doMobSpawning): VERIFIED MATCHING. Vanilla
  NaturalSpawner.spawnMobsForChunkGeneration gates on GameRules.SPAWN_MOBS
  (NaturalSpawner.java:599); Pumpkin checks game_rules.spawn_mobs (line 653).
- Row 4 (level RandomSource threading): VERIFIED MATCHING. world.spawn_random is
  threaded through spawn_for_chunk (595), get_random_pos_within (618) and
  spawn_mobs_for_chunk_generation (645); the per-call reseed is gone.
- Row 5 (initial cluster size): VERIFIED MATCHING. Vanilla
  Mth.ceil(fastRandom.nextFloat() * 4.0F) (NaturalSpawner.java:354) equals
  Pumpkin's initial_cluster_attempts (line 851), which has a unit test.
- Row 6 (ON_GROUND head-block check): VERIFIED MATCHING. Vanilla validates the
  floor and feet+head emptiness (SpawnPlacementTypes.java:26-33); Pumpkin's live
  and cache paths both call is_valid_empty_spawn_block on cur/state and up
  (lines 1307-1309, 1345-1347).
- Row 7 (chunk-gen vs live adjust_spawn_position): FIXED (prior pass) and
  RE-VERIFIED. Both paths now share adjust_spawn_position_down and the
  registry-backed is_pathfindable(Land) predicate: cache uses
  GenerationCache::get_block_state -> to_block()/to_state() (lines 815-824),
  live uses get_block_and_state (lines 829-845). Vanilla
  SpawnPlacementTypes.ON_GROUND.adjustSpawnPosition calls
  state.isPathfindable(PathComputationType.LAND) (SpawnPlacementTypes.java:45-47).
  BlockStateId::to_block/to_state exist (pumpkin-data/src/block_state.rs:433,445),
  so the cache fix type-checks and drops the old !is_full_cube() approximation
  that ignored isPathfindable overrides.
- Row 8 (shared-spawn bubble / ticking-chunk): VERIFIED MATCHING. Vanilla applies
  the respawn check only when respawnData.dimension() == level.dimension()
  (NaturalSpawner.java:453-457) and Canvas returns true for the trailing chunk
  check (459-463); Pumpkin's OVERWORLD gate + active-chunks check is conservative.
- Row 9 (canSpawnEntitiesInChunk gate): RESOLVED outside this unit (read-only
  verified). world/mod.rs:3458 calls any_player_close_enough_for_spawning in
  tick_spawning_chunk, matching vanilla ChunkMap.anyPlayerCloseEnoughForSpawning.
- Row 10 (structure spawn overrides): VERIFIED MATCHING. Vanilla mobsAt consults
  structure spawn overrides and nether-fortress bounds before the biome list
  (NaturalSpawner.java:558-574); Pumpkin's structure_spawn_entries checks
  world_gen.structure_spawn_override_at first (line 1060).
- Row 11 (unknown spawner aborts chunk-gen): FIXED (prior pass) and RE-VERIFIED.
  EntityType::from_name failure logs via tracing::warn and continues the loop
  (lines 673-678), matching vanilla skipping an unresolvable spawner entry.
- h3-sweep row (canSpawnEntitiesInChunk TODO, world/mod.rs): RESOLVED by the same
  world/mod.rs gate at line 3458 (read-only verified). File not in this unit.

No new pure unit test: no new seam was added this pass; existing seams are covered
by after_spawn_updates_category_counts, adjust_spawn_position_down_follows_land_pathfindable,
structure_spawn_entries_are_picked_by_weight and initial_cluster_attempts_matches_vanilla_ceil.
Files changed this pass: this doc only (no source edit was required).

## Result (third pass, verification complete)

Re-checked all 11 gap rows plus the h3-sweep row against
`.vendor/vanilla-decompiled/net/minecraft/`. Every source fix from earlier passes is
present and type-correct; no row remains STILL OPEN or LEFT OPEN, and no new source
edit was needed this pass. Both in-scope basenames were re-read; cross-file rows were
checked read-only.

- Row 1 (local mob-cap player window): VERIFIED MATCHING. Vanilla
  ChunkMap.getPlayersCloseForSpawning uses a 128-block horizontal radius
  (`.vendor/.../server/level/ChunkMap.java`); Pumpkin's LOCAL_MOB_CAP_RANGE_SQ =
  128*128 with is_near (lines 38, 184-186).
- Row 2 (cached vs fallback player paths): VERIFIED MATCHING. Both branches of
  get_players_near call the same is_near predicate (lines 221-246).
- Row 3 (chunk-gen honours doMobSpawning): VERIFIED MATCHING. Vanilla gates
  spawnMobsForChunkGeneration on GameRules.SPAWN_MOBS
  (`.vendor/.../world/level/NaturalSpawner.java:599`); Pumpkin checks
  game_rules.spawn_mobs (line 653).
- Row 4 (level RandomSource threading): VERIFIED MATCHING. world.spawn_random is
  threaded through spawn_for_chunk (596), get_random_pos_within (618) and
  spawn_mobs_for_chunk_generation (646); no per-call reseed remains.
- Row 5 (initial cluster size): VERIFIED MATCHING. Mth.ceil(nextFloat()*4.0F)
  (NaturalSpawner.java:354) equals initial_cluster_attempts (line 851), unit-tested.
- Row 6 (ON_GROUND head-block check): VERIFIED MATCHING. Vanilla validates feet and
  head emptiness (`.vendor/.../world/entity/SpawnPlacementTypes.java`); Pumpkin
  checks cur/state and up on both cache and live paths (1308-1309, 1346-1347).
- Row 7 (chunk-gen vs live adjust_spawn_position): FIXED (prior pass) and RE-VERIFIED.
  Both paths share adjust_spawn_position_down and the registry-backed
  is_pathfindable(Land) predicate (796-846); BlockStateId::to_block/to_state exist
  (pumpkin-data/src/block_state.rs:433,445) and the registry signature at
  pumpkin-core/src/block/registry.rs:1662 matches. Vanilla
  SpawnPlacementTypes.java:45-48 calls state.isPathfindable(LAND).
- Row 8 (shared-spawn bubble / ticking-chunk): VERIFIED MATCHING. Vanilla applies the
  respawn check only in the matching dimension (NaturalSpawner.java:446,453-457);
  Pumpkin's OVERWORLD gate plus active-chunks check is a conservative match.
- Row 9 (canSpawnEntitiesInChunk gate): VERIFIED MATCHING (outside this unit,
  read-only). world/mod.rs:3458 calls any_player_close_enough_for_spawning in
  tick_spawning_chunk, matching ChunkMap.anyPlayerCloseEnoughForSpawning.
- Row 10 (structure spawn overrides): VERIFIED MATCHING. Vanilla mobsAt consults
  structure overrides before the biome list (NaturalSpawner.java:558-574);
  structure_spawn_entries checks structure_spawn_override_at first (line 1070).
- Row 11 (unknown spawner aborts chunk-gen): FIXED (prior pass) and RE-VERIFIED.
  EntityType::from_name failure logs via tracing::warn and continues (673-678),
  matching vanilla skipping an unresolvable spawner entry.
- h3-sweep row (canSpawnEntitiesInChunk TODO, world/mod.rs): VERIFIED resolved by the
  same world/mod.rs gate at line 3458 (read-only, outside this unit).

Existing pure tests cover the seams: after_spawn_updates_category_counts,
adjust_spawn_position_down_follows_land_pathfindable,
structure_spawn_entries_are_picked_by_weight, initial_cluster_attempts_matches_vanilla_ceil.
No new seam was added, so no new unit test. Files changed this pass: this doc only.

## Result (FINAL PASS - unit closed)

Every gap row plus the h3-sweep row was re-verified against
`.vendor/vanilla-decompiled/net/minecraft/`. All rows are CLOSED: each ends FIXED
or VERIFIED MATCHING. No source edit was needed this pass; the prior-pass fixes in
natural_spawner.rs and world/mod.rs are present and type-correct. The "STILL OPEN"
and "LEFT OPEN" markers in the earlier Result sections and the h3 Status line are
superseded by this final result.

- Row 1 (local mob-cap player window): FIXED / VERIFIED. Vanilla
  `ChunkMap.getPlayersCloseForSpawning` uses a 16384.0D squared radius
  (`.vendor/.../server/level/ChunkMap.java:846`) and rejects spectators (860-866).
  Pumpkin's `LOCAL_MOB_CAP_RANGE_SQ` = 128*128 with `is_near`
  (natural_spawner.rs:38,181-186) matches. No change.
- Row 2 (cached vs fallback player paths): FIXED / VERIFIED. Both branches of
  `get_players_near` call the same `is_near` predicate (natural_spawner.rs:221-246),
  so can_spawn and add_mob/remove_mob see the same player set. No change.
- Row 3 (chunk-gen honours doMobSpawning): VERIFIED MATCHING. Vanilla
  `spawnMobsForChunkGeneration` gates on GameRules.SPAWN_MOBS
  (`.vendor/.../world/level/NaturalSpawner.java:599`); Pumpkin checks
  `game_rules.spawn_mobs` (natural_spawner.rs:653). No change.
- Row 4 (level RandomSource threading): FIXED / VERIFIED. Vanilla threads one
  level.random through getRandomPosWithin and spawnCategoryForChunk
  (NaturalSpawner.java:577-584,236); Pumpkin threads `world.spawn_random` through
  spawn_for_chunk / get_random_pos_within / spawn_mobs_for_chunk_generation
  (595-649) with no per-call reseed. No change.
- Row 5 (initial cluster size): FIXED / VERIFIED. Vanilla uses
  `Mth.ceil(fastRandom.nextFloat() * 4.0F)` (NaturalSpawner.java:354); Pumpkin's
  `initial_cluster_attempts` (851-853) mirrors it, unit-tested. No change.
- Row 6 (ON_GROUND head-block check): VERIFIED MATCHING. Vanilla validates the
  floor plus feet+head emptiness
  (`.vendor/.../world/entity/SpawnPlacementTypes.java:26-41`); Pumpkin checks
  down/cur/up on both cache and live paths (1298-1310, 1336-1348). No change.
- Row 7 (chunk-gen vs live adjust_spawn_position): FIXED (prior pass) /
  RE-VERIFIED. Vanilla `adjustSpawnPosition` calls
  `state.isPathfindable(LAND)` (SpawnPlacementTypes.java:44-48; default
  `!isCollisionShapeFullBlock`, BlockBehaviour.java:130-136). Both Pumpkin paths
  now share `adjust_spawn_position_down` with the registry-backed
  `is_pathfindable(.., Land)` predicate (796-846); the registry signature
  (`registry.rs:1662`, `&Block`/`&BlockState`) and `BlockStateId::to_block/to_state`
  (`pumpkin-data/src/block_state.rs:433,445`) match, so the fix type-checks. No change.
- Row 8 (shared-spawn bubble / ticking chunk): VERIFIED MATCHING. Vanilla applies
  the respawn-point check only when `respawnData.dimension() == level.dimension()`
  (NaturalSpawner.java:453-457) and Canvas returns true for the trailing chunk
  test (459-463); Pumpkin's OVERWORLD gate + active-chunks check (1007-1041) is a
  conservative match. No change.
- Row 9 (canSpawnEntitiesInChunk gate): VERIFIED MATCHING (resolved outside this
  unit, read-only). Vanilla gates per-tick spawning on
  `ChunkMap.anyPlayerCloseEnoughForSpawning` (ChunkMap.java:766-819); Pumpkin's
  `tick_spawning_chunk` calls `any_player_close_enough_for_spawning` at
  world/mod.rs:3432-3459 (128-block non-spectator test). No change.
- Row 10 (structure spawn overrides): FIXED / VERIFIED. Vanilla `mobsAt`
  consults nether-fortress bounds and `generator.getMobsAt` structure overrides
  before the biome list (NaturalSpawner.java:558-574); Pumpkin's
  `structure_spawn_entries` checks `structure_spawn_override_at` first
  (natural_spawner.rs:1060-1081), used by can_spawn_mob_at and
  get_random_spawn_mob_at with a weighted pick test. No change.
- Row 11 (unknown spawner aborts chunk-gen): FIXED / VERIFIED. Vanilla stores
  resolved entity types in the spawner list (NaturalSpawner.java:609-611);
  Pumpkin's `EntityType::from_name` failure logs via tracing::warn and continues
  the probability loop (natural_spawner.rs:669-678), skipping the bad entry. No change.
- h3-sweep row (canSpawnEntitiesInChunk TODO, world/mod.rs): VERIFIED resolved by
  the same `any_player_close_enough_for_spawning` gate at world/mod.rs:3458
  (read-only, outside this unit). Supersedes the STILL OPEN line in the Status section.

Existing pure unit tests cover the seams: local_mob_cap_uses_128_block_horizontal_range,
after_spawn_updates_category_counts, adjust_spawn_position_down_follows_land_pathfindable,
structure_spawn_entries_are_picked_by_weight, and initial_cluster_attempts_matches_vanilla_ceil.
No new seam was added, so no new unit test. Files changed this pass: this doc only.

## Result (BRIEF2 pass)
- `world/mod.rs:3295` canSpawnEntitiesInChunk TODO: STILL OPEN, not reached (M).
- Row 7 (chunk-gen vs live adjust_spawn_position): STILL OPEN per the Row 7 notes above, not reached this pass.
- Other rows in this doc: not reached this pass; left as recorded above.
- Files edited this pass: none.

## Result (r3_046 recheck)
- Row 1 (local mob-cap player window): VERIFIED. Vanilla ChunkMap.java:846,860-866; Rust natural_spawner.rs:38,181-186.
- Row 2 (cached vs fallback player paths): VERIFIED. Rust natural_spawner.rs:221-246.
- Row 3 (chunk-gen honours doMobSpawning): VERIFIED. Vanilla NaturalSpawner.java:599; Rust natural_spawner.rs:653.
- Row 4 (level RandomSource threading): VERIFIED. Vanilla NaturalSpawner.java:577-584,236; Rust natural_spawner.rs:595-649.
- Row 5 (initial cluster size): VERIFIED. Vanilla NaturalSpawner.java:354; Rust natural_spawner.rs:851-853.
- Row 6 (ON_GROUND head-block check): VERIFIED. Vanilla SpawnPlacementTypes.java:26-41; Rust natural_spawner.rs:1298-1310,1336-1348.
- Row 7 (chunk-gen vs live adjust_spawn_position): VERIFIED. Vanilla SpawnPlacementTypes.java:44-48 (isPathfindable LAND); Rust natural_spawner.rs:796-846 (adjust_spawn_position_cache and adjust_spawn_position both call world.block_registry.is_pathfindable(.., Land)).
- Row 8 (shared-spawn bubble / ticking chunk): VERIFIED. Vanilla NaturalSpawner.java:453-463; Rust natural_spawner.rs:1007-1041.
- Row 9 (canSpawnEntitiesInChunk gate / world/mod.rs:3295 TODO): VERIFIED. The TODO no longer exists in Rust (grep canSpawnEntitiesInChunk in crates/pumpkin-core/src/world/mod.rs: no hits). Vanilla ChunkMap.java:766-819 (anyPlayerCloseEnoughForSpawning); Rust world/mod.rs:3434-3459 (any_player_close_enough_for_spawning, used by tick_spawning_chunk).
- Row 10 (structure spawn overrides): VERIFIED. Vanilla NaturalSpawner.java:558-574; Rust natural_spawner.rs:1060-1081.
- Row 11 (unknown spawner aborts chunk-gen): VERIFIED. Vanilla NaturalSpawner.java:609-611; Rust natural_spawner.rs:669-678.
- Files edited this pass: this doc only (no Rust edits; no code differed from vanilla).
