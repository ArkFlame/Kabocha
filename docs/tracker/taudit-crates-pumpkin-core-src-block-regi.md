# Tracker audit: crates/pumpkin-core/src/block/registry.rs

Coverage: AUDIT. Registry is mostly a dispatcher; vanilla parity lives in the
per-block `BlockBehaviour` impls. All 8 TODO/FIXME-style markers are doc comments
naming the vanilla class (`ActionResult`, `BlockPlaceContext`, `SUCCESS_SERVER`,
`CONSUME`, `FAIL`, `PASS`, `PASS_TO_DEFAULT_BLOCK_ACTION`) — no real TODO/FIXME.

## Fixed

### Registry obstruction check counted the placing player
`registry.rs:588-640`, `832-848`
Vanilla: `BlockItem.canPlace` -> `Level.isUnobstructed(state, pos, CollisionContext.of(player))`,
whose `isUnobstructed(AABB, @Nullable Entity except)` skips the context entity, so a
player may place a block into their own hitbox (feet/head). Pumpkin iterated
`world.players` without excluding the placer, so such placements silently failed.
Fix: pass the placer `entity_id` and skip it in both entity lists. Committed.

## Remaining gaps

### 1. `get_inside_collision_shape` fallback is a full block
`registry.rs:1598-1615` (and the identical trait default `block/mod.rs:249`)
Vanilla `BlockBehaviour.getInsideCollisionShape` returns
`state.getCollisionShape(level, pos)`; only `AbstractCauldronBlock`,
`EndPortalBlock`, `CampfireBlock`, `TripWireBlock`-like blocks override it to a
different shape. Returning `BoundingBox::full_block()` for every behaviour-less
block makes entities inside slabs/fences/stairs/waterlogged blocks behave as if
inside a full cube (suffocation/inside-block collision paths).
Fix: return the state's collision shape; needs a multi-AABB shape type (or the
existing shape API) in `BoundingBox` instead of a single box. Size: M.

### 2. `post_process_state` is dead and inverted
`registry.rs:1413-1439`
No callers in the tree. It queries `get_state_for_neighbor_update` with *this*
block's behaviour and *this* block's state/position, then writes the answer at
the neighbour position. Vanilla `BlockState.updateNeighbourShapes` calls the
neighbour's own `updateShape(state=neighbour, pos=neighbour, direction=towards
this block, neighbourPos=this block)`. Fix: use the neighbour's block id + state
and swap the neighbour/own arguments, or delete the helper. Size: S.

### 3. Relative-position replacement reports `BlockIsReplacing::None`
`registry.rs:681-704` vs `651-676`
Vanilla `BlockPlaceContext` has one boolean `replaceClicked`: both "clicked
block gives way" and "block at clickedPos.relative(face) gives way" set it.
Pumpkin reports `Other` for the first case and `None` for the second, so
consumers that treat `None` as "placing into free space" (torches.rs:41,
ladders.rs:20, abstract_wall_mounting.rs:43, redstone_torch.rs:112,
lanterns.rs:24, cocoa.rs:59, shelf_mushroom.rs:58, coral_fan.rs:60,
skull_block.rs:85) choose the wrong wall-facing preference when the replaced
block (tall grass, water, snow layer) is at the relative position.
Fix: return `Other` (and audit those consumers for `None` vs `Other`). Size: S/M.

### 4. `can_place_at` is asked about the default state, not the placed state
`registry.rs:779-791`
Vanilla `BlockItem.canPlace(context, blockState)` checks
`state.canSurvive(level, pos)` on the state produced by `getPlacementState`
(e.g. waterlogged/half variants). Pumpkin calls `on_place` only afterwards and
checks survival with `placed_block.default_state`, so states whose survival
depends on the chosen variant (waterlogged ladders/coral, half slabs) are
validated against the wrong shape.
Fix: compute `on_place` first and pass the resulting state to `can_place_at`.
Size: M (touches every block that implements `can_place_at`).

### 5. Entity filter approximates `Entity.blocksBuilding`
`registry.rs:588-617`
Vanilla `Level.isUnobstructed` filters `!isRemoved() && blocksBuilding`, and also
skips passengers of the placing vehicle (`isPassengerOfSameVehicle`). Pumpkin
whitelists living entities plus a hand-written type/resource-name list
(END_CRYSTAL, FALLING_BLOCK, TNT, `_minecart`, `_boat`, `_raft`). Works for the
audited cases but any new/empty-vehicle entity has to be added by hand.
Fix: back the check with a `blocks_building` flag on `EntityBase`, set per type
from vanilla `blocksBuilding = false` overrides. Size: M.

## Verified correct (no gap)

- `BlockActionResult::consumes_action` matches `InteractionResult.consumesAction`
  (`SUCCESS || SUCCESS_SERVER || CONSUME`), `registry.rs:510-515`.
- `bone_meal` matches `BoneMealItem.applyBonemeal`: valid target returns `true`
  even when the success roll fails, `is_bonemeal_success` only gates
  `perform_bonemeal`, and the Bukkit `BlockFertilizeEvent` cancel path returns
  `false` (item not consumed), `550-592`.
- Redstone defaults (`emits_redstone_power` false, weak/strong power 0) match
  `BlockBehaviour`, `1538-1596`.
- `mirror`/`rotate` fall back to `Block::mirror`/`Block::rotate`, same as vanilla
  `BlockBehaviour.mirror`/`rotate`, `1617-1641`.
- `is_pathfindable` fallback matches vanilla defaults (fluid tag for WATER,
  `!is_full_cube()` for LAND/AIR), duplicated identically in `block/mod.rs:266`.
- `update_neighbors` iterates `abstract_block_update_order` =
  WEST, EAST, NORTH, SOUTH, DOWN, UP, i.e. vanilla `UPDATE_SHAPE_ORDER`;
  `replace_with_state_for_neighbor_update` passes `direction.opposite()`, the
  vanilla neighbour-shape convention, `1487-1493`.
- `can_replace_with_other_block`: snow layer only when `layers == 1`, sculk vein
  always, else the state `replaceable` flag — matches the vanilla
  `canBeReplaced` overrides, `540-547`.
- `BlockPlacingError` height limits use `world.get_bottom_y()`/`get_top_y()`,
  matching `Level` build limits instead of a hardcoded 320, `743-760`.

## Gap status
- Registry obstruction check counting placing player: FIXED (per "Fixed" section; not re-spot-checked).
- 1. `get_inside_collision_shape` fallback (`block/registry.rs`): STILL OPEN (not re-read this pass).

## Result

- 1. get_inside_collision_shape fallback (block/registry.rs:1630, trait default block/mod.rs:249): LEFT OPEN. Fix needs files outside this unit (registry.rs, mod.rs) plus a multi-AABB shape type in BoundingBox (cross-crate refactor). Decompiled note: BlockBehaviour.getEntityInsideCollisionShape defaults to Shapes.block() (BlockBehaviour.java:336-338); only TripWireBlock/EndPortalBlock/LayeredCauldronBlock/LavaCauldronBlock/PowderSnowBlock override it, so the doc premise about returning state.getCollisionShape does not hold in this decompiled version and Pumpkin full-block fallback matches the default. No in-scope change.
- 3. relative-position BlockIsReplacing::None (gap-3 consumer files): no in-scope code change. Root fix is in block/registry.rs:705 (outside this unit). Decompiled BlockPlaceContext.getNearestLookingDirections (context/BlockPlaceContext.java:71-90) reorders looking directions only when replaceClicked is false (clicked block not replaceable, i.e. placement at the relative pos). The consumers == None condition already reorders for the relative-position air/replaceable case and skips it for the clicked-position case. The doc proposed None to Other change would suppress that reordering (air is replaceable via state.replaceable, so the relative branch would report Other and no consumer would reorder), breaking vanilla parity. Left open with reason: root cause in registry.rs, outside unit.
Files touched: none (all STILL OPEN rows are out of scope); no rustfmt, no pure-unit-test seam introduced.

## Re-verification (second pass)

- Only row 1 is marked STILL OPEN in "Gap status"; rows 2/4/5 live in registry.rs and row 3's root fix is registry.rs:705, all outside this unit's file list.
- Row 1 re-checked against decompiled BlockBehaviour.java:336-338: getEntityInsideCollisionShape defaults to Shapes.block() (a full block), so Pumpkin's full_block() fallback matches vanilla. No parity gap, and the fix would still need registry.rs + block/mod.rs + a multi-AABB shape type (cross-crate). LEFT OPEN, no in-scope change.
- Row 3 re-checked against decompiled BlockPlaceContext.java:71-90: reordering happens when replaceClicked == false (relative-pos placement). Consumer `== BlockIsReplacing::None` already fires for that case (torches.rs:41 and the other consumers). Changing None->Other as the doc proposed would suppress the required reorder and break parity. No in-scope change.
- No .rs files touched; no rustfmt, no pure-unit-test seam available (logic lives in registry.rs, outside this unit).

## Result (final pass)

Re-verified the single STILL OPEN row against decompiled vanilla; no in-scope edit exists.

- 1. `get_inside_collision_shape` fallback (`block/registry.rs`, trait default `block/mod.rs`): LEFT OPEN. Decompiled `BlockBehaviour.java:336-338` shows the default `getEntityInsideCollisionShape` returns `Shapes.block()` (a full block), so Pumpkin's `full_block()` fallback already matches vanilla for behaviour-less blocks; the doc's premise (default should be `state.getCollisionShape`) does not hold in this decompiled version. Even if it were a gap, the fix requires `registry.rs`, `block/mod.rs` and a multi-AABB shape type in `BoundingBox`, all outside this unit (cross-crate refactor). No in-scope change and no pure-unit-test seam (the logic lives outside the nine allowed consumer files).
- 3. relative-position `BlockIsReplacing::None` (nine allowed consumer files): no in-scope code change. Verified root cause is `block/registry.rs:714`, which returns `BlockIsReplacing::None` (vs `Other` for the clicked-block branch at line 683) when the block at the relative position is replaceable; `registry.rs` is outside this unit. Decompiled `LadderBlock.java:73-95`, `WallTorchBlock.java:53-69` and `LanternBlock.java:35-47` show these blocks derive wall/facing preference from `context.getNearestLookingDirections()`, which is gated on `replaceClicked`; the consumers' `== BlockIsReplacing::None` checks already reorder for the relative-position case and skip the clicked-position case. Changing the consumers without changing `registry.rs:714` would break that parity, so no edit was made.
- No source files in the unit were edited; no rustfmt or unit test needed. Status: BLOCKED for the one STILL OPEN row (out-of-scope files + non-gap per vanilla default), zero fixed.

## Result (consumer-side pass)

Row 3 (relative-position `BlockIsReplacing::None`) fixed at the consumer level, plus
placement-order parity for the standing/wall variants. Evidence: vanilla
`BlockPlaceContext.getNearestLookingDirections` (`.vendor/.../context/BlockPlaceContext.java:71-90`)
reorders to `clickedFace.getOpposite()` only when `replaceClicked` is false, i.e. when the
block lands at the clicked position relative to the clicked block;
`StandingAndWallBlockItem.getPlacementState` walks that single ordered list (DOWN asks for
the standing block, horizontals for the wall block, UP is skipped) and returns null when
nothing fits. Pumpkin's `placement_target` puts the block at the clicked position exactly
when the clicked block gives way, so `args.position != args.use_item_on.position` is the
faithful `!replaceClicked` predicate (the old `replacing == None` check missed the
relative-position water/snow/grass cases, which report `Other` or `Water`).

- 3. relative-position replacement: FIXED. The nine consumers now reorder on
  `args.position != args.use_item_on.position` instead of matching `BlockIsReplacing`;
  the ladder `Water` clause is covered by the new predicate, and its behind-ladder guard
  uses the same predicate. Registry enum semantics untouched (registry.rs is outside this
  unit). Size: S.
- torches.rs, redstone_torch.rs, coral_fan.rs, skull_block.rs: the out-of-order DOWN
  pre-checks (the `args.direction == Down` check before the reorder, plus the post-loop
  unconditional standing fallback) were replaced by a single ordered walk that asks for
  the standing state when DOWN is reached, matching StandingAndWallBlockItem. Standing
  skulls no longer require a non-replaceable block below (vanilla AbstractSkullBlock
  placement only sets POWERED). Test: `torches.rs`
  `prefer_facing_moves_the_clicked_face_first` covers the extracted reorder helper.
- 1. `get_inside_collision_shape`: still out of scope (registry.rs / block/mod.rs, needs a
  multi-AABB shape type); re-verified earlier that the vanilla default is `Shapes.block()`.
- Rows 2/4/5 remain registry.rs work, outside this unit.

Files: crates/pumpkin-core/src/block/blocks/torches.rs,
redstone/redstone_torch.rs, abstract_wall_mounting.rs, lanterns.rs, ladder.rs,
skull_block.rs, coral/coral_fan.rs, plant/cocoa.rs, plant/shelf_mushroom.rs.
Status: UNVERIFIED (no cargo run allowed for workers; rustfmt left to the orchestrator).

## Result (batch r2_002)
- Not reached in this pass: rows keep their existing status. Prose "STILL OPEN" mentions above are historical and were not re-verified against vanilla here.
- Row 1 (`get_inside_collision_shape` fallback): STILL OPEN, not reached in r2_002 (cross-file in `block/registry.rs`).

## Result (batch r3_014)
- 1. `get_inside_collision_shape` fallback: VERIFIED, no Rust change. Vanilla `BlockBehaviour.java:336-338` default `getEntityInsideCollisionShape` returns `Shapes.block()`, and `Entity.java:2037` treats that as full-block inside. Pumpkin `block/registry.rs:1627-1643` falls back to `BoundingBox::full_block()`, and the trait default is `block/mod.rs:305`. Overrides exist in `blocks/redstone/tripwire.rs:28`, `blocks/end_portal.rs:17`, `blocks/cauldron.rs:530`. The "Remaining gaps" premise (default should be `getCollisionShape`) does not hold for this decompiled version.
- 2. `post_process_state`: VERIFIED resolved. `grep -rni post_process crates/pumpkin-core/src` finds no registry helper and no callers.
- 3. relative-position replacement: FIXED earlier at consumer level (see "Result (consumer-side pass)"). Not re-changed this pass.
- 4. `can_place_at` on default state: STILL OPEN (not reached). Fix is M and cross-cutting (item BlockItem placement path, `world/mod.rs:9645` `WorldPortal::can_place_at`, `block/registry.rs:1213`), touching every block's `can_place_at`.
- 5. `blocks_building` entity filter: STILL OPEN (not reached). Fix is M and needs a per-type `blocks_building` flag on `EntityBase` plus vanilla `blocksBuilding = false` audit.
Files edited: none. Status: PARTIAL (row 1 and row 2 closed by verification, row 3 earlier; rows 4 and 5 open). No cargo or rustfmt run (workers may not compile).
