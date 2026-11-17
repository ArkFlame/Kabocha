# Tracker: audit / crates/pumpkin-core/src/block/registry.rs

Deep audit of the central block behaviour dispatcher plus `BlockPlaceContext`
emulation (`placement_target`) and `BlockItem.place` emulation (`place_block`).
Vanilla sources cross-checked from knowledge (no `.vendor/` checkout in this
worktree): `BlockBehaviour`, `AbstractBlock$BlockState`, `BlockItem`,
`BlockPlaceContext`, `Level.isUnobstructed`, `Entity.blocksBuilding`.

No TODO/FIXME/todo! markers exist in the file; the 8 "vanilla" hits are doc
comments naming the mirrored Java members (`BlockActionResult`, `BlockPlaceContext`).

## Fixed in this pass

| Gap | Rust (before) | Vanilla | Fix | Size |
|---|---|---|---|---|
| Placing a block never emitted `GameEvent.BLOCK_PLACE`, so sculk sensors/warden did not hear player placement | `place_block` (now 890) | `BlockItem.place`: `level.gameEvent(GameEvent.BLOCK_PLACE, pos, Context.of(player, state))` | `emit_game_event_with_context(GameEvent::BlockPlace, pos, source_entity = player)` after `set_block_state` | S |
| Build-limit check used the clicked neighbour (`location + face`), rejecting legal placements that replace a block at the top/bottom layer and using the wrong position for the `build.tooHigh` message | `registry.rs:742-758` (before) | `BlockItem.place` checks `BlockPlaceContext.getClickedPos()`, i.e. the destination | bounds checks moved after `placement_target` and applied to `final_block_pos` | S |
| Default `isPathfindable` treated every non-full cube as walkable, so fences, chests, walls, farmland (solid, non-full-cube, some without behaviours) were pathfindable | `registry.rs:1662` (before) | `BlockBehaviour.isPathfindable` default: `case LAND, AIR -> !state.isSolid()` | use `!state.is_solid()`; unit test `default_pathfinding_uses_motion_blocking` | S |

Event order kept: `BlockCanBuildEvent` -> `BlockPlaceEvent` -> `setBlock` -> sound ->
game event -> block entity data -> `setPlacedBy` (`player_placed`) -> advancement,
matching `BlockItem.place` (game event is fired right after the successful `setBlock`).

## Remaining gaps (not fixed)

| Gap | Rust | Vanilla / Paper | Suggested fix | Size |
|---|---|---|---|---|
| `can_update_at` gets the raw clicked face in the "clicked block replaced" branch but the opposite face in the "same block at destination" branch | `registry.rs:651` vs `registry.rs:681` | One notion only: `BlockPlaceContext.getClickedFace()` (raw). `SlabBlock.canBeReplaced` expects the raw face; `MultifaceBlock`/glow lichen expects the direction from the new block towards the support | Pick one convention for `CanUpdateAtArgs.direction` (raw clicked face is vanilla), convert in `placement_target`, then audit `slabs.rs:62-78` and `sculk/sculk_vein.rs:76-87`; one of the two must be wrong today | M |
| `BlockIsReplacing::Other` vs `::None` chosen inconsistently between the two replacement branches | `registry.rs:672` (`Other`) vs `registry.rs:703` (`None`) | `BlockPlaceContext` only exposes `replaceClicked` (true only in the first branch) plus the replaced state's fluid state | Keep `None` for `replaceClicked == false` but return `Other` when the destination held a replaceable non-air block (grass, snow, sculk vein), because `torches.rs:41`, `ladder.rs:20`, `lanterns.rs:24` use `== None` to mean "placed against the clicked face" | M |
| `post_process_state` is dead code and applies the *changed* block's `get_state_for_neighbor_update` to its neighbours | `registry.rs:1420-1435` | `BlockState.updateNeighbourShapes` asks each *neighbour's* block for its new state (`neighborState.getStateForNeighborUpdate(direction.getOpposite(), self, ...)`) in `UPDATE_SHAPE_ORDER` (WEST, EAST, NORTH, SOUTH, DOWN, UP) and removes blocks whose neighbour state is air | Delete it (no caller in `crates/`) or reimplement as above; the live path is `World::set_block_state` -> `registry.update_neighbors` -> `World::replace_with_state_for_neighbor_update` | S |
| Default inside-collision shape is always a full block | `registry.rs:1605-1616` | `BlockBehaviour.getInsideCollisionShape(state, level, pos)` defaults to the state's collision shape at that position | Build the union of `state.get_block_collision_shapes_at(pos)` instead of `BoundingBox::full_block()` | S |
| `isPathfindable` default for `PathComputationType::Water` returns true for any water-tagged fluid state, including waterlogged non-water blocks without behaviour | `registry.rs:1659` | `LiquidBlock.isPathfindable` is true only for the water path type; the base default is `false` and waterlogged blocks opt in via their own behaviour (`SlabBlock` uses `WATERLOGGED`) | Restrict the tag check to water/flowing-water blocks, or default to `false` and let `LiquidBlock` opt in; check waterlogged blocks without a behaviour | S |
| Obstruction test does not exclude the placing entity | `registry.rs:835-850` (`has_blocking_entity_in_box`) | `BlockItem.canPlace` -> `Level.isUnobstructed(state, pos, context.getPlayer())`; the nullable entity argument is excluded from the candidate list | Thread `Some(player)` into `has_blocking_entity_in_box` and skip that entity | S |
| Only the Bedrock `place` level sound is played | `registry.rs:884-888` | Java clients get `SoundType.getPlaceSound` via `Level.playSound` from the placed state's sound type | Also fire `world.play_sound(block.sound_type.place_sound, Blocks, ...)`; needs a `SoundType` lookup in pumpkin-data | S |
| `player_placed` receives the raw clicked face while `on_place` receives the opposite face | `registry.rs:923` (`face`) vs `registry.rs:797` (`final_face`) | `setPlacedBy` has no face parameter; overrides that care use the block state | Document what `PlayerPlacedArgs.direction` means (unused by every implementation) and pass `final_face` if it is meant to match `OnPlaceArgs` | S |

## Verified matching vanilla (no change)

- `placement_target`: replace clicked when the same block allows update (`can_update_at`),
  when the state is replaceable, or when it is a single snow layer / sculk vein;
  otherwise offset by the clicked face. `face.opposite()` is the `on_place` direction
  convention (verified against stairs `Up => Top`, torches `Down => upright`, slab `Up => Top`).
- Water replacement records the fluid level so `water_source()` drives waterlogging.
- `entity_blocks_block_placement` mirrors `Entity.blocksBuilding`: living entities,
  end crystal, falling block, TNT, minecarts, boats and rafts block placement;
  removed/no-physics/spectator entities do not.
- Adventure mode requires `CanPlaceOn` for the clicked block; spectator cannot place;
  spawn protection and `build.tooHigh` messages are sent as in vanilla.
- `can_replace_with_other_block`: a single snow layer yields to another item, sculk vein
  is replaceable despite its state flag, everything else follows the `replaceable` tag.
- Shape updates (`update_neighbors`) and neighbour notifications
  (`World::update_neighbors_at`) are correctly split like `updateNeighbourShapes` vs
  `updateNeighborsAt`.
- `on_landed_upon` fallback reduces the fall distance by the data-driven
  `fall_distance_reduction` then applies vanilla's 1.0 multiplier; blocks that override
  (hay, honey, farmland, turtle egg, dripstone, powder snow) keep their own logic.
- `mirror`/`rotate` fall back to the generated block data, `emits_redstone_power`
  defaults to `false`, weak/strong redstone power default to 0, and behaviour lookups
  return the vanilla default values when no behaviour is registered.
