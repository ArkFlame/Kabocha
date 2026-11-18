# Final rows from docs/tracker/tblocks-pistons-9d1580.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: piston.rs

- h6 row 3 (head slab anchored on far face): VERIFIED, already correct in the tree; no code change needed.
All "Remaining gaps" rows were re-checked against the decompiled vanilla: rows 1-3 are already closed in the tree, row 4 is BLOCKED on a whole missing subsystem.

## Result
Row 1 (non-cancellable destroy): FIXED and VERIFIED. Vanilla `PistonBaseBlock.move` destroys unconditionally via `dropResources` + `setBlock(AIR, UPDATE_CLIENTS|UPDATE_KNOWN_SHAPE)` + `gameEvent(BLOCK_DESTROY)` (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/piston/PistonBaseBlock.java:359-370`); `piston.rs:438` now uses `World::break_block_naturally` plus `GameEvent::BlockDestroy`, i.e. non-cancellable with drops and the game event. Matches.

Row 2 (collision-shape entity sweep): FIXED and VERIFIED. Vanilla `PistonMovingBlockEntity.moveCollidedEntities` sweeps `getCollisionRelatedBlockState().getCollisionShape(...)`; `entities/piston.rs:124-139` uses the 4-pixel head slab for the head BE and `pushed_block_state.get_block_collision_shapes_at(...)` for pushed blocks. Matches.

Row 3 (head slab anchored on far face): VERIFIED, no code change. Vanilla `PistonHeadBlock` (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/piston/PistonHeadBlock.java:37-40`) builds the collision shape from `SHAPE_PLATFORM = Block.boxZ(16.0, 0.0, 4.0)` (a 4-pixel plate on the NORTH face) and `Shapes.rotateAll` (`.vendor/vanilla-decompiled/net/minecraft/world/phys/shapes/Shapes.java:504-520`) puts that plate on the facing side for every direction. `PistonBlockEntity::head_shape_aabb` (`entities/piston.rs:251-264`) already produces exactly those six slabs, locked by the pure test `head_shape_slab_sits_on_facing_side` (`entities/piston.rs:452-481`). End-crystal push geometry is unchanged.

Row 4 (players excluded from piston pushes): BLOCKED. Vanilla `moveCollidedEntities` pushes every entity with `PushReaction != IGNORE_ENTITY` via `Entity.move(PISTON)`, players included (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/piston/PistonMovingBlockEntity.java`). Pumpkin is deliberately client-authoritative (`docs/LEDGER.tsv` #1679); a correct fix needs a whole player movement-reconciliation subsystem in the client-move validation path, outside this unit.

No source files modified (every fixable row is already closed); no new test seam exists. rustfmt not needed.
