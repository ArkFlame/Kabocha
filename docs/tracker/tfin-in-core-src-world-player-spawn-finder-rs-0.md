# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-world-player-spawn-finder-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

The other "Remaining gaps" entries in the child audits are coverage-only (flat-generator fixture, chunk-dedup test, `isRemoved` spurious-block case) and are not correctness rows; none are STILL OPEN.
spurious-block case) are coverage-only and are not correctness rows; none are STILL OPEN.
Final: 5 rows closed (2 FIXED, 3 VERIFIED MATCHING), 0 STILL OPEN, 0 BLOCKED-DATA. No `.rs` file changed;

## Result
FINAL PASS, all rows closed. The task path .vendor/vanilla-decompiled/net/minecraft/ is absent in this
worktree; verification used the decompiled V_26_3 reference tree at
/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft/ (equivalent .vendor paths in parentheses). Every row was
re-read against that source and against crates/pumpkin-core/src/world/player_spawn_finder.rs this pass. The Rust
already matches on every row, so no source edit was required.

Child audit taudit-crates-pumpkin-core-src-wo-8a8c0c.md (3 rows):
- Row 1 - spawn height (player_spawn_finder.rs:219-230): VERIFIED MATCHING.
  (world/level/chunk/ChunkGenerator.java:568-570) returns a constant 64 from getSpawnHeight and
  NoiseBasedChunkGenerator has no override, so the Noise | Custom => DEFAULT_SPAWN_HEIGHT (64) arm at :229 matches.
  Only (world/level/levelgen/FlatLevelSource.java:67-69) overrides it as
  getMinY() + min(getHeight(), layers.size()), matching the bottom_y + min(layers.len(), height) arm at :223-226.
  Pinned by spawn_tests.rs:421-437 (ceiling_dimensions_use_the_fixed_get_spawn_height).
- Row 2 - scan range (player_spawn_finder.rs:257): VERIFIED MATCHING.
  (server/level/PlayerSpawnFinder.java:179) runs for (int y = topY + 1; y >= level.getMinY(); y--) with no top clamp;
  (get_bottom_y()..=top + 1).rev() walks the identical inclusive range. Pinned by spawn_tests.rs:284.
- Row 3 - can_be_collided_with (player_spawn_finder.rs:66-85): VERIFIED MATCHING. A tree-wide search finds exactly
  three non-base overrides: (world/entity/vehicle/boat/AbstractBoat.java:129) returns true,
  (world/entity/monster/Shulker.java:496) returns isAlive() (health > 0), and
  (world/entity/animal/happyghast/HappyGhast.java:574). For the null source at
  (world/level/EntityGetter.java:112) the ghast reduces to !isBaby && isAlive && isOnStillTimeout; base
  (world/entity/Entity.java:3470) returns false. Pumpkin's three arms match; no minecart, armor stand or end crystal
  override exists in V_26_3, so adding arms would be wrong. Pinned by spawn_tests.rs:203,227.

Child audit taudit-crates-pumpkin-core-src-world-spaw.md (2 rows):
- Stored-Y path (player_spawn_finder.rs:89-141): FIXED / CONFIRMED.
  (server/level/PlayerSpawnFinder.java:50-68,70-87) has no stored-Y path: findSpawn always randomises inside the
  radius and each candidate resolves through getLevelRespawnPos (column surface); the suggestion's Y is only the
  final fallback (scheduleNext else-branch :82-86). get_safe_player_spawn_position has no standable/suggestion early
  return and calls fixup_spawn_height_at(suggestion) only after the candidate loop. Covered by
  zero_radius_uses_the_column_surface_not_the_stored_height (spawn_tests.rs:272-282).
- Adventure gate (player_spawn_finder.rs:98): FIXED / CONFIRMED.
  (server/level/PlayerSpawnFinder.java:51) reads level.serverLevelData.getGameType() (level.dat). Current code reads
  self.level_info.load().game_type, the level's own persisted GameType (pumpkin-world
  world_info/mod.rs:58-59, serialized as GameType in level.dat), so no cross-crate change is needed. Covered by
  adventure_mode_fixes_height_on_the_suggestion_not_the_column (spawn_tests.rs:404-419).

The other Remaining gaps in the child audits (flat-generator fixture, chunk-dedup test, isRemoved spurious-block
case) are coverage-only and are not correctness rows; none are STILL OPEN.

Final: 5 rows closed (5 VERIFIED MATCHING / CONFIRMED FIXED), 0 STILL OPEN, 0 BLOCKED-DATA. No .rs file changed;
rustfmt not needed. Status: PASS, fixed=0 (all already fixed/verified), open=0.
