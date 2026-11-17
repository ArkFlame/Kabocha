# Tracker: blocks / Pistons (RE-REVIEW)

Re-reviewed after the source changed (`fix(piston): drop base item when extended head is broken`,
60bdcacf1). No decompiled vanilla tree is present in this worktree, so comparisons below are
against the vanilla class semantics for `PistonBaseBlock` / `PistonHeadBlock` /
`MovingPistonBlock` / `PistonBlockEntity` / `PistonHandler` and `docs/VANILLA_REF.md:22`.

Files audited:

- `crates/pumpkin-core/src/block/blocks/piston/piston.rs`
- `crates/pumpkin-core/src/block/blocks/piston/mod.rs` (`PistonHandler`)
- `crates/pumpkin-core/src/block/blocks/piston/piston_head.rs`
- `crates/pumpkin-core/src/block/blocks/piston/piston_extension.rs`
- `crates/pumpkin-core/src/block/entities/piston.rs` (`PistonBlockEntity`)

Overall: the port is close. Event flow (`handle_synced_block_event` vs `onSyncedBlockEvent`
extend/contract/drop ordering), `PistonHandler::calculate_push` / `try_move` (12-block limit,
sticky chains, `set_moved_blocks`, destroy-vs-block reactions) and the retract fast-retract
`TRIGGER_DROP` gate (`last_progress < 0.5`, same world age, handling tick) all match vanilla
branch for branch. The recent #3144 fix (`piston_head.rs:55-75`, drops the base once from
`on_state_replaced` and forwards `SKIP_DROPS`/`MOVED`) is correct and no longer a gap.

## Fixed in this pass

| # | Where | Gap | Vanilla class | Fix | Size |
|---|-------|-----|---------------|-----|------|
| 1 | `piston.rs:54-66` | `is_movable` never consulted the world border, so pistons could push/retract blocks outside it | `PistonBlock.isMovable` (`!world.getWorldBorder().contains(pos)`) | Added the `contains_block` early return after the height-limit check | S |

## Remaining gaps

| # | file:line | Gap | Vanilla class / behaviour | Suggested fix | Size |
|---|-----------|-----|---------------------------|---------------|------|
| 1 | `piston.rs:411-430` | Destroyed blocks are removed through `World::break_block`, which fires a cancellable `BlockBreakEvent`; a cancelled break leaves the move half-applied | `PistonBaseBlock.move` calls `world.breakBlock(pos, true)` unconditionally (that break is not cancellable) | Use a non-cancellable destroy path (or skip the event) for blocks with `PistonBehavior::Destroy`, keep firing `GameEvent::BlockDestroy` | M |
| 2 | `entities/piston.rs:57-112` | Entity sweep uses the full block AABB for every pushed block; vanilla uses the moved block's `VoxelShape`, so stairs, slabs and fences push less (or not at all) | `PistonBlockEntity.pushEntities` + `getCollisionShape` | Compute the collision shape of `pushed_block_state` and use it for the swept volume instead of `BoundingBox::from_block` | M |
| 3 | `entities/piston.rs:165-178` | Approximated piston-head slab sits on the far face of the head block; `PistonHeadBlock`'s shape is on the base side (east head: `x 0.0-0.25`, not `0.75-1.0`) | `PistonHeadBlock` `EAST_AABB` / `SHORT_EAST_AABB` | Anchor the slab on the base-side face and re-check end-crystal push behaviour | S |
| 4 | `entities/piston.rs:89-93` | Players are excluded from piston pushes | `PistonBlockEntity.pushEntities` moves players via `Entity.move(PISTON)` | Intentional divergence (client-authoritative movement, `docs/LEDGER.tsv` #1679); revisit only with movement reconciliation | M |
| 5 | `entities/piston.rs:229-266` | Tick finalisation ignores the `pushed_block_state.is_air()` case and instead places-then-breaks the pushed state when the shape no longer fits; vanilla's air branch (`setBlockState(AIR)` + `dropStacks`) is not modelled | `PistonBlockEntity.tick` | Add the air branch before `update_from_neighbor_shapes` and compare drop stacks for piston-destroyed blocks | S |
| 6 | `mod.rs:156-161` | Sticky chain positions are appended in reverse (`(0..i).rev()`); vanilla appends the chain ascending, which is the `movedBlocks` order used later for neighbour updates | `PistonHandler.tryMove` | Append ascending and confirm `set_moved_blocks(from, to)` slicing still matches | S |
| 7 | `piston_head.rs:39-53` | Creative head break removes the base via `break_block(Some(player))`, attributing the base break to the player (stats/criteria/events) where vanilla uses a plain `setBlockState(AIR, SKIP_DROPS)` | `PistonHeadBlock.onBreak` | Use `set_block_state` with `SKIP_DROPS` for the creative path, keep `break_block` for survival | S |

## Checked and matching (no action)

- `should_extend` (`piston.rs:313-335`): power from the facing side is ignored, power from above and
  its non-down neighbours is honoured - the #3833 "powered from the head" gate.
- `try_move` / `calculate_push` early outs (`mod.rs:50-80`): air, piston base/head, duplicate
  position, 12-block limit, `Destroy` when retracted, `PushOnly` only along the piston axis.
- Retract ordering (`piston.rs:204-298`): `finish()` of a still-animating head, moving-piston base
  swap, sticky re-pull gate, `TRIGGER_DROP` fast-retract detection.
- `piston_head.rs:77-99` (`get_state_for_neighbor_update` -> air when the base stops fitting)
  covers head removal after the base is destroyed or replaced.
