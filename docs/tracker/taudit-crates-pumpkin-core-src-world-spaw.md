# audit / crates/pumpkin-core/src/world/spawn_tests.rs

Target: JavaMinecraftVersion::V_26_3. Reference: `.vendor` absent, compared against the
decompiled Paper/Canvas tree in `/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft`
(`server/level/PlayerSpawnFinder.java`, `server/level/ServerPlayer.java#adjustSpawnLocation`,
`world/level/portal/TeleportTransition.java#findAdjustedSharedSpawnPos`,
`world/level/CollisionGetter.java`, `world/level/levelgen/FlatLevelSource.java`,
`world/level/chunk/ChunkGenerator.java#getSpawnHeight`, `world/level/levelgen/Heightmap.java`).

The two tracker "markers" are the word `vanilla` in the test names at `spawn_tests.rs:164` and
`spawn_tests.rs:227`; the file has no TODO/FIXME. It is a test module for
`crates/pumpkin-core/src/world/player_spawn_finder.rs`, so the audit was done on that source.

Verified faithful against vanilla (no change):

- `spawn_search_radius` / `spawn_candidate_offsets`: `PlayerSpawnFinder.findSpawn` radius clamp
  (`distToBorder <= 1 -> 1`) and the `scheduleNext` coprime walk are equivalent, including the
  1024-attempt cap and the `side` cap for huge radii.
- `get_level_respawn_pos`: `MOTION_BLOCKING` tag + fluid predicate, the `WORLD_SURFACE` above
  `OCEAN_FLOOR` rejection, fluid break, `Block.isFaceFull(..., UP)` and the cave-world
  `ChunkGenerator.getSpawnHeight` all match.
- `no_collision_no_liquid_at`: `noCollision(null, box, true)` = block+liquid, entity, border;
  border is `true` for a null source (`CollisionGetter.noBorderCollision`), and liquid shapes
  only come from `LiquidBlock` (`context.alwaysCollideWithFluid()` -> full block), so waterlogged
  and aquatic blocks keep their own shape.
- `can_be_collided_with`: vanilla `EntitySelector.CAN_BE_COLLIDED_WITH` =
  `!isSpectator && canBeCollidedWith(null)`; only `AbstractBoat` (always true), `Shulker`
  (`isAlive`) and `HappyGhast` (`!isBaby && isAlive && isOnStillTimeout`) override it.

Fixed in this pass:

- `player_spawn_finder.rs:243` - ceiling-world flat spawn height used the summed layer
  thickness; vanilla `FlatLevelSource.getSpawnHeight` is
  `minY + min(height, settings.getLayers().size())` (layer count), so a default flat world with
  layers `[1,2,1]` scanned from the wrong height.

Remaining gaps:

| Location | Vanilla | Gap | Fix | Size |
|---|---|---|---|---|
| `player_spawn_finder.rs:99-109` | `PlayerSpawnFinder.findSpawn` has no stored-Y path: with `RESPAWN_RADIUS=0` the single candidate goes through `getLevelRespawnPos`, i.e. the column surface | Pumpkin keeps the stored spawn Y when `respawn_radius <= 0` (`fixup_spawn_height_up_at`); `spawn_tests.rs:271-281` encodes the divergence | Drop the branch and always run the candidate search, or keep it deliberately and document the deviation | S |
| `player_spawn_finder.rs:119-130` | `findSpawn` always randomises inside the radius before testing any column | Returns the raw suggestion whenever it is standable, so respawns do not spread even with the default radius 10 | Remove the early return once the stored-Y policy above is decided | S |
| `player_spawn_finder.rs:110-117` | Adventure gate is `serverLevelData.getGameType() == ADVENTURE` (level.dat) | Uses the server-wide `defaultgamemode`; `/defaultgamemode` does not rewrite an existing level's stored game type | Read the level's own game type from `level_info` | S |
| `player_spawn_finder.rs:241-253` | Flat `getSpawnHeight` | No test: the fixture only builds a default (noise) world, so the fixed branch stays uncovered | Add a flat-generator fixture and assert the resulting surface | S/M |
| `player_spawn_finder.rs:67` | `EntitySelector.CAN_BE_COLLIDED_WITH` has no `isRemoved` check; a just-discarded entity can still sit in the entity section for the rest of the tick | Pumpkin excludes removed entities outright (`spawn_tests.rs:202-224` asserts that) | Only relevant if a same-tick spurious block is observed; leave unless reproduced | S |
| `player_spawn_finder.rs:225-237` | `scheduleCandidate` loads the candidate chunk at `ChunkStatus.FULL` before every check | The `loaded_chunks` dedup is untested | Test a second call reusing the set | S |
