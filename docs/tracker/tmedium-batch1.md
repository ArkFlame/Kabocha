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
| 7 | taudit-crates-pumpkin-core-src-bl-6f9a28.md:113 | FIXED | Vanilla redstone SAVE calls `saveStructure(false)` (in-memory only). Pumpkin `blocks/structure_block.rs:82` passed `true` (disk write); changed to `false`. GUI save still writes disk via `save_structure_area`. |
| 8 | taudit-crates-pumpkin-core-src-bl-f6470f.md:50 | BLOCKED | `onExplosionHit` ordering in `world/explosion.rs:805-837` needs an explosion world test; no pure seam. |

Counts: 8 rows, 6 BLOCKED, 1 VERIFIED-EQUIVALENT, 1 fixed.

## Result

Verified against decompiled vanilla at `.vendor/vanilla-decompiled/net/minecraft/` on
2026-11-18. rustfmt run on the one touched .rs file. No cargo/git run (per brief).

- 1 (058f4c:41) BLOCKED: shape-driven light occlusion needs `useShapeForLightOcclusion` in the
  engine, which does not exist; outside these files.
- 2 (29396e:48) BLOCKED: source-water to LiquidBlockContainer conversion is in fluid spreading,
  no pure test seam (needs a world-tick gametest); outside these files.
- 3 (4bd10c:62) BLOCKED: `BlockFadeEvent` new-state field is an additive plugin API change,
  needs WIT/host/SDK update; outside these files.
- 4 (4d09af:79) BLOCKED: `BlockIsReplacing::water_source` fix touches ~46 call sites,
  cross-crate refactor.
- 5 (4f710e:78) VERIFIED-EQUIVALENT: `flower_pots.rs:85-133` potted eyeblossom random tick
  matches vanilla `FlowerPotBlock.randomTick` (157-169): same open/close comparison, same
  `unwrap_or(is_open)` fallback, transform particle colored for the new type, and same
  long-switch sound. Particle helper is already covered by tests in `plant/eyeblossom.rs`.
  No change.
- 6 (6f9a28:106) BLOCKED: entity capture on save needs entity-NBT capture in the template path,
  not a single slice; cross-crate.
- 7 (6f9a28:113) FIXED: vanilla `StructureBlock.trigger` calls `saveStructure(false)` for redstone
  (fills the in-memory template, does not persist the `.nbt`); only the GUI path uses the no-arg
  `saveStructure()` -> `saveStructure(true)`. Pumpkin `blocks/structure_block.rs:82` passed `true`
  (disk write) from an earlier misreading; changed to `false` with a comment on the vanilla quirk.
  GUI save still writes disk through `save_structure_area` -> `save_structure(world, true)`.
- 8 (f6470f:50) BLOCKED: `onExplosionHit` ordering needs an explosion world test, no pure seam;
  outside these files.

Files touched: `crates/pumpkin-core/src/block/blocks/structure_block.rs`.
No pure unit test seam for a redstone-tick flag (needs a live world / filesystem).
