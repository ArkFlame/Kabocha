# Parity rows for crates/pumpkin-core/src/block/blocks/piston/mod.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/tblocks-pistons-9d1580** [partial/OWN-OPEN] own audit: 5 open, 2 fixed, 1 verified, 0 blocked-data (docs/tracker/tblocks-pistons-9d1580.md) docs/tracker/tblocks-pistons-9d1580.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)

## Result (third pass, final)

Re-verified every remaining row of the piston tracker against the decompiled Paper/vanilla
source under .vendor/vanilla-decompiled/. Three rows are already closed in the current tree
and are VERIFIED with their vanilla citations; one row needs a whole missing subsystem and
is BLOCKED-DATA.

- Remaining gap 1 (non-cancellable destroy): VERIFIED. PistonBaseBlock.java:358-370 destroys
  each block unconditionally via dropResources + setBlock(AIR, UPDATE_CLIENTS|UPDATE_KNOWN_SHAPE)
  + gameEvent(BLOCK_DESTROY), with no cancellable BlockBreakEvent. Pumpkin piston.rs:422-439
  uses World::break_block_naturally + GameEvent::BlockDestroy, matching.
  (.vendor/vanilla-decompiled/net/minecraft/world/level/block/piston/PistonBaseBlock.java)
- Remaining gap 2 (collision-shape entity sweep): VERIFIED. PistonMovingBlockEntity.java:131
  sweeps getCollisionRelatedBlockState().getCollisionShape(...), and line 193 moves entities
  with entity.move(MoverType.PISTON, ...). Pumpkin entities/piston.rs:121-139 uses the 4-pixel
  head slab and pushed_block_state.get_block_collision_shapes_at(...), matching.
  (.vendor/vanilla-decompiled/net/minecraft/world/level/block/piston/PistonMovingBlockEntity.java)
- Remaining gap 3 (head slab anchoring): VERIFIED. PistonHeadBlock.java:37-40 defines
  SHAPE_PLATFORM = Block.boxZ(16,0,4) and Shapes.rotateAll keeps the 4-pixel plate on the
  facing side for all six directions; head_shape_aabb (entities/piston.rs:251-264) produces
  exactly those six slabs, locked by the pure test head_shape_slab_sits_on_facing_side.
  (.vendor/vanilla-decompiled/net/minecraft/world/level/block/piston/PistonHeadBlock.java)
- Remaining gap 4 (players excluded from pushes): BLOCKED-DATA. Missing subsystem: piston-aware
  client-move reconciliation. Vanilla PistonMovingBlockEntity.java:193 pushes every entity,
  players included, via entity.move(MoverType.PISTON, ...); Pumpkin skips players at
  entities/piston.rs:153-157 because entity/player/client_move.rs:306 overwrites entity.pos
  with the client absolute position on every accepted movement packet, so a server-side push
  is cancelled next tick. A correct fix needs piston-aware rejection/re-sync in the client-move
  validation path (extending reject_client_move): a whole movement-reconciliation subsystem
  spanning entity/player/client_move.rs and the piston block entity, outside a small edit.

No source behaviour changed this pass; no new test seam exists (the row-3 test already locks the
only formerly-STILL-OPEN row). rustfmt not needed (no source edits). No em dashes added.
