# Parity rows for crates/pumpkin-core/src/block/entities/mob_spawner.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-en-6f030b** [partial/OWN-OPEN] own audit: 7 open, 14 fixed, 3 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-en-6f030b.md) docs/tracker/taudit-crates-pumpkin-core-src-en-6f030b.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)

## Result

Third pass. The single row in this unit (`own/taudit-crates-pumpkin-core-src-en-6f030b`, "7 open" snapshot) is closed VERIFIED. The audit doc it points to has no remaining STILL OPEN row: all open gaps are FIXED across its earlier Result sections, and the walk-target / checkSpawnRules gap (row 18) is verified for this file below. No source change was required, so no .rs file was edited and no rustfmt was needed.

- Row 18 `PathAwareEntity::can_spawn` / `Mob.checkSpawnRules`, applied to mob spawners: VERIFIED FIXED. Vanilla `BaseSpawner.serverTick` at `.vendor/vanilla-decompiled/net/minecraft/world/level/BaseSpawner.java:246-251` calls `mob.checkSpawnRules(level, EntitySpawnReason.SPAWNER)` and `mob.checkSpawnObstruction(level)` on the freshly created mob before `finalizeSpawn`. The only instance override of `checkSpawnRules` is `PathfinderMob.checkSpawnRules` = `getWalkTargetValue(blockPosition(), level) >= 0.0F` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/PathfinderMob.java:30-33`). Pumpkin's `tick_spawner` applies the same per-mob gate: `block/entities/mob_spawner.rs:237-242` `is_some_and(|mob| get_walk_target_value(&mob.get_entity().block_pos.load()) < 0.0) -> continue` before `finalize_spawn`. Pumpkin's default `get_walk_target_value` is `0.0` for non-animals, so the gate is a no-op for non-pathfinder mobs, matching vanilla's default `Mob.getWalkTargetValue = 0.0`.
- The rest of `tick_spawner` matches vanilla `serverTick` (`.vendor/vanilla-decompiled/net/minecraft/world/level/BaseSpawner.java:160-311`): player-proximity gate, negative-delay reset, positive-delay decrement, spawn-position formula (`:190-194`), `noCollision(getSpawnAABB)` -> `is_space_empty(get_spawn_bounding_box)` (`:196`), entity-type `SpawnPlacements.checkSpawnRules` -> `check_spawn_rules(..., Spawner)` (`:207`), per-attempt nearby recount with AABB `inflate(spawnRange)` -> `search_radius = spawn_range + 0.5` (`:235-241`), rotation (`:246` -> `:220-221`), `finalizeSpawn` (`:256` -> `:245`), entity event 20 on `spawnAnim` (`:284` -> `:252-257`), and `PARTICLES_MOBBLOCK_SPAWN` level event (`:281` -> `:258`). NBT defaults match the vanilla defaults (`:43-49`).
- No row needs generated data, WIT changes or a cross-crate refactor; no BLOCKED-DATA. No pure unit-test seam: the walk-target gate is a one-line comparison on a freshly created `Mob` requiring a live `World`, so no isolated test was added.
- Compile-pitfall re-read: no `.rs` file changed, so no imports, helpers, macro paths, struct initializers, clones, `unwrap`/`expect`/`panic`, println or em dashes to check.

Status: PASS (verified completion, no source change needed). Commands: none (no `.rs` file changed); cargo/git/rustfmt intentionally not run per worker contract.
