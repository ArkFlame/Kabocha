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
