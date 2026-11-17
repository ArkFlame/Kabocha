# block registry — deep audit

Source: crates/pumpkin-core/src/block/registry.rs (1769 lines) plus the `BlockBehaviour` defaults it
falls back to in crates/pumpkin-core/src/block/mod.rs.
Compared against vanilla `BlockBehaviour`, `BlockPlaceContext`, `BlockItem`, `BoneMealItem`,
`BlockState.updateNeighbourShapes` and `SnowLayerBlock.canBeReplaced`; Paper/Bukkit event contract
(BlockPlaceEvent, BlockCanBuildEvent, BlockFertilizeEvent). No decompiled sources on this machine,
so items below marked *verify* rest on class-level knowledge, not grep of `.vendor/`.

Note: the file has no TODO/FIXME markers. The 9 "vanilla" hits are doc comments that describe the
mirrored vanilla method (`ActionResult`, `BlockPlaceContext`, `BlockItem.canPlace`, `isPathfindable`, ...).

## Fixed in this audit
- `BlockBehaviour::is_pathfindable` default used `!state.is_full_cube()` (block/mod.rs:282) while the
  registry fallback used `!state.is_solid()` (registry.rs:1691). Vanilla defaults to `!state.isSolid()`,
  so every registered behaviour without an override (fences.rs `FenceBlock`, walls, panes, bars, ...)
  was pathfindable through motion-blocking non-full cubes: mobs could path straight through fences.
  Default now `!state.is_solid()`; regression test `registered_motion_blocking_block_is_not_pathfindable`
  in registry.rs drives `default_registry()` and OAK_FENCE (fails before the fix, passes after).

## Remaining gaps
| # | Location | Vanilla | Gap | Size |
|---|---|---|---|---|
| 1 | registry.rs:1678-1695 and block/mod.rs:277-284 (Water arm) | `BlockBehaviour.isPathfindable`: `WATER -> !state.blocksMotion()`, i.e. air and non-colliding plants are pathfindable in water | Pumpkin returns true only when the state's own fluid is water, so a dolphin/fish at the surface cannot path into the air block above the water (the swim evaluator already gates on water path types), and waterlogged solid blocks that do report the water tag are wrongly pathfindable. Use `!state.is_solid()` for Water too. *verify* against decompiled source before changing; aquatic-mob behaviour needs a gametest. | M |
| 2 | registry.rs:1444-1470 `post_process_state` | `BlockState.updateNeighbourShapes`: for each neighbour, ask the **neighbour's** `updateShape(direction.opposite(), sourceState)` and write the result to the neighbour | Dead code (no caller in crates/), and wrong if it ever gets one: it queries `self.get_pumpkin_block(block.id)` (the changed block) and passes the changed block's `state_id`/`position` for every neighbour. Delete it, or rewrite to fetch `world.get_block(&neighbor_pos)` and pass `position = neighbor_pos`, `state_id = neighbor_state_id`, `direction = direction.opposite()`. | M |
| 3 | registry.rs:792-806 | `BlockItem.canPlace`: survival and `isUnobstructed` are tested on the **placement state** returned by `getStateForPlacement` | The `must_survive` + `can_place_at` check runs on `placed_block.default_state` before `on_place()` computes the real state, so blocks whose survival depends on orientation/half (wall-mounted, double plants, door halves) are validated against the wrong state. Move the check after `new_state` is computed and pass `BlockState::from_id(new_state)`. | S-M |
| 4 | registry.rs:547-554 `can_replace_with_other_block` | `SnowLayerBlock.canBeReplaced`: `layers == 1` only for the snow item, otherwise `super.canBeReplaced` -> `state.replaceable()`, which is true for every snow layer | Pumpkin hardcodes `layers == 1` for every item, so placing a block on a 2-7 layer snow is refused (vanilla replaces the snow entirely). *verify* with the generated `replaceable()` flag for snow states, then drop the special case or gate it on the held item. The same-block branch is already handled by `can_update_at`. | M |
| 5 | registry.rs:1632-1649 + block/mod.rs:252-254 | Default `getInsideCollisionShape` returns the state's collision shape; only `ScaffoldingBlock` overrides to a full block | Default is `BoundingBox::full_block()`, so every block without an override (slabs, trapdoors, snow layers, carpet) is treated as a full cube in entity/mod.rs:3194 for on-inside-block/collision effects. *verify* the vanilla default first (tripwire already overrides in Pumpkin). | M |
| 6 | registry.rs:557-593 `bone_meal` | Bukkit `CraftEventFactory.callBlockFertilizeEvent` / `BlockFertilizeEvent.getBlocks()` lists the states the growth actually changed | Event is fired with `vec![]` before `perform_bonemeal`, so plugins can never see the resulting states (Paper region protection reads them). Needs the perform path to report changed positions. | S-M (plugin API) |
| 7 | registry.rs:1361-1364 `on_landed_upon` fallback | `BlockBehaviour.onLandedUpon` = `entity.causeFallDamage(fallDistance, 1.0F, fall())` for any Entity; per-block reductions (hay: 0.2) are overrides | Fallback only handles living entities and applies `1.0 - block.fall_distance_reduction` from generated data. Fine for mobs/players, silently skips non-living entities that override fall damage (`ItemEntity`, `TNTPrimitive`). Check where `fall_distance_reduction` comes from before touching, then route through `entity.cause_fall_damage`. | S |
| 8 | registry.rs:646-712 `placement_target` | `ServerPlayerGameMode.useItemOn` rejects placement outside the world border (`Player.mayUseItemAt`) | Only `is_in_height_limit` and spawn protection are checked in `place_block`; no world-border gate. Client prediction also differs (client rejects at the border), so this shows as a ghost block. | S |
| 9 | registry.rs:320 and 382 | n/a | `TestBlock` is registered twice in `default_registry()`, burning a behaviour index. Harmless; delete the second line. | trivial |
| 10 | registry.rs:1560-1570 `get_pumpkin_fluid` | Flowing and still fluids share behaviour | Hardcoded fluid ids 2/4 instead of resolving still from the flowing id (or the other way round). Breaks silently if generated fluid ids are reordered. | S |

Checked, no change: `BlockActionResult` variant set and `consumes_action()` match `InteractionResult`; `place_block` gamemode (spectator/adventure `CanPlaceOn`), spawn protection, `BlockCanBuildEvent` and `BlockPlaceEvent` ordering, `GameEvent.BLOCK_PLACE` with the placer context, block-entity NBT merge from the `block_entity_data` component, `player_placed`/advancement trigger; `has_blocking_entity_in_box`/`entity_blocks_block_placement` approximate `isUnobstructed` (`!isRemoved && !isSpectator && blocksBuilding`, placer skipped) closely; `update_neighbors` uses the vanilla `UPDATE_SHAPE_ORDER` (west, east, north, south, down, up); mirror/rotate and `get_state_for_neighbor_update` delegate with the same fallbacks; redstone power defaults return 0 as in `BlockBehaviour`.
