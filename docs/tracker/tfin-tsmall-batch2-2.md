# Final rows from docs/tracker/tsmall-batch2.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: sculk_catalyst.rs

| taudit-crates-pumpkin-core-src-block-enti : 1 (`entities/sculk_catalyst.rs:125`) | VERIFIED | Bloom spawns two SCULK_SOUL particles server-side via `world.spawn_particle` (`sculk_catalyst.rs:175-187`), vanilla `SculkCatalystBlockEntity.java:124`. Client-side rendering is not checkable here. |
| taudit-crates-pumpkin-core-src-block-enti : 2 (`entities/sculk_catalyst.rs:46,68-77`) | FIXED (already at HEAD) | Cursors persist: `from_nbt` loads via `spreader.load` (:89-100), `write_nbt` saves via `spreader.save` (:118-124). Vanilla `SculkCatalystBlockEntity.java:52-62`. |
| taudit-crates-pumpkin-core-src-block-enti : 3 (`entities/sculk_catalyst.rs:158-160`) | VERIFIED | The death path is now game-event dispatch: `on_game_event` (:56-86) uses inclusive `within_hearing_radius` (:202-206) on floored offsets. Vanilla `EuclideanGameEventListenerRegistry.java:115-124` rejects only when `distSqr > radiusSqr`. Test `catalyst_hearing_includes_boundary`. The old `on_entity_death` no longer exists. |
| taudit-crates-pumpkin-core-src-block-enti : 5 (`entities/sculk_catalyst.rs:62`) | VERIFIED | `tick` (:102-115) seeds the spreader from `world.next_loot_seed()`, the world-seeded source. Vanilla `SculkCatalystBlockEntity.java:47` uses `level.getRandom()`. |

## Result
All four rows closed: 1, 2, 3 and 5 VERIFIED or FIXED at HEAD. No row remains open. No `.rs` file was edited this pass.
