# world / Player Spawn Finder (`crates/pumpkin-core/src/world/player_spawn_finder.rs`)

Status: AUDITED (2026-11-17, second pass). Coverage row: `audit` / `crates/pumpkin-core/src/world/player_spawn_finder.rs`.
Vanilla classes: `server/level/PlayerSpawnFinder.java`, `ChunkGenerator#getSpawnHeight`,
`FlatLevelSource#getSpawnHeight`, `NoiseBasedChunkGenerator` (no override),
`EntitySelector.CAN_BE_COLLIDED_WITH`, `ServerPlayer#adjustSpawnLocation`.
Compared against the decompiled Canvas tree in `/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft`.
Companion tests: `crates/pumpkin-core/src/world/spawn_tests.rs`.

Tracker note "5 TODO/FIXME/vanilla markers" is stale: the file has no TODO/FIXME. The five hits are the word
"vanilla" in explanatory comments and test names (lines 87, 97, 121, 315, 354).

## Fixed in this pass

1. `:269` ceiling-world spawn height for noise/custom generators used `self.sea_level + 1` (33 in the nether,
   datapack-dependent elsewhere). Vanilla `ChunkGenerator#getSpawnHeight` returns the fixed `64` and
   `NoiseBasedChunkGenerator` does not override it, so any non-flat ceiling dimension scans from 64, not from
   sea level. Replaced with `DEFAULT_GENERATOR_SPAWN_HEIGHT = 64`. No automated test: the branch needs a full
   ceiling-dimension `World` fixture (see gap 4 of the companion doc `taudit-crates-pumpkin-core-src-world-spaw.md`).

## Remaining gaps

| Location | Vanilla | Gap | Fix | Size |
|---|---|---|---|---|
| `:98-108`, `:118-129` | `ServerPlayer#adjustSpawnLocation` (`:688-697`) calls `PlayerSpawnFinder.findSpawn` on every respawn, and `findSpawn` (`:50-68`) always randomises inside the radius (only `ADVENTURE` skips it) | Pumpkin returns the stored `/setworldspawn` Y, and the raw suggestion when standable, before the candidate search; `spawn_tests.rs:271-281` encodes this. The in-code comment "vanilla only runs PlayerSpawnFinder while creating the world" is wrong for 26.3 | Drop both early returns and let the candidate search run, keeping a single `fixup_spawn_height_at` fallback; update `spawn_tests.rs` | M |
| `:109-116` | Adventure gate is `level.serverLevelData.getGameType()` (`PlayerSpawnFinder.java:51`, `// CraftBukkit`), i.e. the level's stored game type | Uses the server-wide `defaultgamemode`, so `/defaultgamemode adventure` changes the spawn search while a level created in adventure mode and later switched to survival does not | Add a game-type field to `LevelData` (`crates/pumpkin-world/src/world_info/mod.rs:32` has none) and read it here | M |
| `:296` | `for (y = topY + 1; y >= minY; y--)` with `topY` from the generator/heightmap | `top + 1` can exceed `get_top_y()`; Pumpkin returns air above the build limit, so this is unobservable today | Clamp the upper bound with `get_top_y()` | S, cosmetic |
| `:255-295` | `surface <= topY && surface > getHeight(OCEAN_FLOOR)` | Pumpkin stores only `WORLD_SURFACE` and emulates the `OCEAN_FLOOR` rejection from the surface block's motion-blocking tag; equivalent for real heightmaps, unverified for a cached heightmap that lags the blocks | Keep; only revisit if a stale-heightmap case is reproduced | S |

## Verified matching (no change)

- `findSpawn` radius: `max(0, RESPAWN_RADIUS)`, clamp to `floor(distanceToBorder)`, forced to 1 when
  `distanceToBorder <= 1` (`:24-31`, test `:353`); the border distance is computed from the clamped border box
  like `WorldBorder#getDistanceToBorder`.
- Candidate schedule: `count = min((2r+1)^2, 1024)`, `coprime = count - 1` for `count <= 16` else `17`,
  `value = (offset + coprime * i) % count`, `deltaX = value % (2r+1)`, `deltaZ = value / (2r+1)`
  (`PlayerSpawnFinder.java:44-47,70-77`; `:21-61`, tests `:314-361`). The `side` clamp for huge radii only
  differs where vanilla would overflow `int`.
- Exhausted search: vanilla schedules one final candidate on the unmodified suggestion and returns
  `fixupSpawnHeight` (`:82-86`); `fixup_spawn_height_at` matches `fixupSpawnHeight` step for step (up, one down,
  down while free, one up) (`:216-227`).
- Non-ceiling `topY`: scanning down from `WORLD_SURFACE` for the motion-blocking tag or any non-empty fluid
  yields the inclusive top block, so the loop start equals vanilla's `topY`; vanilla's extra iteration sits above
  the topmost motion-blocking block, where a face-full-up block cannot exist (`:270-282`).
- Flat ceiling worlds: `minY + min(height, layers.size())`, layer count not summed thickness
  (`FlatLevelSource.java:67-69`; `:257-266`).
- Candidate chunk load before the check, and `None` when the column has no floor (`:239-252`,
  `PlayerSpawnFinder.java:159-192`).
- Space test: `PLAYER` box at `(x + 0.5, y, z + 0.5)`, epsilon-expanded query over neighbouring blocks, water and
  lava treated as full cubes because `EmptyWithFluidCollisions` does not replace every aquatic collision shape
  (`:172-206`).
- Null-source entity query: spectators, removed entities and the three `canBeCollidedWith` overrides
  (`AbstractBoat` always, `Shulker#isAlive`, adult `HappyGhast` alive on a still timeout) (`:63-83`).
- Adventure default skips the scan and only fixes the height (`:163-169`), matching `findSpawn`'s early return.
- Spawn-search chunks are released unless a player already watches them (`:229-237`), a Pumpkin-only need.
