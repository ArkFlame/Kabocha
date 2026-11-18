# Tracker: blocks / Pistons (RE-REVIEW)

Re-reviewed after the source changed since the last review (`60bdcacf1`, drop base item when an
extended head is broken). Two piston commits landed afterwards:

- `bb9367799` `fix(block): match vanilla piston moving block nbt facing and final state` -
  moving-block `facing` is now a vanilla int NBT tag, and the terminal tick gained the
  `is_air()` place-then-break branch (old gap #5).
- `a43ff9353` `fix(piston): follow vanilla structure resolver sweep and piston pos checks` -
  renamed the handler to the vanilla `PistonStructureResolver` shape (`extending`, `addBlockLine`,
  `addBranchingBlocks`, `isPistonPos`), moved `Destroy`-start handling to extend-only, and sweeps
  appended sticky blocks with a live `while i < len()` bound.

No decompiled vanilla tree is present in this worktree, so comparisons below are against the
vanilla class semantics for `PistonBaseBlock` / `PistonHeadBlock` / `MovingPistonBlock` /
`PistonBlockEntity` / `PistonStructureResolver` and `docs/VANILLA_REF.md:22`.

Files audited:

- `crates/pumpkin-core/src/block/blocks/piston/piston.rs`
- `crates/pumpkin-core/src/block/blocks/piston/mod.rs` (`PistonHandler`, now
  `PistonStructureResolver`-shaped)
- `crates/pumpkin-core/src/block/blocks/piston/piston_head.rs`
- `crates/pumpkin-core/src/block/blocks/piston/piston_extension.rs`
- `crates/pumpkin-core/src/block/entities/piston.rs` (`PistonBlockEntity`)

Overall: the port is close. Event flow (`handle_synced_block_event` vs `onSyncedBlockEvent`
extend/contract/drop ordering), `calculate_push` / `add_block_line` (12-block limit
`i + moved_blocks.len() > 12` on the growing vector, sticky chains, `set_moved_blocks`,
destroy-vs-block reactions) and the retract fast-retract `TRIGGER_DROP` gate
(`last_progress < 0.5`, same world age, handling tick) all match vanilla branch for branch.
#3144 (`piston_head.rs:38-56`, drops the base once from `on_state_replaced` and forwards
`SKIP_DROPS`/`MOVED`) is correct. Old gap #5 is fixed by the `is_air()` branch in
`entities/piston.rs:238-252`.

## Fixed in this pass

| # | Where | Gap | Vanilla class | Fix | Size |
|---|-------|-----|---------------|-----|------|
| 1 | `piston.rs:54-65` | `is_movable` never consulted the world border, so pistons could push/retract blocks outside it | `PistonBlock.isMovable` (`!world.getWorldBorder().contains(pos)`) | Added the `contains_block` early return after the height-limit check | S |
| 2 | `entities/piston.rs:238-252` | Tick finalisation lacked the moved-block air case; a block that cannot survive at the destination was not identified and shattered | `PistonBlockEntity.finish` / terminal `tick`: place `pushedBlockState` then `breakBlock` so it drops | Added the `new_state.to_state().is_air()` place-then-break branch (landed in `bb9367799`) | S |
| 3 | `mod.rs:52-90` | Handler did not reproduce vanilla `resolve()` ordering: `Destroy` start handling was not extend-gated, and the branch sweep froze `len()` so newly appended sticky blocks were not swept | `PistonStructureResolver.resolve` / `addBranchingBlocks` | `extending && ... Destroy` gate plus a live `while i < self.moved_blocks.len()` sweep (landed in `a43ff9353`) | M |

## Remaining gaps

| # | file:line | Gap | Vanilla class / behaviour | Suggested fix | Size |
|---|-----------|-----|---------------------------|---------------|------|
| 1 | `piston.rs:426-437` | Destroyed blocks are removed through `World::break_block`, which fires a cancellable `BlockBreakEvent`; a cancelled break leaves the move half-applied (the code already carries a `TODO` saying so) | `PistonBaseBlock.move` calls `world.breakBlock(pos, true)` unconditionally; that break is not cancellable | Use a non-cancellable destroy path for `PistonBehavior::Destroy`, keep the `GameEvent::BlockDestroy` emission | M |
| 2 | `entities/piston.rs:57-82` | Entity sweep uses `BoundingBox::from_block(&self.position)` (full cube) for non-head pushed blocks; vanilla sweeps the moved block's `VoxelShape`, so stairs, slabs and fences push less (or not at all) | `PistonBlockEntity.moveCollidedEntities` + `state.getCollisionShape(world, pos)` | Compute the collision shape of `pushed_block_state` and use it for the swept volume instead of the full block AABB | M |
| 3 | `entities/piston.rs:165-178` | Head slab is anchored on the far face: `East => min.x = max.x - 0.25` (x 0.75-1.0), while the base sits west of the head. Vanilla `PistonHeadBlock.EAST_AABB` is `Block.box(0.0, 0.0, 0.0, 4.0, 16.0, 16.0)`, i.e. x 0.0-0.25 (base side); the short-head variants (`SHORT_EAST_AABB` = `Block.box(0.0, 0.0, 0.0, 8.0, 16.0, 16.0)`) are not modelled either | `PistonHeadBlock.EAST_AABB` / `WEST_AABB` / `UP_AABB` / `DOWN_AABB` / `SOUTH_AABB` / `NORTH_AABB` and their `SHORT_*` counterparts | Anchor the slab on the base-side face for all six facings and re-check end-crystal push behaviour | S |
| 4 | `entities/piston.rs:88-93` | Players are excluded from piston pushes | `PistonBlockEntity.moveCollidedEntities` moves players via `Entity.move(PISTON)` | Intentional divergence (client-authoritative movement, `docs/LEDGER.tsv` #1679); revisit only with movement reconciliation | M |

## Checked and matching (no action)

- `should_extend` (`piston.rs:328-350`): power from the facing side is ignored, power from above and
  its non-down neighbours is honoured - the #3833 "powered from the head" gate.
- `calculate_push` / `add_block_line` early outs (`mod.rs:46-90`): air, piston base/head, duplicate
  position, 12-block limit on the growing vector, `Destroy` when retracted, `PushOnly` only along
  the piston axis.
- Sticky chain order (`mod.rs:161-166`): `for k in (0..i).rev()` with
  `motion_direction.opposite()` matches vanilla `addBlockLine`, which first walks
  `pos.relative(motionDirection.getOpposite(), i)` for the chain then appends
  `pos.relative(motionDirection.getOpposite(), k)` for `k = i-1..0` - same reversed order, so the
  old "append ascending" note was wrong (old gap #6 removed).
- Retract ordering (`piston.rs:204-298`): `finish()` of a still-animating head, moving-piston base
  swap, sticky re-pull gate, `TRIGGER_DROP` fast-retract detection.
- `piston_extension.rs` and `piston_head.rs:58-99` (`get_state_for_neighbor_update` -> air when
  the base stops fitting) cover head removal after the base is destroyed or replaced.
- Moving-block NBT (`entities/piston.rs:273-310`): `facing` stored/read as an int with a byte
  fallback, matching vanilla `PistonMovingBlockEntity`.

## h6 gap status
- Row 1 (world border in `is_movable`): FIXED in tree at HEAD (`piston.rs:62` `contains_block` check).
- Row 2 (air moved-block place then break): FIXED bb9367799 (as the note states).
- Row 3 (head slab anchored on far face): STILL OPEN, needs the six `PistonHeadBlock` AABBs and end-crystal re-check.


## Result

Row 3 (head slab anchoring, the only row marked STILL OPEN): VERIFIED against the current decompiled vanilla, no code change needed.

- Vanilla `PistonHeadBlock` (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/piston/PistonHeadBlock.java:37-40`) builds the head collision shape from `SHAPE_PLATFORM = Block.boxZ(16.0, 0.0, 4.0)` (x 0-16, y 0-16, z 0-4) plus a thin arm, then `Shapes.rotateAll` (Shapes.java:504-520) maps NORTH to identity and EAST to `BLOCK_ROT_Y_90` which is `ROT_90_Y_NEG`. The platform therefore sits on the side the head FACES for every direction: EAST x 0.75-1.0, WEST x 0-0.25, UP y 0.75-1.0, DOWN y 0-0.25, SOUTH z 0.75-1.0, NORTH z 0-0.25.
- `PistonBlockEntity::head_shape_aabb` (`entities/piston.rs:165-178`) already produces exactly these six slabs, so it matches current vanilla. The tracker's cited `EAST_AABB = box(0,0,0,4,16,16)` and 8-pixel `SHORT_*` variants are an older MC shape convention and do not apply to the decompiled target. Current vanilla keeps the platform 4 pixels thick for both normal and short heads; `SHORT` only changes the arm length (z 4-16 vs z 4-20), which Pumpkin intentionally does not model for the sweep.
- End-crystal push geometry is unchanged, so no re-check is needed.

Added pure unit test `head_shape_slab_sits_on_facing_side` (`entities/piston.rs:347-376`) that locks all six directions so a future review does not "fix" already-correct code.

Other remaining gaps left open:
- Row 1 (non-cancellable destroy): needs a new non-cancellable destroy path in `World`, a cross-crate refactor outside this unit's files.
- Row 2 (collision-shape entity sweep): needs a `VoxelShape`/collision-shape API from another crate, cross-crate refactor.
- Row 4 (players excluded from pushes): intentional divergence, client-authoritative movement (`docs/LEDGER.tsv` #1679).

Files touched: `crates/pumpkin-core/src/block/entities/piston.rs` (test/import only, no behaviour change); `rustfmt --edition 2024` ran clean on it.


## Result (recheck)

Only row 3 of the h6 gap status is marked STILL OPEN: the piston-head collision slab anchoring in `entities/piston.rs`. Rechecked against the decompiled target and it is already correct, so no code change was needed.

- `PistonHeadBlock.SHAPE_PLATFORM` is `Block.boxZ(16, 0, 4)` = box(0,0,0,16,16,4) (Block.java:200-212), a 4-pixel plate on the NORTH/front face. `Shapes.rotateAll` (Shapes.java:504-520) puts that plate on the facing side for every direction: NORTH z 0-0.25, SOUTH z 0.75-1.0, EAST x 0.75-1.0 (`BLOCK_ROT_Y_90` = `ROT_90_Y_NEG`, OctahedralGroup.java:70), WEST x 0-0.25, UP y 0.75-1.0 (`BLOCK_ROT_X_270` = `ROT_90_X_POS`), DOWN y 0-0.25.
- `PistonBlockEntity::head_shape_aabb` (`entities/piston.rs:188-201`) produces exactly these six slabs, and the pure test `head_shape_slab_sits_on_facing_side` locks all six directions. The tracked `EAST_AABB = box(0,0,0,4,16,16)` and `SHORT_*` 8-pixel variants come from an older MC convention and do not apply here. Current vanilla keeps the plate 4 pixels thick for both normal and short heads; `SHORT` only shortens the arm (z 4-16 vs 4-20), which Pumpkin intentionally does not model for the sweep.
- End-crystal push geometry is unchanged, so no recheck is needed.

Other rows remain as previously recorded:

- Row 2 (collision-shape entity sweep): now implemented in the current tree via `get_block_collision_shapes_at` in `push_entities` (`entities/piston.rs:78-86`), matching vanilla `PistonMovingBlockEntity.moveCollidedEntities` (line 131). No action left.
- Row 1 (non-cancellable destroy): still open. Needs a non-cancellable destroy API on `World`, a cross-crate refactor outside this unit's files.
- Row 4 (players excluded from pushes): intentional divergence because movement is client-authoritative (`docs/LEDGER.tsv` #1679).

No source files were modified this pass; rustfmt was not needed.

## Result (vanilla re-verification, decompiled tree available)

Only h6 row 3 was marked STILL OPEN; re-checked every "Remaining gaps" row against the
decompiled vanilla now present under `.vendor/vanilla-decompiled/`. All four rows are already
closed in the current tree, so no source files were modified and no new test was needed (the
row-3 test already exists).

- Row 3 (head slab anchoring): CLOSED, already correct. `PistonHeadBlock.java:38` defines
  `SHAPE_PLATFORM = Block.boxZ(16.0, 0.0, 4.0)`; `Block.boxZ` (Block.java:200-212) expands that
  to `box(0,0,0,16,16,4)`, a 4-pixel plate on the NORTH/front face, and `Shapes.rotateAll`
  keeps it on the facing side for all six directions. `PistonBlockEntity::head_shape_aabb`
  (`entities/piston.rs:251-264`) produces exactly those six slabs, locked by the existing pure
  test `head_shape_slab_sits_on_facing_side` (`entities/piston.rs:452-481`). No change needed.
- Row 1 (non-cancellable destroy): CLOSED in tree, not open. `PistonBaseBlock.move`
  (PistonBaseBlock.java:359-370) destroys unconditionally via `dropResources` +
  `setBlock(AIR, UPDATE_CLIENTS|UPDATE_KNOWN_SHAPE)` + `gameEvent(GameEvent.BLOCK_DESTROY)`.
  `piston.rs:422-439` now uses `world.break_block_naturally(...)` plus
  `GameEvent::BlockDestroy`, i.e. non-cancellable with drops and the game event. Matches.
- Row 2 (collision-shape entity sweep): CLOSED in tree, not open. Vanilla
  `PistonMovingBlockEntity.moveCollidedEntities` (line 131) sweeps
  `getCollisionRelatedBlockState().getCollisionShape(...)` for every sub-shape.
  `entities/piston.rs:124-139` now uses the 4-pixel head slab for the head BE and
  `pushed_block_state.get_block_collision_shapes_at(...)` for pushed blocks. Matches.
- Row 4 (players excluded): LEFT OPEN. One-line reason: intentional divergence because player
  movement is client-authoritative (`docs/LEDGER.tsv` #1679); fixing needs cross-crate movement
  reconciliation and entity `Entity.move(PISTON)` behavior outside this unit's three files.

No behaviour change, so no unit test seam was added; the existing row-3 test already locks the
only STILL OPEN row. rustfmt not needed (no source edits).

## Result (second pass, cross-file edits permitted)

Re-verified every "Remaining gaps" row against the decompiled vanilla. The two rows that earlier
passes left open as cross-file/outside-unit are now CLOSED in the current tree, so nothing remains
for this pass to fix:

- Row 1 (non-cancellable destroy): CLOSED. `piston.rs:422-439` destroys via
  `World::break_block_naturally` plus `GameEvent::BlockDestroy`, matching vanilla
  `PistonBaseBlock.move` (unconditional `setBlock(AIR, ...)`, no cancellable `BlockBreakEvent`).
- Row 2 (collision-shape entity sweep): CLOSED. `entities/piston.rs:124-139` uses
  `pushed_block_state.get_block_collision_shapes_at` for pushed blocks and the 4-pixel head slab
  for the head, matching `PistonMovingBlockEntity.moveCollidedEntities` (`getCollisionShape`).
- Row 3 (head slab anchoring): CLOSED, already correct, locked by the existing pure test
  `head_shape_slab_sits_on_facing_side`.

Row 4 (players excluded from piston pushes) remains open and is BLOCKED here: it requires a whole
player movement-reconciliation subsystem. Vanilla `moveCollidedEntities` pushes every entity with
`PushReaction != IGNORE_ENTITY` via `Entity.move(PISTON)`
(`PistonMovingBlockEntity.java:139-193`), players included. Pumpkin is deliberately
client-authoritative: `apply_client_move` (`entity/player/client_move.rs:298-306`) overwrites
`entity.pos` with the client's absolute position on every movement packet, so a naive server-side
push is cancelled next tick and causes position flip-flop. A correct fix needs piston-aware
rejection/re-sync in the client-move validation path (extending the existing `reject_client_move`
correction), a cross-cutting subsystem outside this unit's files and outside a small edit. Left for
a dedicated movement-reconciliation task; client-authoritative movement is a documented
intentional vanilla difference.

No source files modified (all fixable rows already closed); no new test seam exists. rustfmt not
needed.
