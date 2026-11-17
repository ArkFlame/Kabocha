# Tracker: blocks / Piston Head (RE-REVIEW-UNTRACKED)
Re-audit after `945f7538e` (breaking player passed to base removal), `7a9b21932`
(headless-piston retract guard) and `5eba75adf` (dropped the duplicate `broken()` head
handler, leaving `on_state_replaced` as the only base-removal path).

Source: `crates/pumpkin-core/src/block/blocks/piston/piston_head.rs` plus its call chain
(`entity/player.rs::break_block`, `world/mod.rs::set_block_state`/`break_block`,
`block/entities/piston.rs`).
Vanilla reference used this pass: decompiled tree at
`/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft/world/level/block/piston/`
(`PistonHeadBlock.java`, `PistonBaseBlock.java`), not memory.

## Verified matching vanilla (no change)

| Rust | Vanilla | Note |
|---|---|---|
| `piston_head.rs:19-34` `is_fitting_base` | `PistonHeadBlock.java:57-60` `isFittingBase` | Exact: base must be the piston matching the head `TYPE`, `EXTENDED`, same `FACING`. The older note that this was "stricter than vanilla" was wrong; the TYPE->base mapping is vanilla. |
| `piston_head.rs:38-56` `on_state_replaced` | `playerWillDestroy` (`:63-71`) + `affectNeighborsAfterRemoval` (`:75-79`) | Removes the base only when it fits; `SKIP_DROPS` forwarded gives the creative dropless path, survival drops the base once. Gated on the block actually changing (`world/mod.rs`). |
| `piston_head.rs:58-80` `get_state_for_neighbor_update` | `updateShape` (`:83-96`) | Only the neighbour opposite `FACING` can invalidate the head, via `can_place_at`. Vanilla has no `SHORT` reset branch in this version, so none is needed. |
| `piston_head.rs:82-91` `can_place_at` | `canSurvive` (`:99-102`) | Fitting base, or `MOVING_PISTON` with the same `FACING`. |
| `piston_head.rs:93-116` `on_neighbor_update` | `neighborChanged` (`:105-115`) | Survives, then forwards the update to the base position. |
| `piston_head.rs:118-120` | `isPathfindable` | `false`. |
| `piston.rs:112-118`, `:492-504` | `PistonBaseBlock.java` extend | Head state = facing + type, `SHORT` false, pushed as the source moving-block entity. |
| `pumpkin-world/src/lighting/occlusion.rs:21,25-39` | `useShapeForLightOcclusion` (`:48-50`) | Handled: `PISTON_HEAD` is a shape-occlusion singleton, per-face masks from generated side-solidity. |

`SHORT` is never written by Pumpkin and vanilla only uses it in `getShape` (`:53-54`,
`piston_head_short` model); no server-side behaviour depends on it.

## Remaining gaps

| Location | Vanilla | Gap | Suggested fix | Size |
|---|---|---|---|---|
| `piston_head.rs:39-41` | `affectNeighborsAfterRemoval` (`:75-79`) ignores `movedByPiston` and always `destroyBlock(base, true)` | Rust returns early when `args.moved`, so a head removed as part of a piston move never removes its base | Drop the `moved` early return. Not done here: verify first that the retract path always replaces the base with `MOVING_PISTON` before clearing the head (`piston.rs:263-266` then `:307-311`), or every retract would drop a piston item | S |
| `piston_head.rs` (no shape hook) | `getShape` (`:39-40,53-54`): 4px platform + 4px stem protruding to `20/16` of the cell, `SHORT` variant without the stem | Pumpkin stores only side/center-solidity masks (`block_state.rs:169-193`), no per-state collision AABB, so the head has no plate/stem collision; entities cannot stand on an extended head and walk through the plate | Needs per-state collision AABB data from the assets extractor plus a `get_shape`-style hook; do not fake it with `head_shape_aabb` in the block entity (that only bounds the push) | M |
| `piston_head.rs` (no clone-item hook) | `getCloneItemStack` (`:118-120`) returns the piston/sticky-piston item | Pick-block (`ItemStackRequest`/middle-click) on a head yields nothing; `BlockBehaviour` has no clone-item hook | Add `get_clone_item_stack` to `BlockBehaviour`, implement `PISTON`/`STICKY_PISTON` from `TYPE`, dispatch from the pick-block handler | S |
| `piston_head.rs:50-55` | `destroyBlock` (`:67`, `:78`) has no block event and no player | Base removal goes through `World::break_block` with `None` player, so plugins get a `BlockBreakEvent` with no player and no cause | Use plain `set_block_state(AIR, NOTIFY_ALL)` + explicit `drop_resources` when drops are enabled; or pass the head breaker through `OnStateReplacedArgs` | S/M |
