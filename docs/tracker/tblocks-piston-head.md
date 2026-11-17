# blocks / Piston Head (RE-REVIEW)

Target: JavaMinecraftVersion::V_26_3. Reference: `.vendor` absent, compared against the
decompiled Paper/Canvas tree in `/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft`
(`world/level/block/piston/PistonHeadBlock.java`, `PistonBaseBlock.java`,
`PistonMovingBlockEntity.java`, `MovingPistonBlock.java`).

Reviewed source change: `945f7538e` (pass the breaking player when removing the piston base).
Re-review confirms that change: `broken` now mirrors `PistonHeadBlock#playerWillDestroy`
(`player.preventsBlockDrops()`, i.e. creative) plus `#affectNeighborsAfterRemoval`
(`destroyBlock(base, true)`), so creative head breaks are dropless and survival breaks drop
the piston.

Verified matches: `is_fitting_base` = `PistonHeadBlock#isFittingBase` (type/EXTENDED/facing),
`can_place_at` = `#canSurvive`, `get_state_for_neighbor_update` = `#updateShape`,
`on_neighbor_update` = `#neighborChanged` (forwards to the base), `is_pathfindable` =
`#isPathfindable`, and `piston.rs` head placement/targeting.

Fixed in this pass:

- `piston.rs:291-305` - a non-sticky retract removed whatever block sat at `armPos`. Paper
  `PistonBaseBlock.java:228-232` only removes it when it is `PISTON_HEAD` with the same
  facing, otherwise it just resends the block to the client (`allowPermanentBlockBreakExploits`
  false). The removal is now guarded the same way, so a stale `extended` base cannot delete an
  unrelated block.

Remaining gaps:

| Location | Vanilla | Gap | Fix | Size |
|---|---|---|---|---|
| `piston.rs:282-289` | `PistonBaseBlock.java:231` `level.removeBlock(pos.relative(direction), false)` for the sticky no-pull branch is unguarded | A headless sticky piston retracting with nothing to pull still deletes the block in front | Apply the same head+facing guard as the non-sticky branch, or mirror Paper exactly | S |
| `piston_head.rs:55-58` | `PistonHeadBlock#affectNeighborsAfterRemoval` ignores `movedByPiston` and always `destroyBlock(base, true)` | `on_state_replaced` returns early when `moved` and reuses the removal `SKIP_DROPS` flag | Drop the `moved` early return; keep the creative dropless path in `broken` | S |
| `piston_head.rs:38-72` | Removal is `#playerWillDestroy` (pre-removal) then `#affectNeighborsAfterRemoval` | Both Rust hooks run after removal and `broken` duplicates `on_state_replaced`, so the base is removed once through whichever hook fires first | Consolidate into a pre-removal hook when `BlockBehaviour` gains one | S |
| `piston_head.rs:134-136` | `PistonHeadBlock#getCloneItemStack` returns the piston/sticky piston item | No clone-item hook in `BlockBehaviour`; middle-click pick-block has no server-provided item | Add `get_clone_item_stack` to the trait and dispatch it | S |
| `piston_head.rs` (no light hook) | `PistonHeadBlock#useShapeForLightOcclusion` returns true | Head occludes light as a full cube; vanilla uses the 4px plate/arm shape | Model shape-based occlusion in the light engine | M |
| `block/entities/piston.rs:72-78,165-178` | `PistonMovingBlockEntity#moveCollidedEntities` uses `getCollisionRelatedBlockState()` (moved block's shape, or the head with `SHORT` = plate + 4x4 arm out to 1.25) | Rust sweeps a full cube for moved blocks and a 4px front slab for the head | Use the moved-state collision shape and vanilla head shape/progress for the sweep | M |
| `block/entities/piston.rs:57-112` | `PistonMovingBlockEntity#moveCollidedEntities` slime `causeBounce` velocity and `moveStuckEntities` honey drag | No slime bounce or honey stuck-entity handling | Port the slime bounce and honey drag paths | M |
| `piston.rs:383-389` | `PistonBaseBlock#moveBlocks` clears the arm with `UPDATE_NONE \| UPDATE_KNOWN_SHAPE` | Rust adds `SKIP_BLOCK_ENTITY_REPLACED_CALLBACK` on top (no observable effect) | Keep; noted for completeness | S |

Note: the head block's other vanilla methods (`getShape`/SHORT, `#rotate`, `#mirror`) are
data-driven in `pumpkin-data` and match the generated `piston_head` states.
