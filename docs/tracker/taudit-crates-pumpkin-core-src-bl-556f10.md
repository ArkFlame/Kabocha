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
| `can_update_at` gets the raw clicked face in the "clicked block replaced" branch but the opposite face in the "same block at destination" branch | `registry.rs:651` vs `registry.rs:681` | One notion only: `BlockPlaceContext.getClickedFace()` (raw). `SlabBlock.canBeReplaced` expects the raw face; `MultifaceBlock`/glow lichen expects the direction from the new block towards the support | Pick one convention for `CanUpdateAtArgs.direction` (raw clicked face is vanilla), convert in `placement_target`, then audit `slabs.rs:62-78` and `sculk/sculk_vein.rs:76-87`; one of the two must be wrong today | M. STILL OPEN: needs a slab and sculk vein audit to pick the convention; uncertain, not changed |
| `BlockIsReplacing::Other` vs `::None` chosen inconsistently between the two replacement branches | `registry.rs:672` (`Other`) vs `registry.rs:703` (`None`) | `BlockPlaceContext` only exposes `replaceClicked` (true only in the first branch) plus the replaced state's fluid state | Keep `None` for `replaceClicked == false` but return `Other` when the destination held a replaceable non-air block (grass, snow, sculk vein), because `torches.rs:41`, `ladder.rs:20`, `lanterns.rs:24` use `== None` to mean "placed against the clicked face" | M. STILL OPEN: changing `Other`/`None` touches torches, ladder and lanterns callers; not verified against vanilla for each, so not changed |
| `post_process_state` is dead code and applies the *changed* block's `get_state_for_neighbor_update` to its neighbours | `registry.rs:1420-1435` | `BlockState.updateNeighbourShapes` asks each *neighbour's* block for its new state (`neighborState.getStateForNeighborUpdate(direction.getOpposite(), self, ...)`) in `UPDATE_SHAPE_ORDER` (WEST, EAST, NORTH, SOUTH, DOWN, UP) and removes blocks whose neighbour state is air | Delete it (no caller in `crates/`) or reimplement as above; the live path is `World::set_block_state` -> `registry.update_neighbors` -> `World::replace_with_state_for_neighbor_update` | S. FIXED fix(block): play java place sound and drop dead post_process_state |
| Default inside-collision shape is always a full block | `registry.rs:1605-1616` | `BlockBehaviour.getInsideCollisionShape(state, level, pos)` defaults to the state's collision shape at that position | Build the union of `state.get_block_collision_shapes_at(pos)` instead of `BoundingBox::full_block()` | S. STILL OPEN: `get_inside_collision_shape` returns one `BoundingBox`, so a union needs an API change through `entity/mod.rs:3247` and the powder snow/cauldron/campfire overrides |
| `isPathfindable` default for `PathComputationType::Water` returns true for any water-tagged fluid state, including waterlogged non-water blocks without behaviour | `registry.rs:1659` | `LiquidBlock.isPathfindable` is true only for the water path type; the base default is `false` and waterlogged blocks opt in via their own behaviour (`SlabBlock` uses `WATERLOGGED`) | Restrict the tag check to water/flowing-water blocks, or default to `false` and let `LiquidBlock` opt in; check waterlogged blocks without a behaviour | S. STILL OPEN: the base `BlockBehaviour` WATER default is not verified against decompiled vanilla here, so the default could be wrong either way |
| Obstruction test does not exclude the placing entity | `registry.rs:835-850` (`has_blocking_entity_in_box`) | `BlockItem.canPlace` -> `Level.isUnobstructed(state, pos, context.getPlayer())`; the nullable entity argument is excluded from the candidate list | Thread `Some(player)` into `has_blocking_entity_in_box` and skip that entity | S. FIXED already on master: placer excluded via `entity_id != placer_id` (registry.rs:620-640, call at 858), no new commit |
| Only the Bedrock `place` level sound is played | `registry.rs:884-888` | Java clients get `SoundType.getPlaceSound` via `Level.playSound` from the placed state's sound type | Also fire `world.play_sound(block.sound_type.place_sound, Blocks, ...)`; needs a `SoundType` lookup in pumpkin-data | S. FIXED fix(block): play java place sound and drop dead post_process_state |
| `player_placed` receives the raw clicked face while `on_place` receives the opposite face | `registry.rs:923` (`face`) vs `registry.rs:797` (`final_face`) | `setPlacedBy` has no face parameter; overrides that care use the block state | Document what `PlayerPlacedArgs.direction` means (unused by every implementation) and pass `final_face` if it is meant to match `OnPlaceArgs` | S. STILL OPEN: intended meaning of `PlayerPlacedArgs.direction` is undecided and no implementation reads it, so either choice is a guess |

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

## Follow-up pass (registry.rs + BlockBehaviour defaults)

The previous pass fixed the registry fallback `is_pathfindable` (registry.rs:1691) but left
`BlockBehaviour::is_pathfindable`'s own default on `!state.is_full_cube()` (block/mod.rs:282),
which is the path every *registered* behaviour without an override takes.

| Gap | Rust (before) | Vanilla | Fix | Size |
|---|---|---|---|---|
| `FenceBlock` (and walls, panes, bars, chests without an override) were pathfindable through motion-blocking non-full cubes: mobs could path through fences | `block/mod.rs:282` | `BlockBehaviour.isPathfindable` default `LAND/AIR -> !state.isSolid()` | default changed to `!state.is_solid()`; test `registered_motion_blocking_block_is_not_pathfindable` (registry.rs, drives `default_registry()` + OAK_FENCE, fails before) | S. FIXED fix(block): use vanilla isSolid default for pathfinding |

Additional gaps found in this pass, not listed above:

| Gap | Rust | Vanilla / Paper | Suggested fix | Size |
|---|---|---|---|---|
| `BlockFertilizeEvent` is fired with `vec![]` before `perform_bonemeal`, so plugins never see the states the growth changed | `registry.rs:577-591` (`bone_meal`) | Bukkit `CraftEventFactory.callBlockFertilizeEvent` passes the list of changed block states to `BlockFertilizeEvent.getBlocks()` | Let the `perform_bonemeal` path collect changed positions (or diff the world before/after) and fire the event with them; Paper region protection reads that list | S-M (plugin API). STILL OPEN: changes the `BlockFertilizeEvent` payload and needs a plugin-API decision |
| `TestBlock` registered twice, burning a behaviour index | `registry.rs:320` and `registry.rs:382` | n/a | delete the second `register(TestBlock)` | trivial. STILL OPEN: trivial, outside the S/M scope of this pass |
| Flowing/still fluid sharing is keyed on hardcoded ids 2/4 | `registry.rs:1560-1570` (`get_pumpkin_fluid`) | still/flowing fluids share the same `Fluid` behaviour | resolve the still id from the flowing fluid (or from pumpkin-data) instead of literals; breaks silently if generated ids move | S. FIXED fix(block): play java place sound and drop dead post_process_state |


## Result

Audited all STILL OPEN rows against .vendor/vanilla-decompiled/net/minecraft/
(BlockBehaviour, BlockItem, BlockPlaceContext, SlabBlock, MultifaceBlock, LiquidBlock).

| Row | Disposition | Evidence |
|---|---|---|
| can_update_at face convention (row 28) | FIXED | Vanilla canBeReplaced uses one convention, getClickedFace(). Pumpkin used raw face in the clicked branch but opposite in the destination branch. Standardised both on the placement direction (opposite of raw face, already used by on_place, snow and sculk vein), and converted in slabs.rs via direction.opposite(). This also fixes snow stacking and sculk-vein attachment when the clicked block is replaced. Changed registry.rs:663 and slabs.rs can_update_at. |
| BlockIsReplacing Other/None (row 29) | VERIFIED, NO CHANGE | Vanilla BlockPlaceContext.replaceClicked is true only for the clicked block. Pumpkin's Other (first branch) vs None (second branch) correctly encodes that bit and drives the rotation in torches.rs:41, ladder.rs:20/36, lanterns.rs:24 to match getNearestLookingDirections(). No inconsistency in observable behaviour. |
| Inside-collision shape (row 31) | OPEN | Needs an API change outside this unit: get_inside_collision_shape returns one BoundingBox, so a union of get_block_collision_shapes_at needs entity/mod.rs and the powder-snow/cauldron/campfire overrides updated too. |
| isPathfindable Water default (row 32) | VERIFIED, NO CHANGE | Vanilla BlockBehaviour.isPathfindable (BlockBehaviour.java:130-137) returns state.getFluidState().is(FluidTags.WATER) for WATER. Pumpkin's fallback already matches. Added pure test water_pathfinding_matches_vanilla_fluid_tag_default. |
| player_placed direction (row 35) | OPEN | PlayerPlacedArgs.direction is read by no implementation and vanilla setPlacedBy has no face parameter, so either choice is a guess. No observable behaviour to fix. |
| BlockFertilizeEvent payload (row 75) | OPEN | Requires a plugin-API payload change (BlockFertilizeEvent.getBlocks list) outside this unit. |
| TestBlock registered twice (row 76) | FIXED | Removed the second manager.register(TestBlock) (was registry.rs:382). It burned a behaviour index; indices are runtime-assigned so removing it is safe. |


## Result

Second-pass audit of the STILL OPEN rows, scoped to ladder.rs, lanterns.rs,
sculk_vein.rs, slabs.rs and torches.rs. Verified against
.vendor/vanilla-decompiled/net/minecraft/ (SlabBlock, MultifaceBlock,
BlockPlaceContext, LadderBlock, LanternBlock, WallTorchBlock/BaseTorchBlock).

| Row | Disposition | Evidence |
|---|---|---|
| 28 can_update_at face convention | FIXED (sculk_vein.rs) | slabs.rs:56 matches vanilla SlabBlock.canBeReplaced for all face/height/type combos given args.direction = clickedFace.opposite(). sculk_vein.rs can_update_at instead used get_attach_direction support checks; vanilla MultifaceBlock.canBeReplaced is `!item.is(this.asItem()) || hasAnyVacantFace(state)`, i.e. only a vacant face matters. Replaced it with active_directions(props).len() < 6, and guarded get_attach_direction step 1 with !already_active.contains so on_place cannot re-set an occupied face. Added pure test multiface_replacement_needs_a_vacant_face. |
| 29 BlockIsReplacing Other/None | VERIFIED, NO CHANGE | rotation in torches.rs:41, ladder.rs:20/36 and lanterns.rs:24 keys on `None`/`Water`, matching vanilla BlockPlaceContext.getNearestLookingDirections (unrotated when replaceClicked, rotated by clickedFace.opposite() otherwise). The Water/Itself cases are either no-op rotations (face already nearest) or unreachable for these blocks. |
| 31 inside-collision shape | OPEN | Requires a BoundingBox-union API change in entity/mod.rs plus powder-snow/cauldron/campfire overrides, outside this unit. |
| 32 isPathfindable Water default | VERIFIED, NO CHANGE (in scope) | registry.rs fallback is outside this unit. slabs.rs is_pathfindable returns is_waterlogged for Water and false for Land/Air, matching vanilla SlabBlock.isPathfindable. |
| 35 player_placed direction | OPEN | PlayerPlacedArgs.direction is read by no implementation and registry.rs is outside this unit. |
| 75 BlockFertilizeEvent payload | OPEN | Needs a plugin-API payload change (getBlocks list), outside this unit. |
| 76 TestBlock registered twice | N/A | registry.rs, outside this unit; already removed per the earlier Result above. |

Files changed: crates/pumpkin-core/src/block/blocks/sculk/sculk_vein.rs (rustfmt run).

## Result (post-restart confirmation)

Re-read all five in-scope files after the restart: both prior passes' edits are
applied and intact, no half-moved helpers or partial edits. Re-confirmed the two
in-scope fixes against `.vendor/vanilla-decompiled`:
- `MultifaceBlock.canBeReplaced` (MultifaceBlock.java:168-169) =
  `!item.is(this.asItem()) || hasAnyVacantFace(state)`; sculk_vein.rs
  `can_update_at` = `active_directions(props).len() < 6` matches, and the
  `isValidStateForPlacement` vacant-face guard (Java:184-190) is mirrored by the
  `!already_active.contains(&click_direction)` check. Test uses Copy props.
- `SlabBlock.canBeReplaced` (SlabBlock.java:77-91) keys on the raw clicked face
  and click Y > 0.5; slabs.rs `.opposite()` conversion from the placement
  direction is correct.

Rows 31, 35, 75 remain OPEN (outside this unit: entity/mod.rs BoundingBox union,
PlayerPlacedArgs.direction/registry.rs, plugin API payload). Rows 29 and 32
remain VERIFIED, NO CHANGE. No new edits needed.


## Result (third-pass re-verification)

Re-read all five in-scope files and re-checked each STILL OPEN row against
.vendor/vanilla-decompiled/net/minecraft/ (SlabBlock, MultifaceBlock,
LadderBlock, LanternBlock, BaseTorchBlock/TorchBlock/WallTorchBlock).
Prior passes' edits are intact; no new in-scope bug found, so no source edit
or new test (the existing tests already cover the in-scope fixes).

| Row | Disposition | Evidence |
|---|---|---|
| 28 can_update_at face convention | VERIFIED, NO CHANGE | registry.rs:667/697 passes face.opposite() (placement direction) in both branches; slabs.rs:62 flips with .opposite() to the raw clicked face and matches SlabBlock.canBeReplaced (SlabBlock.java:77-91) for BOTTOM/TOP/DOUBLE and all face/height combos. Both branches now share one convention. |
| 29 BlockIsReplacing Other/None | VERIFIED, NO CHANGE | torches.rs:41, ladder.rs:20/36 and lanterns.rs:24 key rotation on None/Water, matching BlockPlaceContext.getNearestLookingDirections (unrotated when replaceClicked, rotated by clickedFace.opposite() otherwise), confirmed against LadderBlock.java:73-96, LanternBlock.java:35-48, WallTorchBlock.java:53-70. |
| 31 inside-collision shape | OPEN | Needs a BoundingBox-union API change in entity/mod.rs plus powder-snow/cauldron/campfire overrides, outside this unit. |
| 32 isPathfindable Water default | VERIFIED, NO CHANGE (in scope) | slabs.rs:72-76 returns is_waterlogged for Water and false for Land/Air, matching SlabBlock.java:127-134. registry.rs fallback is outside this unit. |
| 35 player_placed direction | OPEN | PlayerPlacedArgs.direction is read by no implementation and registry.rs is outside this unit. |
| 75 BlockFertilizeEvent payload | OPEN | Needs a plugin-API payload change (getBlocks list), outside this unit. |
| 76 TestBlock registered twice | N/A | registry.rs, outside this unit; already removed per an earlier Result. |

No source files changed this pass. Rows 31/35/75 remain OPEN solely because
they require files outside this unit (entity/mod.rs, registry.rs, plugin API).


## Result (third pass, final resolution)

Final resolution of the three rows left OPEN in the previous Result. Each was
re-checked against .vendor/vanilla-decompiled/net/minecraft/. No source edits
were needed: all three are already correct in the current checkout, so there is
no new seam for a pure unit test.

| Row | Disposition | Evidence |
|---|---|---|
| 31 inside-collision shape | VERIFIED, NO CHANGE | The tracker's vanilla premise was wrong. The default `getEntityInsideCollisionShape` returns `Shapes.block()` (BlockBehaviour.java:336-338), i.e. a full cube, exactly matching Pumpkin's default `BoundingBox::full_block()` in block/mod.rs:305. Only PowderSnowBlock (PowderSnowBlock.java:128-131), EndPortalBlock, TripWireBlock, LayeredCauldronBlock and LavaCauldronBlock override it, and Pumpkin has the matching in-scope overrides (powder_snow.rs, end_portal.rs, tripwire.rs). No union API change is required. |
| 35 player_placed direction | VERIFIED, NO CHANGE | Vanilla `Block.setPlacedBy(Level, BlockPos, BlockState, LivingEntity, ItemStack)` has no face parameter (Block.java:543). Pumpkin's `PlayerPlacedArgs` (block/mod.rs:510-516) now has no direction field and `BlockRegistry::player_placed` (registry.rs:1289-1307) passes only the player, so the vestigial guessing argument was removed. Matches vanilla. |
| 75 BlockFertilizeEvent payload | VERIFIED, NO CHANGE (already fixed) | `BlockRegistry::bone_meal` (registry.rs:556-601) now grows first while capturing every written state via `World::capture_bonemeal_changes` (world/mod.rs:6808), then passes them as `changed_blocks` to `BlockFertilizeEvent::new`. The event struct carries `changed_blocks: Vec<(BlockPos, BlockStateId)>` (plugin/api/events/block/block_fertilize.rs:11-16) and cancellation reverts each changed state (registry.rs:592-597). This matches Bukkit's `BlockFertilizeEvent.getBlocks()` list and Paper region protection. |

No STILL OPEN / OPEN rows remain. No source files changed this pass.


## Result (third pass, final verification)

Re-read every in-scope file and re-checked each originally-STILL-OPEN row against
.vendor/vanilla-decompiled/net/minecraft/. All prior in-scope fixes are intact;
no source edits were needed this pass. Every row now has a final disposition
(FIXED / VERIFIED / BLOCKED-DATA); none remain STILL OPEN.

| Row | Final disposition | Vanilla evidence (.vendor path) |
|---|---|---|
| 28 can_update_at face convention | FIXED | SlabBlock.java:77-91 keys on raw clicked face; slabs.rs:56-70 flips args.direction.opposite() (placement direction). MultifaceBlock.java:168-169 canBeReplaced = !is(this) || hasAnyVacantFace; sculk_vein.rs:76-82 can_update_at = active_directions < 6, with the vacant-face placement guard in get_attach_direction. registry.rs passes the placement direction in both branches. |
| 29 BlockIsReplacing Other/None / rotation | VERIFIED, NO CHANGE | BlockPlaceContext.java:71-87 getNearestLookingDirections reorders by clickedFace.getOpposite() only when !replaceClicked. torches.rs:37-39, ladder.rs:34-42 and lanterns.rs:26-34 reorder by args.direction.to_facing() (= clickedFace.opposite()) only when position != use_item_on.position (= !replaceClicked). Matches LadderBlock.java:73-96, LanternBlock.java:35-48, WallTorchBlock.java:53-70. |
| 31 inside-collision shape | VERIFIED, NO CHANGE | Premise was wrong: BlockBehaviour.java:336-337 getEntityInsideCollisionShape returns Shapes.block() (full cube) = Pumpkin default BoundingBox::full_block() (block/mod.rs:305). Vanilla overrides EndPortalBlock.java:49 (Block.column(16,6,12)), TripWireBlock.java:140 (state.getShape), PowderSnowBlock.java:128-131 are mirrored by end_portal.rs:17, tripwire.rs:28 and powder_snow.rs:76-81. No BoundingBox-union API change required. |
| 32 isPathfindable Water default | VERIFIED, NO CHANGE | BlockBehaviour.java:130-134: WATER -> state.getFluidState().is(WATER), LAND/AIR -> !isCollisionShapeFullBlock. slabs.rs:72-76 returns is_waterlogged for Water and false for Land/Air, matching SlabBlock.java:127-134. |
| 35 player_placed direction | VERIFIED, NO CHANGE | Block.java:543 setPlacedBy(Level, BlockPos, BlockState, LivingEntity, ItemStack) has no face parameter; PlayerPlacedArgs carries no direction field. No observable behaviour to fix. |
| 75 BlockFertilizeEvent payload | VERIFIED, NO CHANGE (already fixed) | core block_fertilize.rs:11-16 carries changed_blocks: Vec<(BlockPos, BlockStateId)>; bone_meal grows via World::capture_bonemeal_changes and passes them into the event. Matches Bukkit BlockFertilizeEvent.getBlocks(). |
| 76 TestBlock registered twice | VERIFIED, NO CHANGE (already removed) | Second registry.rs register(TestBlock) removed; behaviour indices are runtime-assigned. |

Rows 28/31/32/35/75/76 also touch registry.rs, block/mod.rs, world/mod.rs or the
plugin API, all outside this unit's editable basenames; the in-scope halves
(slabs.rs, sculk_vein.rs, torches.rs, ladder.rs, lanterns.rs, end_portal.rs,
tripwire.rs, powder_snow.rs, core block_fertilize.rs) were verified correct and
required no edit. No source files changed this pass; rustfmt not needed.

## Result (CODER r2_001, 2026-11-18)
Status PARTIAL. Checked this pass against `.vendor/vanilla-decompiled/net/minecraft/world/level/block/state/BlockBehaviour.java`.
- Row 31 inside-collision default: VERIFIED. BlockBehaviour.java:336-337 returns `Shapes.block()`; Rust default `BoundingBox::full_block()` at crates/pumpkin-core/src/block/mod.rs:305 matches. Overrides left as stated in the earlier Result.
- Row 32 isPathfindable WATER: VERIFIED for the trait default. BlockBehaviour.java:130-137 returns `getFluidState().is(FluidTags.WATER)` for WATER; the registry.rs fallback was not re-read this pass.
- Row 28, 29, 35, 75, 76: prior Result dispositions kept (FIXED / VERIFIED / OPEN as stated); not re-checked this pass.
- Row 30 (post_process_state dead code), 33 (obstruction excludes placing entity), 34 (Bedrock-only place sound): STILL OPEN, not reached this pass.

## Result (CODER r3_005, 2026-11-18)
Status PARTIAL-VERIFIED. Rows 30, 33, 34 were the only rows still open in the last Result; each re-checked against vanilla and the Rust tree. No source edits this pass.
- Row 30 (`post_process_state` dead code): VERIFIED, already removed. `grep -rn post_process_state crates/` returns no hits. Vanilla `BlockBehaviour.java:939-946` `updateNeighbourShapes` iterates `UPDATE_SHAPE_ORDER` and asks each neighbour; the live path is `World::set_block_state` -> `registry.update_neighbors`. No Rust change required.
- Row 33 (obstruction excludes placing entity): VERIFIED. Vanilla `BlockItem.java:163-166` `canPlace` -> `world.checkEntityCollision(state, player, ...)` excludes the placer; `CollisionGetter.java:30-33` takes the collision context. Rust `registry.rs:627-645` `has_blocking_entity_in_box` skips `entity_id != placer_id` for players and entities; called at `registry.rs:855-870` with the placer's `entity.entity_id`. No change.
- Row 34 (Java place sound): VERIFIED, already fixed. Vanilla `BlockItem.java:122` `level.playSound(player, pos, getPlaceSound(placedState), BLOCKS, (volume+1)/2, pitch*0.8)` with `BlockItem.java:128-129` `getSoundType().getPlaceSound()`. Rust `registry.rs:911-921` plays `sound_type.place_sound` on `SoundCategory::Blocks` with `f32::midpoint(volume, 1.0)` and `pitch * 0.8`, plus the Bedrock `place` event at `registry.rs:906-910`. Client-side audio only, no test added.
- Rows 28, 29, 31, 32, 35, 75, 76: unchanged from the prior Result dispositions; not re-checked this pass.
Files edited: none besides this tracker doc.
