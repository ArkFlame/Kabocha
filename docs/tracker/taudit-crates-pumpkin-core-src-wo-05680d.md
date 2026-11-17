# Audit: crates/pumpkin-core/src/world/spawn_tests.rs (coverage=AUDIT)

Tracker note said "2 TODO/FIXME/vanilla markers"; both are only test names containing
"vanilla" (spawn_tests.rs:164, :227). There are no TODO/FIXME comments in the file.

The audited file is the test suite for `crates/pumpkin-core/src/world/player_spawn_finder.rs`,
a port of Paper's PlayerSpawnFinder. Paper/vanilla sources are not present in this worktree
(no `.vendor/`) and network access is blocked, so items below that need the reference source
are recorded instead of guessed.

## Fixed

- player_spawn_finder.rs:267 (was 269) - ceiling dimensions used a hardcoded
  `DEFAULT_SPAWN_HEIGHT = 64` for noise/custom generators. Vanilla
  `NoiseBasedChunkGenerator#getSpawnHeight` returns `settings.seaLevel() + 1`
  (overworld 64, nether 33, end 1), so the nether column search started 31 blocks too high
  and could accept floors vanilla rejects. Now uses `self.sea_level + 1`; test
  `ceiling_dimensions_start_the_spawn_column_at_the_generator_sea_level` fails with the old
  constant.

## Remaining gaps

1. player_spawn_finder.rs:39 `get_coprime` - returns 17 for every count > 16. For radius 8
   count is 289 and 17 divides 289, so the step degenerates: only 17 of 289 cells are probed
   (walking the (-8,-8)..(-8,8) line). If Paper searches for a step coprime with the count,
   this needs the gcd loop. Verify against Paper's PlayerSpawnFinder. Size S.
2. player_spawn_finder.rs:267 `WorldGenerator::Custom` - spawn height uses the dimension's
   default noise sea level; a datapack custom generator's own sea level is not exposed by
   `CustomChunkGenerator`. Size M.
3. player_spawn_finder.rs:65 `can_be_collided_with` - the null-source entity query must match
   vanilla `EntitySelector` + `Entity::canBeCollidedWith` (boat removed check, shulker alive,
   happy ghast adult/alive/still-timeout). Values look right but are unverified against
   decompiled vanilla. Size S.
4. player_spawn_finder.rs:230 `release_spawn_search_chunks` - removes `loaded_chunks` and
   `chunks_with_scheduled_ticks` entries directly instead of dropping the tickets the search
   added. A chunk loaded by another system between the load and the release (portal, plugin,
   /forceload) can be evicted. Size S/M.
5. player_spawn_finder.rs:289 - OCEAN_FLOOR is inferred from WORLD_SURFACE plus the 26.3
   motion-blocking tag instead of a stored OCEAN_FLOOR heightmap. Exact only when the surface
   block is the top motion-blocking block; verify with a chunk dump. Size M.
6. spawn_tests.rs:390 - the snow assertion expects `layers=8` to be rejected. Vanilla
   `SnowLayerBlock` LAYERS_TO_SHAPE[8] is `Shapes.block()`, i.e. a valid spawn floor. Check
   Pumpkin's generated snow collision shape; if it is full, the assertion (not the code) is
   wrong. Size S.
7. spawn_tests.rs - no test covers the `defaultgamemode = adventure` branch of
   `get_safe_player_spawn_position`, because the fixture has no `Server` Weak. Needs a server
   fixture. Size M.
8. player_spawn_finder.rs:140 - border distance clamps with `portal_teleport_boundary`
   (configurable). Confirm Paper's distance-to-border helper uses the same bound. Size S.

## Verified as matching

- fixup_spawn_height_at up-then-down-once ordering, best-effort fallback, bottom/top bounds.
- Spawn column: 26.3 motion-blocking tag (not legacy solid predicate), fluid break, full top
  face via union-of-shapes (`is_face_full_up`), waterlogged/partial faces rejected.
- Spawn collision box uses PLAYER spawn AABB at x+0.5/z+0.5 expanded by 1e-7, block shapes and
  entity collisions, chunk release after a failed search.
