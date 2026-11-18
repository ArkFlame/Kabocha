# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-bl-556f10.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: ladder.rs lanterns.rs

Second-pass audit of the STILL OPEN rows, scoped to ladder.rs, lanterns.rs,
Re-read all five in-scope files and re-checked each STILL OPEN row against
No STILL OPEN / OPEN rows remain. No source files changed this pass.
(FIXED / VERIFIED / BLOCKED-DATA); none remain STILL OPEN.

## Result

Final pass. Re-read both in-scope files (ladder.rs, lanterns.rs) and re-checked
every still-open parent-tracker row against
`.vendor/vanilla-decompiled/net/minecraft/`. Both in-scope files already match
vanilla; no source edit was needed and no pure-test seam exists (`on_place`
needs a live World/Player/BlockAccessor, and the related pure tests already
live in sculk_vein.rs/slabs.rs from earlier passes).

| Row | Final disposition | Vanilla evidence (.vendor path) |
|---|---|---|
| 28 can_update_at face convention | FIXED (parent; consumers VERIFIED here) | `BlockPlaceContext.getNearestLookingDirections` (`.vendor/.../world/item/context/BlockPlaceContext.java:71-89`) moves `clickedFace.getOpposite()` to the front only when `!replaceClicked`. registry.rs:779/822 passes `final_face` (= `clickedFace.opposite()`, the placement direction) as `OnPlaceArgs.direction`; ladder.rs:18-42 and lanterns.rs:26-34 reorder by `args.direction.to_facing()` under guard `*position != use_item_on.position` (`!replaceClicked`), with `copy_within`/`directions[0]` mirroring `System.arraycopy`. In-scope consumers correct; the can_update_at fix itself was registry.rs/slabs.rs/sculk_vein.rs (outside this unit), already FIXED. |
| 29 BlockIsReplacing Other/None / rotation | VERIFIED, NO CHANGE | Ladder rotation/dedup (ladder.rs:17-55) matches `LadderBlock.getStateForPlacement` (`.vendor/.../world/level/block/LadderBlock.java:73-96`): dedup on `behind == LADDER && facing == clickedFace`, FACING = dir.opposite(), WATERLOGGED from replaced fluid. Lantern (lanterns.rs:19-47) matches `LanternBlock.getStateForPlacement` (`.vendor/.../world/level/block/LanternBlock.java:34-48`): hanging = (dir == UP), WATERLOGGED from replaced fluid. Both key rotation on `None`/`Water` only via `args.direction`, consistent with BlockPlaceContext.java:71-89. |
| ladder survival / neighbour update | VERIFIED, NO CHANGE | `can_place_ladder_at` and `can_place_at`/`get_state_for_neighbor_update` (ladder.rs:59-109,123-131) match `LadderBlock.canAttachTo`/`canSurvive`/`updateShape` (`.vendor/.../world/level/block/LadderBlock.java:39-69`): solid side at `pos.relative(FACING.opposite())` facing FACING, and AIR when `directionToNeighbour.getOpposite() == FACING && !canSurvive`. |
| lantern survival / neighbour update / pathfindable | VERIFIED, NO CHANGE | `can_survive` and `get_state_for_neighbor_update` (lanterns.rs:49-89,92-103) match `LanternBlock.canSurvive`/`updateShape` (`.vendor/.../world/level/block/LanternBlock.java:61-88`): support at `pos.relative(getConnectedDirection(state).opposite())`; `is_pathfindable -> false` matches LanternBlock.java:95-98. |
| 31 inside-collision shape | VERIFIED, NO CHANGE (outside unit) | Parent tracker confirmed the premise was wrong: default `getEntityInsideCollisionShape` returns `Shapes.block()` (`.vendor/.../world/level/block/state/BlockBehaviour.java:336-338`), matching Pumpkin's full-block default in block/mod.rs. No union API change needed. |
| 32 isPathfindable Water default | VERIFIED, NO CHANGE (in scope) | lanterns.rs:87-89 returns false, matching `LanternBlock.isPathfindable` (`.vendor/.../world/level/block/LanternBlock.java:95-98`). registry.rs fallback is outside this unit and was already VERIFIED (`.vendor/.../BlockBehaviour.java:130-134`). |
| 35 player_placed direction | VERIFIED, NO CHANGE (outside unit) | Vanilla `Block.setPlacedBy` (`.vendor/.../world/level/block/Block.java:543`) has no face parameter; `PlayerPlacedArgs` carries no direction field. registry.rs outside this unit. |
| 75 BlockFertilizeEvent payload | VERIFIED, NO CHANGE (outside unit) | Already fixed via `World::capture_bonemeal_changes` -> `changed_blocks`; plugin-API files outside this unit. |
| 76 TestBlock registered twice | VERIFIED, NO CHANGE (outside unit) | Second `register(TestBlock)` already removed from registry.rs. |

No STILL OPEN / OPEN rows remain. No source files changed this pass; rustfmt not needed.
