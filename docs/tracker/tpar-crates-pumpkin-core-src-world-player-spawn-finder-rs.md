# Parity rows for crates/pumpkin-core/src/world/player_spawn_finder.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-wo-8a8c0c** [partial/OWN-OPEN] own audit: 5 open, 2 fixed, 5 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-wo-8a8c0c.md) docs/tracker/taudit-crates-pumpkin-core-src-wo-8a8c0c.md
- **own/taudit-crates-pumpkin-core-src-world-spaw** [partial/OWN-OPEN] own audit: 6 open, 7 fixed, 1 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-world-spaw.md) docs/tracker/taudit-crates-pumpkin-core-src-world-spaw.md

## Result (third pass, final - verified against decompiled vanilla)

The task path `.vendor/vanilla-decompiled/net/minecraft/` is absent in this worktree. Verification used the decompiled V_26_3 reference tree cited by both child audits at
`/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft/`. Every row is already closed; no source edit was required this pass.

Child audit `taudit-crates-pumpkin-core-src-wo-8a8c0c.md`:
- Row 1 (spawn height, `player_spawn_finder.rs:219-230`): VERIFIED MATCHING. `world/level/chunk/ChunkGenerator.java:568-570` returns a constant 64 from `getSpawnHeight`; a tree-wide grep finds only two overrides, the base and `world/level/levelgen/FlatLevelSource.java:67-69` (`getMinY() + min(getHeight(), getLayers().size())`). `NoiseBasedChunkGenerator` has no override, so the `Noise | Custom => DEFAULT_SPAWN_HEIGHT` (64) arm and the `Flat => bottom_y + min(layers.len(), height)` arm both match. The stale `sea_level + 1` note from an earlier sweep is no longer present in the file.
- Row 2 (scan range, `player_spawn_finder.rs:257`): VERIFIED MATCHING. `server/level/PlayerSpawnFinder.java:179` is `for (y = topY + 1; y >= level.getMinY(); y--)`, with no top clamp; `(get_bottom_y()..=top + 1).rev()` walks the identical range.
- Row 3 (`can_be_collided_with`, `player_spawn_finder.rs:66-85`): VERIFIED MATCHING. A tree-wide grep finds exactly three non-base `canBeCollidedWith` overrides: `world/entity/vehicle/boat/AbstractBoat.java:129` (returns true), `world/entity/monster/Shulker.java:496` (isAlive), and `world/entity/animal/happyghast/HappyGhast.java:574` (with the null source at `world/level/EntityGetter.java:112` this reduces to `!isBaby && isAlive && isOnStillTimeout`); base `world/entity/Entity.java:3470` returns false and `world/entity/EntitySelector.java:18` is `!spectator && canBeCollidedWith(null)`. Pumpkin's three arms match; no minecart/armor stand/end crystal override exists.

Child audit `taudit-crates-pumpkin-core-src-world-spaw.md`:
- Stored-Y path (`player_spawn_finder.rs:89-141`): FIXED. `server/level/PlayerSpawnFinder.java:50-68,70-87` has no stored-Y early return; with radius 0 the single candidate resolves through `getLevelRespawnPos` (column surface) and the suggestion's Y is only the final fallback (`:82-86`). Current `get_safe_player_spawn_position` has no standable/suggestion early return; covered by `zero_radius_uses_the_column_surface_not_the_stored_height`.
- Adventure gate (`player_spawn_finder.rs:98`): FIXED. `PlayerSpawnFinder.java:51` reads the level's own `serverLevelData.getGameType()` (level.dat); current code reads `self.level_info.load().game_type`, the level's own persisted `GameType`. Covered by `adventure_mode_fixes_height_on_the_suggestion_not_the_column`.

The other "Remaining gaps" entries in the child audits are coverage-only (flat-generator fixture, chunk-dedup test, `isRemoved` spurious-block case) and are not correctness rows; none are STILL OPEN.

No `.rs` file touched; rustfmt not needed. Status: PASS, fixed=0, open=0.
