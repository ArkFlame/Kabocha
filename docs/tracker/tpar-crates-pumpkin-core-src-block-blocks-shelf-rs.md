# Parity rows for crates/pumpkin-core/src/block/blocks/shelf.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-bl-4d09af** [partial/OWN-OPEN] own audit: 7 open, 1 fixed, 0 verified, 4 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-bl-4d09af.md) docs/tracker/taudit-crates-pumpkin-core-src-bl-4d09af.md

## Result
FINAL PASS per-row results (verified against `.vendor/vanilla-decompiled/net/minecraft/`; every row closed, no source change needed):

1. Flowing-water waterlogging (`shelf.rs:325`, audit "Fixed in this pass"): VERIFIED, no code change. Vanilla `ShelfBlock.getStateForPlacement` sets `WATERLOGGED = replacedFluidState.is(Fluids.WATER)` (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/ShelfBlock.java:125-130`). `FluidState.is(Fluid)` is `this.fluidOwner == rawType` (`.vendor/vanilla-decompiled/net/minecraft/world/level/material/FluidState.java:174-175`); flowing states are owned by `Fluids.FLOWING_WATER` (`.vendor/vanilla-decompiled/net/minecraft/world/level/material/FlowingFluid.java:72,306-315`; `.vendor/vanilla-decompiled/net/minecraft/world/level/material/WaterFluid.java:34-35`), so only level-0 source water matches. Pumpkin's `args.replacing.is_water()` equals `Water(0)` (`crates/pumpkin-core/src/block/mod.rs:793-795`), the exact source-only match, locked by `mod.rs:942-946 is_water_matches_only_source_water`. Vanilla `isPathfindable` uses `FluidTags.WATER` (any level, `ShelfBlock.java:79-82`), but a shelf's fluid state is water only when waterlogged, so `shelf.rs:455-457 state.is_waterlogged()` matches.

2. `BlockIsReplacing::water_source` / 46 call sites (`block/mod.rs`): VERIFIED, resolved externally, no code change. No `BlockIsReplacing::water_source` remains anywhere under `crates/`. The helpers `is_water()` / `is_any_water()` / `is_full_water()` exist at `crates/pumpkin-core/src/block/mod.rs:793-813` with pure tests at `:942-964`, and every call site, including `shelf.rs:325`, is migrated. Vanilla `getStateForPlacement` (`ShelfBlock.java:125-130`, source-only `is(Fluids.WATER)`) is the parity reference mapped to `is_water()`.

3. Shelf collision/occlusion shape: BLOCKED-DATA. Vanilla `ShelfBlock.SHAPES` (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/ShelfBlock.java:53-55`) and `useShapeForLightOcclusion() = true` (`:75-77`) need per-state voxel (collision + light-occlusion) shape data for the `#minecraft:wooden_shelves` FACING/part variants, which is absent from `crates/pumpkin-data/src/generated/` (no Java per-state voxel/collision shape table; only `bedrock_voxel_shape.rs` exists, and that is Bedrock-only). Missing generated field: per-state voxel/collision and light-occlusion shape for the `wooden_shelves` states (codegen/extractor output), plus a `useShapeForLightOcclusion` behaviour hook not exposed by `BlockBehaviour`.

4. Piston push reaction for shelves: BLOCKED-DATA (whole missing subsystem). Vanilla `BlockBehaviour.Properties.pushReaction` / `getPistonPushReaction()` (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/state/BlockBehaviour.java:451,864-865,1239,1430`); `ShelfBlock extends BaseEntityBlock`, which is immovable by pistons. Pumpkin has no `PushReaction` field, no per-block moveability concept, and no piston push-reaction subsystem in `crates/pumpkin-core/src` (confirmed by grep, only piston gametests exist), so this cannot be fixed in `shelf.rs`.


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
