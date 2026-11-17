# Audit: crates/pumpkin-core/src/block/fluid/physics.rs

Tracker: category=audit, coverage=AUDIT. Source is 22 lines (2 vanilla-reference comments,
no TODO/FIXME). Deep audit against vanilla `FlowingFluid` / `LiquidBlockContainer`
(1.21.x, `BlockTags.WASHED_AWAY_BY_FLUIDS` era) plus Paper/Spigot behaviour.

Result: no small certain in-file fix. One real behaviour gap (medium), listed below.

## Verified correct (no change needed)

- `can_hold_fluid` (physics.rs:10-12) matches vanilla `FlowingFluid#canHoldFluid`'s
  tag branch: `state.is(BlockTags.WASHED_AWAY_BY_FLUIDS)`. The generated tag
  (crates/pumpkin-data/src/generated/tag.rs:18834) contains air/cave_air/void_air, fire,
  saplings, flowers, corals, torches, rails, vines, snow, powder_snow, seagrass and
  tall_seagrass, i.e. exactly the blocks water destroys on contact.
- `is_liquid_block_container` (physics.rs:16-22) covers every vanilla
  `LiquidBlockContainer` implementor: `SimpleWaterloggedBlock` via the waterlogged
  property (`Block::is_waterloggable`, crates/pumpkin-data/src/blocks.rs:192), plus
  `KelpBlock`, `KelpPlantBlock`, `SeagrassBlock`, `TallSeagrassBlock`.
- Keeping kelp/seagrass in the container list is required even though their
  `canPlaceLiquid`/placeLiquid return false: vanilla `BucketItem#use` still targets the
  clicked `LiquidBlockContainer` and empties the bucket without placing. bucket.rs:413-433
  depends on that and documents it.
- Excluding containers from the tag branch is required because seagrass/tall_seagrass are
  themselves in `washed_away_by_fluids`; without the exclusion flowing water would replace
  them.

## Gap 1 (medium): flowing water never waterlogs a LiquidBlockContainer

physics.rs:10-12 (with flowing_trait.rs:145-165, pathfinder.rs:22/54/169/185)

Vanilla:

- `FlowingFluid#canHoldFluid` asks the block first: a `LiquidBlockContainer` is holdable
  when `container.canPlaceLiquid(level, pos, state, fluidType)`; only otherwise does it
  fall through to the replaceable-by-fluid check.
- `SimpleWaterloggedBlock#canPlaceLiquid` is true for water on a state whose
  `waterlogged` is false, so flowing water is allowed to spread into a dry slab, stairs,
  fence, coral block, etc.
- `FlowingFluid#spreadTo` then calls `LiquidBlockContainer#placeLiquid` (sets
  `waterlogged=true`) instead of `beforeDestroyingBlock` + `setBlock(fluid)`.

Pumpkin returns `false` for every container regardless of state, so flowing water never
enters or waterlogs them (and `pathfinder::is_hole` also treats a dry waterloggable block
below as solid). The physics.rs:6-8 comment claims vanilla only waterlogs through source
conversion; that is inaccurate for `SimpleWaterloggedBlock`, so the comment hides the gap.

Suggested fix:

1. Give `can_hold_fluid` the target `BlockStateId` and fluid type:
   containers -> `fluid is water && block.is_waterloggable() && !block.is_waterlogged(state_id)`
   (kelp/seagrass stay false, matching their `canPlaceLiquid`); else the tag branch.
2. In `flowing_trait::spread_to`, when the target is a liquid container that can hold the
   fluid, set `waterlogged=true` (extract the waterlog state code already in
   bucket.rs:470-520 into a helper) instead of `break_block` + `set_block_state`; keep the
   `BlockFromToEvent` fire point before the change.
3. Update the 6 call sites (pathfinder.rs:22,54,169,185, flowing_trait.rs:148) to pass
   state/fluid; fix the physics.rs:4-8 comment at the same time.

Size: M (~80-120 LOC across 3 files). Test: pumpkin-gametest, water source beside a dry
`oak_slab` / `oak_fence` becomes waterlogged; dry slab below flowing water becomes
waterlogged and is not removed.
