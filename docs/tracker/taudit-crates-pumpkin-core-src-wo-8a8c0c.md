# world / Player Spawn Finder (`crates/pumpkin-core/src/world/player_spawn_finder.rs`)

Status: AUDITED (2026-11-17). Coverage row: `audit` / `crates/pumpkin-core/src/world/player_spawn_finder.rs`.
Vanilla classes: `PlayerSpawnFinder`, `ChunkGenerator#getSpawnHeight`, `NoiseBasedChunkGenerator#getSpawnHeight`,
`EntityGetter#getEntities` (null-source `canBeCollidedWith` query), `Entity#canBeCollidedWith`.
Companion tests: `crates/pumpkin-core/src/world/spawn_tests.rs`.

Tracker note "5 TODO/FIXME/vanilla markers" is stale: the file has no TODO/FIXME. The five hits are the word
"vanilla" in explanatory comments and test names (lines 87, 97, 121, 315, 354). No behaviour marker is left in code.

## Fixed in this pass
None: no gap that is both certain and small was found. No decompiled vanilla tree (`.vendor/`, assets datapack) is
present in this worktree, so the three items below are recorded instead of edited.

## Remaining gaps (unverified, need the decompile)
1. `player_spawn_finder.rs:269` maps `WorldGenerator::Noise(_) | WorldGenerator::Custom(_)` to `DEFAULT_SPAWN_HEIGHT`
   (64). Confirm `NoiseBasedChunkGenerator#getSpawnHeight`: if it returns `settings.seaLevel() + 1` instead of
   inheriting `ChunkGenerator#getSpawnHeight`, the value is 64 only for the overworld (sea level 63) and 33 for the
   nether (sea level 32). `NoiseSettings.sea_level` is reachable as `generator.settings.sea_level`, so the fix is a
   one-line `match` arm. Observable only for datapack sea levels and ceiling dimensions (nether/end candidate
   columns). Size S.
2. `player_spawn_finder.rs:297` scans `(get_bottom_y()..=top + 1).rev()`, which can read `get_top_y() + 1` when
   `top == get_top_y()` (non-ceiling dimension with a motion-blocking block on the build limit). Pumpkin returns air
   above the build limit, so this is harmless today, but vanilla clamps the loop at the top. Size S, cosmetic.
3. `player_spawn_finder.rs:65` `can_be_collided_with` enumerates exactly three overrides: boat, shulker with
   `health > 0`, adult happy ghast on a still timeout. Vanilla `Entity#canBeCollidedWith` must be checked for the full
   override list (minecart, armor stand, end crystal are the plausible missing ones); if `AbstractMinecart` overrides
   it, a minecart today does not block the spawn search. Add arms only after confirming the list. Size S.

## Verified matching (no change)
- Attempt schedule: `count = min((2r + 1)^2, 1024)`, `coprime = count - 1` for `count <= 16` else `17`, offsets walk
  `value % side`/`value / side` with wraparound (`:21-62`); unit tests `:314-361` pin the order, the cap and the
  divisor clamp that keeps huge radii inside the attempt budget.
- Radius: `floor()` of the distance to the world border, forced to 1 at `<= 1`, otherwise the spawn-radius gamerule
  clamped into `[0, distance]` (`:25-32`, test `:353`).
- Fixed-Y path: with `respawn_radius <= 0` and a stored Y inside the build limits, the stored height is kept and the
  player is only pushed up out of blocks (`:99-109`); randomisation runs only for a positive radius, matching that
  vanilla runs `PlayerSpawnFinder` at world creation while a `/setworldspawn` Y stays authoritative (`:118-158`).
- Adventure default gamemode skips the chunk scan and only fixes the height (`:110-117`, `:164-169`).
- `fixup_spawn_height_at` moves up, down once, then down while free and returns `.up()` - vanilla's best-effort
  fallback that may stay obstructed (`:217-228`, test `spawn_tests.rs:164`).
- Space test: `Player` standing box at `(x + 0.5, y, z + 0.5)`, epsilon-expanded query over neighbouring blocks so
  overhanging shapes count, and water/lava treated as full cubes because `EmptyWithFluidCollisions` does not replace
  every aquatic collision shape (`:173-207`; tests `spawn_tests.rs:119,139,299,371`).
- Null-source entity query reproduces `canBeCollidedWith`, ignoring spectators and removed entities (`:65-84`, tests
  `spawn_tests.rs:203,227`).
- Height selection uses the 26.3 motion-blocking block tag plus every non-empty fluid, and emulates the OCEAN_FLOOR
  rejection through the surface block's tag because Pumpkin stores only WORLD_SURFACE (`:255-307`; tests
  `spawn_tests.rs:347,371`).
- Ceiling dimensions use the flat generator's summed layer height (capped at the dimension height) (`:257-270`).
- Spawn-search chunks are released unless a player already watches them (`:231-238`).

## Tests
`crates/pumpkin-core/src/world/spawn_tests.rs` already covers radius/border interplay, fluid columns, split
collision shapes, the height fixup boundaries, removed collidable entities, the null-source entity filter, the
26.3 motion heightmap tag and the highest-floor case. No test was added in this pass because no code changed.
