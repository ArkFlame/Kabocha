# blocks / Shelf — audit

Target: MC 26.3 (`CURRENT_MC_VERSION`). Vanilla reference classes:
`net.minecraft.world.level.block.ShelfBlock`, `SideChainPartBlock`,
`state/properties/SideChainPart`, `SelectableSlotContainer`
(decompiled tree at `/run/media/linsaftw/SDD/tmp/mc-proof/ref`, `.vendor/` absent).

Reviewed source: `crates/pumpkin-core/src/block/blocks/shelf.rs` (602 lines,
no TODO/FIXME markers; the two "vanilla" hits are test names at :567 and :581).

## Verified matching vanilla (no change)

- `get_all_blocks_connected_to` (:205-241) == `SideChainPartBlock.getAllBlocksConnectedTo`:
  left loop then right loop, `MAX_CHAIN_LENGTH = 3` steps `< 3`, add when
  `isConnectionTowards(end)`, break on `isChainEnd`. Geometry matches too:
  `step_left = facing.rotate_clockwise()` == vanilla `facing.getClockWise()`,
  `step_right` == `getCounterClockWise()`.
- Helper tables (:476-519) are exact copies of `SideChainPart`: `isConnectionTowards`,
  `isChainEnd`, `whenConnectedToTheLeft/Right`, `whenDisconnectedFromTheLeft/Right`.
- `update_self_and_neighbors_on_powering_up` / `update_neighbors_after_powering_down`
  (:244-316) == interface defaults, including the `isBeingUpdatedByNeighbor` guard and
  "left first" priority.
- `on_place` (:320-328) == `getStateForPlacement`: facing = opposite of player horizontal
  facing, `POWERED` from neighbour signal, waterlogged from the replaced fluid.
- `on_state_replaced` (:363-368) == `affectNeighborsAfterRemoval`
  (`Containers.updateNeighboursAfterDestroy` + disconnect neighbours).
- `on_neighbor_update` (:370-407) == `neighborChanged`: early return when power is
  unchanged, chain cleared before the state write, sound + `BLOCK_ACTIVATE`/`BLOCK_DEACTIVATE`.
- `use_with_item` (:409-434) == `useItemOn`: off-hand PASS, block-entity presence check,
  `get_hit_slot` with `rows = 1`, `columns = 3` (vanilla `getRows`/`getColumns`).
- `swap_single_item` (:39-91) == `ShelfBlock.swapSingleItem`: whole-stack swap,
  creative/infinite-materials keeps the placed stack when the slot was empty,
  `ITEM_INTERACT_FINISH` gated on `UseEffects.interactVibrations`, sound order
  (`TAKE_ITEM` / `SINGLE_SWAP` / `PLACE_ITEM`) and `PASS` on empty hand + empty slot.
- `swap_hotbar` (:93-151) == `ShelfBlock.swapHotbar`: slot `9 - (len - index) * 3 + slot`,
  skip empty swaps when counting, `SUCCESS` on none, `MULTI_SWAP` sound,
  `setChanged(ENTITY_INTERACT)` per shelf part (test at :567 covers the mapping).
- `get_comparator_output` (:436-453) == `getAnalogOutputSignal`: back-face only
  (`direction != facing.getOpposite()` -> 0) and bitmask `bit0|bit1<<1|bit2<<2`.
- `is_pathfindable` (:455-457) == `type == WATER && fluid is water`.
- `get_state_for_neighbor_update` (:330-344) == `updateShape` water tick schedule.

## Fixed in this pass

| Gap | Rust | Vanilla | Fix | Size |
|---|---|---|---|---|
| Shelves placed in **flowing** water were not waterlogged | `shelf.rs:325` | `ShelfBlock.getStateForPlacement`: `getFluidState(...).is(Fluids.WATER)` is also true for flowing water (`FlowingFluid.isSame` covers still/flowing) | waterlogged now `matches!(replacing, BlockIsReplacing::Water(_))` instead of the source-only `water_source()` | S |

## Remaining gaps

| Location | Vanilla | Gap | Fix | Size |
|---|---|---|---|---|
| `block/mod.rs:667-673` `BlockIsReplacing::water_source` (46 call sites) | every `getStateForPlacement` that uses `FluidState#is(Fluids.WATER)` | only accepts level-0 sources, so all other waterloggable blocks still ignore flowing water | add `BlockIsReplacing::is_water()` and migrate the call sites; shelf already fixed locally | S |
| `shelf.rs:171-181` `set_part` | `SideChainPartBlock.setPart` -> `level.setBlockAndUpdate` -> `ShelfBlock.onPlace`, which re-runs `updateSelfAndNeighborsOnPoweringUp` when the written part leaves a powered shelf fully `UNCONNECTED` | `set_block_state` runs no `on_place` hook, so that recompute never happens after a neighbour disconnect; a powered shelf that only became unconnected is not re-evaluated against its other side | call `update_self_and_neighbors_on_powering_up` for the written position when the new part is `Unconnected` and the shelf is still powered | S |
| `shelf.rs` (absent) | `ShelfBlock.SHAPES` + `useShapeForLightOcclusion() = true` | shelf collision/occlusion shape comes from generated state data; not modelled in this file | verify the generated shape for `#minecraft:wooden_shelves`; engine/codegen work | L |
| `shelf.rs` (absent) | `BaseEntityBlock` + block entity | wiki: shelves cannot be pushed by pistons; Pumpkin has no block push-reaction/piston moveability concept | needs an engine-level piston push reaction, not a shelf-local fix | M |

Client-only / not required: vanilla's early client-side `CONSUME` in `useItemOn`
(:170-172) — Pumpkin interacts server-side only and the client predicts the swap.
