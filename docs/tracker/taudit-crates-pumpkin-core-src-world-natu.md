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
