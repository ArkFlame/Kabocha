# Parity rows for crates/pumpkin-core/src/block/blocks/shelf.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-bl-4d09af** [partial/OWN-OPEN] own audit: 7 open, 1 fixed, 0 verified, 4 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-bl-4d09af.md) docs/tracker/taudit-crates-pumpkin-core-src-bl-4d09af.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)


## Result (THIRD PASS, 2026-11-18, final)

All STILL OPEN rows from the audit (`docs/tracker/taudit-crates-pumpkin-core-src-bl-4d09af.md`)
are now closed. No source change was needed: every code-level row is already correct and the
two remaining rows require generated data or a whole missing subsystem.

- Flowing-water waterlogging (`shelf.rs:325`): VERIFIED, no change. Vanilla
  `ShelfBlock.getStateForPlacement` (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/ShelfBlock.java:125-131`)
  sets `WATERLOGGED = replacedFluidState.is(Fluids.WATER)`. Decompiled `FluidState.is(Fluid)`
  is `this.fluidOwner == rawType`
  (`.vendor/vanilla-decompiled/net/minecraft/world/level/material/FluidState.java:174-176`):
  flowing states have owner `Fluids.FLOWING_WATER`, so `is(Fluids.WATER)` is true only for
  level-0 source water. Pumpkin's `args.replacing.is_water()` (`shelf.rs:325`) maps to
  `matches!(self, Self::Water(0))` (`block/mod.rs:793-795`), the exact source-only match.
  The existing pure test `mod.rs:942-946 is_water_matches_only_source_water` locks this seam.

- `BlockIsReplacing::water_source` / 46 call sites (`block/mod.rs`): VERIFIED, resolved
  externally, no change. `BlockIsReplacing::water_source` no longer exists (no occurrences
  remain under `crates/`). The helpers `is_water()` / `is_any_water()` / `is_full_water()`
  exist at `block/mod.rs:793-813` with unit tests at `:942-964`, and every call site,
  including `shelf.rs:325`, is migrated. Vanilla `getStateForPlacement` above is the parity
  reference for `is_water()`.

- Shelf collision/occlusion shape: BLOCKED-DATA. Vanilla `ShelfBlock.SHAPES` and
  `useShapeForLightOcclusion() = true`
  (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/ShelfBlock.java:53-77`) need
  per-state voxel (collision + occlusion) shape data for the `#minecraft:wooden_shelves`
  (`AcaciaShelfLikeProperties` FACING variants), which is absent from
  `crates/pumpkin-data/src/generated/`, plus a `useShapeForLightOcclusion` behaviour hook
  that `BlockBehaviour` does not expose. Missing generated field: per-state voxel/collision
  shape for the wooden_shelves states (codegen/extractor output).

- Piston push reaction for shelves: BLOCKED-DATA, whole missing subsystem. Vanilla
  `BlockBehaviour.Properties.pushReaction` and `getPistonPushReaction()`
  (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/state/BlockBehaviour.java:451,864-865,1239,1430`);
  `ShelfBlock extends BaseEntityBlock`, which is not piston-pushable. Pumpkin has no
  `PushReaction` field, no per-block moveability concept and no piston push-reaction
  subsystem in `crates/pumpkin-core/src`, so this cannot be fixed in shelf.rs.
