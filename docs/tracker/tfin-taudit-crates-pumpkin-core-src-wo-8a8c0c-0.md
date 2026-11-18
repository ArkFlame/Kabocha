# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-wo-8a8c0c.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: player_spawn_finder.rs spawn_tests.rs

FIXED: NoiseBasedChunkGenerator and Custom spawn height uses sea_level + 1 (world/player_spawn_finder.rs:264-265). STILL OPEN: scan range row 2 and can_be_collided_with row 3 not re-read.
- STILL OPEN row 2 (scan range, `player_spawn_finder.rs:293`): VERIFIED MATCHING, no change. Vanilla
- STILL OPEN row 3 (`can_be_collided_with`, `player_spawn_finder.rs:63-83`): VERIFIED MATCHING, no change. A
it is not a STILL OPEN row and `spawn_tests.rs:404` encodes the current value, so correcting it needs a

## Result

All rows CLOSED (FINAL PASS 2026-11-18). No source edit required: the code already matches vanilla on every row.

- Row 1 (spawn height, `player_spawn_finder.rs:217-230`): VERIFIED MATCHING. `.vendor/vanilla-decompiled/net/minecraft/world/level/chunk/ChunkGenerator.java:568-570` returns a constant 64 from `getSpawnHeight`, and a tree-wide grep shows no `getSpawnHeight` override in `NoiseBasedChunkGenerator` or `CustomChunkGenerator`, so the Noise/Custom arm using `DEFAULT_SPAWN_HEIGHT` (64) at `:229` is correct for every sea level. The ceiling/Flat arm at `:223-226` matches `.vendor/vanilla-decompiled/net/minecraft/world/level/levelgen/FlatLevelSource.java:67-69` (`getMinY() + min(getHeight(), layers.size())`). The stale "sea_level + 1" note in the sweep status is no longer true in the file. `spawn_tests.rs:404-419` pins the constant-64 ceiling case.
- Row 2 (scan range, `player_spawn_finder.rs:257`): VERIFIED MATCHING. `.vendor/vanilla-decompiled/net/minecraft/server/level/PlayerSpawnFinder.java:179` runs `for (int y = topY + 1; y >= level.getMinY(); y--)` with no top clamp; Pumpkin's `(get_bottom_y()..=top + 1).rev()` walks the identical range down to min Y. `spawn_tests.rs:284` pins the top boundary.
- Row 3 (`can_be_collided_with`, `player_spawn_finder.rs:65-85`): VERIFIED MATCHING. A tree-wide grep finds exactly three non-base `canBeCollidedWith` overrides: `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java:129` (true), `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Shulker.java:496` (isAlive), and `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/happyghast/HappyGhast.java:574`. For the null source at `.vendor/vanilla-decompiled/net/minecraft/world/level/EntityGetter.java:112` the ghast arm reduces to `!isBaby && isAlive && isOnStillTimeout`; the base `.vendor/vanilla-decompiled/net/minecraft/world/entity/Entity.java:3522` returns false. Pumpkin's three arms match; no minecart/armor stand/end crystal override exists. `spawn_tests.rs:203,227` pin it.

No `.rs` file changed and no new pure unit test was warranted (existing tests cover all three rows); rustfmt not needed.
