# Audit: crates/pumpkin-core/src/block/fluid/physics.rs

Scope: the two vanilla-reference doc comments (no TODO/FIXME markers) in a 22 line helper.
Fixed in this audit: lily pad was missing from the `LiquidBlockContainer` set, so flowing
fluid washed it away and a water bucket placed water next to it instead of targeting it like
vanilla `WaterlilyBlock`.

## Remaining gaps

### 1. Source water never waterlogs a `LiquidBlockContainer`
- file:line `crates/pumpkin-core/src/block/fluid/physics.rs:4-8`, caller
  `crates/pumpkin-core/src/block/fluid/flowing_trait.rs:126,155-161`
- vanilla: `FlowingFluid.spreadTo` + `SimpleWaterloggedBlock.placeLiquid` (a water *source*
  sets `waterlogged=true` and schedules a fluid tick)
- gap: `can_hold_fluid` documents that the container path is unreachable because spread always
  places flowing fluid. Vanilla reaches it whenever the spreading fluid is a source (level 0,
  not falling), so dry slabs/stairs/steps in water currently stay dry.
- suggested fix: in `spread_to`, when `props.level == Level::L0 && props.falling == Falling::False`
  and the target block `is_waterloggable()`, set the waterlogged property and schedule the fluid
  tick instead of replacing the block; keep the flowing-fluid rejection.
- size: M (needs a world/tick test, touches flowing_trait + block state writes)

### 2. `bubble_column` is not in `#minecraft:washed_away_by_fluids`
- file:line `crates/pumpkin-core/src/block/fluid/physics.rs:10-12`
- vanilla: `FlowingFluid.canHoldFluid` / `canHoldAnyFluid` treat a liquid-like block as a
  holder; `BubbleColumnBlock` sits in water and is not washed away.
- gap: the tag drives the predicate, so `can_hold_fluid(BUBBLE_COLUMN) == false` and fluid never
  flows into an existing bubble column (side and downward spread both skip it).
- suggested fix: confirm against the vanilla datapack (`assets/` ships the tag but not this
  entry); if vanilla allows it, special-case `Block::BUBBLE_COLUMN` next to the container check
  or extend the generated tag list.
- size: S

### 3. Dead lava/water clause in `try_flow`
- file:line `crates/pumpkin-core/src/block/fluid/flowing_trait.rs:148-150`
- observed: `below_block == &Block::WATER && fluid is lava` is redundant, because `water` is in
  `#minecraft:washed_away_by_fluids`, so `can_hold_fluid(WATER)` is already true.
- suggested fix: drop the clause (or keep it with a comment) once `can_hold_fluid(WATER)` is
  confirmed to match vanilla `canHoldFluid` for a liquid target state.
- size: S

## Notes
- `is_liquid_block_container` is also used by `crates/pumpkin-core/src/item/items/bucket.rs:413`;
  the lily pad fix makes that path match vanilla `BucketItem.use` (bucket emptied, no block placed).
- No test infrastructure covers fluid spreading, so only the helper contract is unit tested.

## Gap status
- 1. Source water never waterlogs a LiquidBlockContainer (`block/fluid/physics.rs:4-8`): STILL OPEN (M, needs world tick test).
- 2. `bubble_column` missing from `washed_away_by_fluids` (`physics.rs:10-12`): BLOCKED-DATA (note says confirm against vanilla tag data).
- 3. Dead lava/water clause in `try_flow`: STILL OPEN (not re-read this pass).


## Result

- 1. Source water never waterlogs a `LiquidBlockContainer`: STILL OPEN (out of unit). Verified against vanilla `FlowingFluid.spreadTo` and `SimpleWaterloggedBlock.placeLiquid`: a source water state does waterlog a dry waterloggable block. In Pumpkin the waterloggable target is filtered before `spread_to` by `pathfinder.rs:54` and by `try_flow`'s `can_hold_fluid` hole check; widening `can_hold_fluid` would also let lava into those blocks. A correct fix needs `pathfinder.rs` (side targets) and a lava guard in `lava.rs`, both outside this unit. Note the doc's `Level::L0` is `Level::L8` in Pumpkin.
- 2. `bubble_column` missing from `washed_away_by_fluids`: STILL OPEN (out of unit). Confirmed against vanilla datapack `assets/datapack/data/minecraft/tags/block/washed_away_by_fluids.json`: vanilla also omits `bubble_column`, so the generated tag is correct and must not be hand-edited. Vanilla treats it as a water hole via `bottomState.getFluidState().isSame(water)`, which Pumpkin's tag-only `can_hold_fluid` does not model; fixing that needs fluid-state-aware hole detection in `pathfinder.rs`, outside this unit.
- 3. Dead lava/water clause in `try_flow`: FIXED. Confirmed `water` is in the vanilla `#minecraft:washed_away_by_fluids` tag and is not a liquid container, so `can_hold_fluid(WATER)` is already true for lava. Removed the redundant `below_block == WATER && fluid is lava` clause in `flowing_trait.rs` and added a pure unit test asserting `can_hold_fluid(&Block::WATER)`.
- Files changed: `crates/pumpkin-core/src/block/fluid/flowing_trait.rs`, `crates/pumpkin-core/src/block/fluid/physics.rs`. Worker did not run cargo or tests (per task); `rustfmt --edition 2024` succeeded on both files; orchestrator to compile and test.
