# Audit: crates/pumpkin-core/src/world/natural_spawner.rs

Coverage AUDIT. Tracker note said "7 TODO/FIXME/vanilla markers"; all 7 grep hits are
the word "vanilla" inside doc comments (lines 79, 288, 607, 885, 990, 1021, 1273).
There is no `TODO`, `FIXME`, `todo!()` or `unimplemented!()` in the file.

Reference: decompiled vanilla `net.minecraft.world.level.NaturalSpawner`,
`SpawnState`, `LocalMobCapCalculator`, `SpawnPlacements`/`SpawnPlacementTypes`;
Paper only adds the `CreatureSpawnEvent` hook, which Pumpkin already fires.

## Fixed in this pass

| file:line | gap | fix |
|---|---|---|
| natural_spawner.rs:806 (pre-fix) | cluster attempt budget was `ceil(rand*4)` (0..4) before a spawner is picked; vanilla `NaturalSpawner.spawnCategoryForPosition` starts each of the 3 clusters with `int j1 = 6;` and only replaces it with the pack size afterwards | use `let mut max = 6;` (commit ab36467e7) |
| natural_spawner.rs:626-629 (pre-fix) | `choose_spawner(...) else { continue; }` spins forever when the spawner list weights sum to 0 (`choose_spawner` returns `None`); vanilla ends the pass when the weighted pick fails | `return` + comment (commit 7be0f9c7a) |

## Remaining gaps (not fixed here)

1. **No structure spawn overrides - L.** `get_random_spawn_mob_at` (natural_spawner.rs:992)
   and `can_spawn_mob_at` (:960) read only `biome.spawners`. Vanilla
   `NaturalSpawner.getMobsAt` unions the biome list with structure-contributed
   `MobSpawnSettings` (`StructureSpawnOverride`/`MOB_SPAWN_OVERRIDES`:
   nether fortress blaze + wither skeleton, ocean monument guardians, swamp hut
   witches, pillager outpost, bastion/bridge piglins). Fix: extract per-structure
   spawn overrides in `pumpkin-codegen` (structures already parsed), add a
   `structures_at(pos)` lookup on `World`, then merge the lists before
   `choose_spawner`. Also makes `can_spawn_mob_at`'s name comparison match vanilla's
   membership test.
2. **Approximate biome lookup - M.** :521, :966, :997 use `level.get_rough_biome(pos)`;
   vanilla uses `level.getBiome(pos)` (exact, blended at borders) for both the
   spawner list and `spawn_costs`. Near biome boundaries Pumpkin can pick spawners
   and costs from the wrong biome. Needs a cheap exact-biome query.
3. **Spawn-point exclusion is Overworld-only - S.** :933 gates the 24-block
   `level.getSharedSpawnPos()` exclusion on `Dimension::OVERWORLD`; vanilla applies
   `level.getSharedSpawnPos().closerToCenterThan(...)` in every dimension (Nether/End
   included, using that level's shared spawn position). Drop the dimension gate after
   confirming `level_info.spawn_*` holds the per-level shared spawn pos.
4. **RNG source for the random chunk position - S/M.** :593 `get_random_pos_within`
   builds a fresh `Xoroshiro::from_seed(get_seed())` per call; vanilla draws x, z and y
   from the shared `ServerLevel.random`. Statistically similar but not reproducible and
   not the vanilla stream; thread the level random through instead.
5. **Local mob cap state is a per-cycle snapshot - M.** :96-280 `LocalMobCapCalculator`
   caches `Arc<[ActiveSpawnPlayer]>` at `for_world` time and `SpawnState` is cloned per
   spawn pass, so player positions, mob counts and `players_near_chunk` are stale for
   the whole 289-chunk cycle; vanilla keeps one calculator on the level and re-queries
   `chunkMap.getPlayersCloseForSpawning(pos)` live each call. Staleness biases the local
   cap (players who move/join mid-cycle), and the cloned `DashMap`s are pure overhead.
6. **Players-near predicates disagree - S.** :214-222 (cached path) requires
   chessboard chunk distance <= 9 *and* 128-block distance; :223-232 (uncached path)
   only checks 128 blocks. Vanilla `ChunkMap.getPlayersCloseForSpawning` filters by
   distance only, so pick one rule; the chessboard gate currently drops players whose
   chunk is > 9 chunks away but whose centre is inside 128 blocks (only possible for
   odd shapes, but the two paths already disagree).
7. **`is_spawn_position_ok` OnGround is stricter than vanilla - M.** :1098-1113
   requires both `pos` and `pos.above()` to pass `is_valid_empty_spawn_block`
   (full cube / liquid / `PREVENT_MOB_SPAWNING_INSIDE` / dangerous-block checks);
   vanilla `SpawnPlacementTypes.ON_GROUND` only validates the block *below*
   (`state.isValidSpawn`) and leaves the cell itself to the AABB/no-collision test plus
   `checkSpawnRules`. Verify against vanilla before loosening: the extra check can veto
   otherwise valid cells (e.g. spawn attempts inside snow layers/tall grass whose state
   is not "empty" by Pumpkin's definition).
8. **Two divergent `adjust_spawn_*` implementations - M.** :745 `adjust_spawn_position_cache`
   moves the position down when the block below is neither a full cube nor a liquid,
   while :764 `adjust_spawn_position` does it when the block below is
   `is_pathfindable(Land)`. Vanilla has a single `SpawnPlacements.adjustSpawnLocation`
   driven by the placement type's `isSpawnPositionOk`; both variants should delegate to
   `is_valid_spawn_floor` so chunk-generation and runtime spawns agree.
9. **Stale biome on the spawn-cost charge - S.** :416 and :443 use
   `base_entity.current_biome` for `biome.spawn_costs`; vanilla evaluates
   `level.getBiome(entity.blockPosition())` at add/remove time, so a mob that walks
   across a biome border should release its charge in the biome it dies in. Removing the
   cached-biome dependency (same lookup as the fix for gap 2) closes it.
10. **Unknown entity id aborts the chunk pass - S.** :637 `EntityType::from_name(...) else { return; }`
    ends all remaining passive spawning for that chunk when a datapack spawner names an
    unloaded entity. Vanilla cannot hit this, so prefer skipping that spawner over
    aborting (careful: `continue` on a single-entry list would loop, so skip the spawner
    *and* the count loop).

## Checked and matching

- `MAGIC_NUMBER = 17*17`, `can_spawn_for_category_global`, `get_filtered_spawning_categories`
  ordering (friendly/monster/persistent flags) match `SpawnState`/`NaturalSpawner`.
- `spawn_for_chunk`'s `y > min_y` equals vanilla's `getY() >= minY + 1`.
- `PointCharge::get_potential_change` (incl. the `dist_sq == 0 -> infinity` case) and the
  `<= energy_budget` test match `PotentialCalculator`; the wide-cast fix is correct.
- `is_valid_spawn_floor` covers every `SpawnFloorPredicate` variant; `is_block_dangerous`
  matches `isBlockDangerous` for the vanilla block set.
- Chunk-generation pack size (`min + nextInt(1 + max - min)`), the 4-attempt retry, the
  clamp to the chunk, and `finalize_spawn`/`CREATURE_SPAWN` reason all match
  `NaturalSpawner.spawnMobsForChunkGeneration`.
- `limit_per_chunk` is the vanilla `Mob.getMaxSpawnClusterSize` value, used at the same
  point in the cluster loop.
- `is_right_distance_to_player_and_spawn_point` 24-block player and spawn-point radii
  (576.0), chunk-local exemption, active-chunk and worldborder checks match vanilla.
- `get_top_non_colliding_pos` correctly uses the per-entity `spawn_restriction.heightmap`
  (`SpawnPlacements` registers one per type) and the has-ceiling downward walk.
