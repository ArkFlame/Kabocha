# tmedium batch 1

Scope: first 8 rows tagged `M, STILL OPEN` in `docs/tracker/*.md` by file name order, excluding `tverify-mobs*`, `tsmall-batch*`, `tdeferred*`. Checks were source reads against vanilla (`.vendor/vanilla-decompiled`). No cargo run.

| # | file:line | status | reason |
|---|---|---|---|
| 1 | taudit-crates-pumpkin-core-src-bl-058f4c.md:41 | BLOCKED | Shape-driven light occlusion needs `useShapeForLightOcclusion` in the engine, which does not exist. |
| 2 | taudit-crates-pumpkin-core-src-bl-29396e.md:48 | BLOCKED | Source water to LiquidBlockContainer conversion is in fluid spreading; no pure test seam, needs a world-tick gametest. |
| 3 | taudit-crates-pumpkin-core-src-bl-4bd10c.md:62 | BLOCKED | `BlockFadeEvent` new-state field is an additive plugin API change, needs WIT/host/SDK update. |
| 4 | taudit-crates-pumpkin-core-src-bl-4d09af.md:79 | BLOCKED | `BlockIsReplacing::water_source` fix touches 46 call sites, large refactor. |
| 5 | taudit-crates-pumpkin-core-src-bl-4f710e.md:78 | VERIFIED-EQUIVALENT | Potted eyeblossom random tick in `crates/pumpkin-core/src/block/blocks/flower_pots.rs` matches vanilla `FlowerPotBlock.randomTick` (`FlowerPotBlock.java:157-169`): same open/close comparison and sound choice. No change. |
| 6 | taudit-crates-pumpkin-core-src-bl-6f9a28.md:106 | BLOCKED | Entity capture on save needs entity NBT capture in the structure template path; not a single slice. |
| 7 | taudit-crates-pumpkin-core-src-bl-6f9a28.md:113 | VERIFIED-EQUIVALENT | Tracker claims redstone SAVE does not write disk. Vanilla `StructureBlock.java:81` calls `saveStructure(false)` for redstone too, and Pumpkin `block/entities/structure_block.rs:1071` uses the same `save_to_disk` flag. Not a gap. |
| 8 | taudit-crates-pumpkin-core-src-bl-f6470f.md:50 | BLOCKED | `onExplosionHit` ordering in `world/explosion.rs:805-837` needs an explosion world test; no pure seam. |

Counts: 8 rows, 4 BLOCKED, 2 VERIFIED-EQUIVALENT, 0 fixed.
