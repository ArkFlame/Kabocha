# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-block-blocks-piston-mod-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

Per-row results recorded below. No source edits were needed this pass: every fixable row was already closed in the tree and re-verified against the decompiled vanilla; the single remaining row needs a missing subsystem.

## Result

- Row 1 (non-cancellable destroy): VERIFIED. `PistonBaseBlock.move` destroys blocks
  unconditionally with `level.setBlock(pos, AIR, UPDATE_CLIENTS | UPDATE_KNOWN_SHAPE)` then
  `gameEvent(BLOCK_DESTROY)`, with no cancellable `BlockBreakEvent`
  (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/piston/PistonBaseBlock.java:368-369`).
  `piston.rs:433-449` destroys via `world.break_block_naturally(...)` plus
  `GameEvent::BlockDestroy`, matching vanilla: non-cancellable, with drops and the game event.
- Row 2 (collision-shape entity sweep): VERIFIED. `PistonMovingBlockEntity.moveCollidedEntities`
  sweeps `getCollisionRelatedBlockState().getCollisionShape(level, pos)` and moves entities with
  `entity.move(MoverType.PISTON, ...)`
  (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/piston/PistonMovingBlockEntity.java:131,193`).
  `entities/piston.rs:121-139` uses the 4-pixel head slab for the head BE and
  `pushed_block_state.get_block_collision_shapes_at(...)` for pushed blocks, matching.
- Row 3 (head slab anchoring): VERIFIED. `PistonHeadBlock.SHAPE_PLATFORM = Block.boxZ(16, 0, 4)`
  and `Shapes.rotateAll` keep a 4-pixel plate on the facing side for all six directions
  (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/piston/PistonHeadBlock.java:38-40`).
  `PistonBlockEntity::head_shape_aabb` (`entities/piston.rs:251-264`) produces exactly those six
  slabs, locked by the pure test `head_shape_slab_sits_on_facing_side` (`entities/piston.rs:452-481`).
- Row 4 (players excluded from piston pushes): BLOCKED-DATA. Missing subsystem: piston-aware
  client-move reconciliation. Vanilla pushes every entity with `PushReaction != IGNORE_ENTITY`,
  players included, via `entity.move(MoverType.PISTON, ...)`
  (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/piston/PistonMovingBlockEntity.java:128-193`).
  Pumpkin is deliberately client-authoritative: `apply_client_move` overwrites `entity.pos` with
  the client's absolute position on every accepted movement packet
  (`entity/player/client_move.rs:306`), so a naive server-side push is cancelled next tick. A fix
  needs piston-aware rejection/re-sync in the client-move validation path (extending
  `reject_client_move`, `client_move.rs:155`), a whole subsystem spanning `client_move.rs` and the
  piston block entity, outside a small edit.

No behaviour change, no new test seam; the existing row-3 test already locks the formerly-open row.
rustfmt not needed (no source edits). No em dashes added.
