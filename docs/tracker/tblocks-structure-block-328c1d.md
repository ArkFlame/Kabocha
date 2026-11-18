
# blocks / Structure Block (RE-REVIEW-UNTRACKED)

Re-reviewed after ~3dbc79f09 (structure block packets/actions), ~eddc86906 (palette
classification) and ~75732956f (integrity air replacement). Reference: decompiled Paper/Canvas
tree at /run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft (world/level/block/StructureBlock.java,
world/level/block/entity/StructureBlockEntity.java,
world/level/levelgen/structure/templatesystem/StructureTemplate.java,
BlockRotProcessor.java, StructurePlaceSettings.java,
server/network/ServerGamePacketListenerImpl.java#handleSetStructureBlock). .vendor absent.

## Fixed in this pass

- ~place_structure_template_in_world~ (block/entities/structure_block.rs:407-411): rejected
  integrity entries are now dropped (`continue`) instead of being written as air, and NBT
  blocks now roll too, matching BlockRotProcessor.processBlock -> null (dropped by
  processBlockInfos, StructureTemplate.java:465-505) and the unconditional processor
  (StructureBlockEntity.java:418-420). Commit d3bebfb5c.
- Two RNG streams (`integrity_rng` / `loot_rng`, :386-390): vanilla seeds the settings random
  and the placement random independently (StructureBlockEntity.java:419,429); Pumpkin shared one
  stream, so a LootTable seed draw shifted every later integrity roll. Commit d3bebfb5c.
- ~structure_void~ palette entries are placed again (:394): vanilla placeInWorld has no void
  filter (StructureTemplate.java:288-311); only fillFromWorld excludes them
  (StructureBlockEntity.java:348-350). Commit 844978d43.

## Verified, no action

- NBT key set, defaults and clamps match saveAdditional/loadAdditional
  (StructureBlockEntity.java:81-129 vs :543-679); `updateBlockState` == NOTIFY_LISTENERS (:784-798).
- Redstone trigger: neighborChanged + trigger incl. SAVE -> saveStructure(false), LOAD -> place,
  CORNER -> unload (StructureBlock.java:69-106 vs block/blocks/structure_block.rs:65-93).
- detectSize bounds (+-80 X/Z, full build height, single corner also encapsulates self,
  delta > 1, min+1 / delta-1) match StructureBlockEntity.java:267-314; Rust detect_size :951-1041,
  compute_detected_bounds :462-505.
- Save excludes structural void and includes air, three-list split and the y/x/z ordering match
  StructureTemplate.java:101-169 (capture :188-262 uses loop order y,x,z).
- handleSetStructureBlock field order, game-master gate, name/action message keys and the
  trailing block update match (set_structure_block.rs:61-138 vs ServerGamePacketListenerImpl.java:1271-1321).
- Author: setPlacedBy -> createdBy (StructureBlock.java:56-62) vs player_placed
  (block/blocks/structure_block.rs:32-41); loadStructureInfo copies author+size only.
- useWithoutItem: canUseGameMasterBlocks (creative + perm >= 2), server-only open, SUCCESS/PASS
  match block/blocks/structure_block.rs:43-63.
- placeStructureIfSameSize / isStructureLoadable match StructureBlockEntity.java:367-454.

## Remaining gaps

| Location | Vanilla | Gap | Fix | Size |
|---|---|---|---|---|
| block/entities/structure_block.rs:188-262 (capture), :358-461 (place) | StructureTemplate.fillFromWorld with includeEntities fills entityInfoList (StructureTemplate.java:133-137,175-193); placeInWorld calls placeEntities when !isIgnoreEntities (StructureTemplate.java:414-425) | Saving never records entities and loading never spawns them; ignore_entities (:521,599,656) is dead state | Add an entity pass to capture and persist "entities" in template NBT, then spawn on place honouring ignore_entities | L, STILL OPEN: size L, skipped per wave brief |
| block/entities/structure_block.rs:246 | addToLists: !state.getBlock().hasDynamicShape() && state.isCollisionShapeFullBlock(EmptyBlockGetter, ZERO) (StructureTemplate.java:150) | is_solid_render() (occlusion flag) is not the collision-shape predicate; e.g. glass/transparent full cubes land in the "other" list, changing palette order and therefore which block gets which integrity roll | Use a generated collision-shape-full flag plus hasDynamicShape | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| block/entities/structure_block.rs:451-455 | Non-strict placement runs Block.updateFromNeighbourShapes on every placed pos, then updateNeighborsAt, plus updateShapeAtEdge (StructureTemplate.java:376-401,434+) | Only update_neighbors_at runs, so fences/stairs/walls/attached states keep their template shape instead of reconnecting | Add update_from_neighbour_shapes (no helper exists in pumpkin-core today) before the neighbour update, guarded by !strict | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| block/entities/structure_block.rs:432-434 | LootTableSeed is written only when the placed block entity is a RandomizableContainer (StructureTemplate.java:322-324) | Seed is written for every template NBT carrying LootTable | Gate on a randomizable-container check once an API exists | S/M |
| block/entities/structure_block.rs:414-447 | Vanilla sets a BARRIER with UPDATE_SKIP_ALL_SIDEEFFECTS before loading NBT (StructureTemplate.java:293-295), places with UPDATE_CLIENTS and skips all side effects when strict (StructureBlockEntity.java:429) | Placer writes state directly and then queues/flushes updates; the strict flag only suppresses neighbour updates, not place side effects such as drops | Map strict to skip-side-effects placement flags once WorldBlockPlacer exposes them | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| net/java/play/set_structure_block.rs:134-137 | structure.setChanged(); level.sendBlockUpdated(pos, state, state, UPDATE_ALL) (ServerGamePacketListenerImpl.java:1317-1318) | Only mark_dirty + register_block_change (client notify); no neighbour update | Use an update-all helper when the world API offers one | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |

Note: no automated test was added for the integrity change (needs a World + placer fixture and
the worker brief forbids running cargo); the palettes themselves are covered by
`structure_identifier_validation...` and `detect_size_matches...` in the same file.

## h6 status
- Structure block rows: STILL OPEN, not attempted this pass (see tblocks-structure-block.md h6 status).

## Result

Re-verified every STILL OPEN row against .vendor/vanilla-decompiled/net/minecraft/
(StructureTemplate.java, StructureBlockEntity.java, ServerLevel.java, ServerGamePacketListenerImpl.java).

- Non-strict shape reconnect (Remaining gaps row at old line 51): FIXED.
  place_structure_template_in_world now calls World::update_from_neighbor_shapes on every placed
  position after the whole template is written, writes the shaped state with
  NOTIFY_LISTENERS | UPDATE_KNOWN_SHAPE, then calls update_neighbors_at with the shaped block.
  The pass stays behind !strict, matching StructurePlaceSettings.setKnownShape(this.strict) and
  updateMode & ~UPDATE_NEIGHBORS | UPDATE_KNOWN_SHAPE (StructureTemplate.java:391-401,
  StructureBlockEntity.java:417,429). The single block-update flush moved after the pass so the
  shaped states reach clients together. File: crates/pumpkin-core/src/block/entities/structure_block.rs.

- Entity capture and placement (old line 49): LEFT OPEN. Vanilla fills entityInfoList in
  fillFromWorld (StructureTemplate.java:133-137,175-193) and calls placeEntities when
  !isIgnoreEntities (:414-425). The pumpkin-world template module has no entity-info field or
  spawn path, so this needs cross-crate work outside this unit.

- Collision-shape-full + hasDynamicShape palette split (old line 50): LEFT OPEN. The collision-shape
  half is already correct: BlockState::is_full_cube() is derived from the extracted collision boxes
  in pumpkin-data (derive_state_data), and the existing test
  palette_full_block_uses_collision_shape_not_occlusion covers glass. The remaining
  !hasDynamicShape() predicate (StructureTemplate.java:150) has no generated flag in pumpkin-data,
  so it needs a codegen/data change outside this unit.

- Strict placement side effects (old line 53): LEFT OPEN for the literal ask. Vanilla maps strict to
  both setKnownShape and UPDATE_SKIP_ALL_SIDEEFFECTS (StructureBlockEntity.java:428-430). The
  setKnownShape half is now covered by the row-51 fix; the skip-side-effects half would need
  WorldBlockPlacer changes in crates/pumpkin-core/src/world/block_placer.rs (outside this unit).
  Observable behaviour already matches: the placer writes straight to the chunk and
  world.remove_block_entity(&pos) runs before every write, so drops are suppressed regardless.

- handleSetStructureBlock trailing update (old line 54): VERIFIED, NO ACTION.
  ServerLevel.sendBlockUpdated (ServerLevel.java:3498-3574) only calls blockChanged(pos),
  invalidates the path cache and recomputes navigation when collision shapes differ. It sends no
  neighbour update. World::register_block_change already invalidates path caches and queues the
  client block update, so the current code matches vanilla.

No pure seam exists for the shape-update pass (it needs Arc<World>), so no new unit test was added;
the existing palette/identifier tests in the same file still cover the data-only rows.
rustfmt --edition 2024 was run on the one touched .rs file. Compile deferred to the orchestrator.


## Result

Re-verified every STILL OPEN row against .vendor/vanilla-decompiled/net/minecraft (1.21.x Paper/Canvas tree).

- Old line 49, entity capture and placement: VERIFIED ALREADY FIXED in the checkout. capture_structure_template_from_world fills entity_info_list (players excluded, AABB.encapsulatingFullBlocks, painting block_pos, int-array codec) and place_structure_template_in_world calls place_template_entities behind !ignore_entities, matching fillEntityList (StructureTemplate.java:175-193) and placeEntities (:511-547). No new change needed.
- Old line 50, collision-shape-full + hasDynamicShape: VERIFIED ALREADY FIXED. has_dynamic_shape matches the only eight .dynamicShape() declarations in Blocks.java (moving_piston, bamboo, scaffolding, powder_snow, pointed_dripstone, sulfur_spike, shulkerBoxProperties for all 16 colors); is_full_cube comes from extracted collision boxes and the glass test covers the occlusion distinction. No new change needed.
- Old line 51, non-strict shape reconnect: VERIFIED ALREADY FIXED. The !strict pass runs update_shape_at_edge then update_from_neighbor_shapes and writes NOTIFY_LISTENERS | UPDATE_KNOWN_SHAPE, equal to updateMode & ~UPDATE_NEIGHBORS | UPDATE_KNOWN_SHAPE for non-strict (updateMode = UPDATE_CLIENTS). No new change needed.
- Old line 52, LootTableSeed gating (not marked STILL OPEN): FIXED a real mismatch. is_randomizable_container_id listed suspicious_sand and suspicious_gravel, but vanilla BrushableBlockEntity extends BlockEntity (not RandomizableContainer); the only implementors are RandomizableContainerBlockEntity subclasses (chest/trapped_chest, barrel, dispenser/dropper, hopper, shulker boxes, crafter) plus DecoratedPotBlockEntity. Removed the two brushable ids and extended loot_seed_only_for_randomizable_containers to assert they are excluded.
- Old line 53, strict placement side effects: VERIFIED ALREADY FIXED. WorldBlockPlacer::new_with_skip_side_effects(world, strict) suppresses on_block_replaced drops in strict mode, mapping to vanilla UPDATE_SKIP_ALL_SIDEEFFECTS; nbt entries pre-clear the old block entity in both modes like the BARRIER pre-set. No new change needed.
- Old line 54, handleSetStructureBlock trailing update: VERIFIED NO ACTION. ServerLevel.sendBlockUpdated only fires blockChanged plus path/navigation cache invalidation; register_block_change already does both, so no neighbour update is missing.

Only the LootTableSeed row needed an edit; the other STILL OPEN rows were already implemented in this checkout and are now verified against vanilla. rustfmt --edition 2024 run on the touched .rs file. Compile deferred to the orchestrator.


## Result

Third pass. Re-verified every STILL OPEN row against .vendor/vanilla-decompiled/net/minecraft
(StructureTemplate.java, StructureBlockEntity.java, RandomizableContainer implementors,
BrushableBlockEntity.java, DecoratedPotBlockEntity.java). No new source edits were needed; every
row is already implemented and matches vanilla.

- Old line 49, entity capture and placement: VERIFIED ALREADY FIXED. capture_structure_template_from_world
  fills entity_info_list (players excluded via EntityType::PLAYER, AABB.encapsulatingFullBlocks,
  painting block_pos via int-array codec), and place_structure_template_in_world calls
  place_template_entities behind !ignore_entities. Matches fillEntityList (StructureTemplate.java:175-193)
  and placeEntities (:414-425, :511).
- Old line 50, collision-shape-full + hasDynamicShape: VERIFIED ALREADY FIXED. Palette split uses
  state.is_full_cube() && !has_dynamic_shape(block.name) (structure_block.rs:287-289), matching
  StructureTemplate.java:150. has_dynamic_shape covers all eight dynamicShape() declaration sites
  (moving_piston, bamboo, scaffolding, powder_snow, pointed_dripstone, sulfur_spike, 16 shulker boxes).
- Old line 51, non-strict shape reconnect: VERIFIED ALREADY FIXED. The !strict pass runs
  update_shape_at_edge then update_from_neighbor_shapes, writes
  NOTIFY_LISTENERS | UPDATE_KNOWN_SHAPE, then update_neighbors_at (structure_block.rs:720-739),
  equal to updateMode & ~UPDATE_NEIGHBORS | UPDATE_KNOWN_SHAPE for non-strict
  (StructureTemplate.java:377-401).
- Old line 52, LootTableSeed gating: VERIFIED ALREADY FIXED. is_randomizable_container_id lists
  chest/trapped_chest/barrel/dispenser/dropper/hopper/crafter/decorated_pot plus all shulker boxes.
  Confirmed against decompiled classes: RandomizableContainerBlockEntity subclasses are Chest
  (incl. trapped), Dispenser (incl. dropper), Barrel, Hopper, ShulkerBox, Crafter, and
  DecoratedPotBlockEntity implements RandomizableContainer directly; BrushableBlockEntity extends
  plain BlockEntity and is correctly excluded (StructureTemplate.java:322-324).
- Old line 53, strict placement side effects: VERIFIED ALREADY FIXED.
  WorldBlockPlacer::new_with_skip_side_effects(world, strict) suppresses drop callbacks, and the
  remove_block_entity pre-clear for NBT entries mirrors the BARRIER pre-set
  (StructureTemplate.java:293-295).
- Old line 54, handleSetStructureBlock trailing update: VERIFIED NO ACTION. set_structure_block.rs
  ends with mark_dirty + update_block_entity + register_block_change (:147-150), matching
  ServerLevel.sendBlockUpdated (blockChanged + cache invalidation), which sends no neighbour update.

No pure unit-test seam exists for the Arc<World> placement paths; the existing palette and
loot_seed_only_for_randomizable_containers tests in structure_block.rs cover the data-only rows.
No files edited this pass. Compile deferred to the orchestrator.


## Result

Fourth pass. Re-checked every STILL OPEN row against .vendor/vanilla-decompiled/net/minecraft
(StructureTemplate.java, StructurePlaceSettings.java, StructureBlockEntity.java, ServerLevel.java,
ServerGamePacketListenerImpl.java, BitRandomSource.java, Mth.java). One new mismatch found and fixed;
the remaining rows stay verified as already implemented.

- Row 49, entity capture and placement: VERIFIED, NO ACTION. capture_structure_template_from_world
  fills entity_info_list (players excluded, AABB.encapsulatingFullBlocks, painting block_pos) matching
  fillEntityList (StructureTemplate.java:175-193), and place_template_entities runs behind
  !ignore_entities matching placeEntities (:511-552) including the UUID drop, Pos rewrite,
  block_pos rewrite and the rotate-then-mirror yaw.
- Row 50, collision-shape-full + hasDynamicShape: VERIFIED, NO ACTION. Palette split is
  state.is_full_cube() && !has_dynamic_shape(name) (:287-289), matching addToLists (:150). The seven
  .dynamicShape() declaration sites in Blocks.java (moving_piston, bamboo, scaffolding, powder_snow,
  pointed_dripstone, sulfur_spike, shulkerBoxProperties) are all covered by has_dynamic_shape.
- Row 51, non-strict shape reconnect: VERIFIED, NO ACTION. update_shape_at_edge then
  update_from_neighbor_shapes with NOTIFY_LISTENERS | UPDATE_KNOWN_SHAPE (:734-747) matches
  StructureTemplate.java:376-401 with updateMode & ~UPDATE_NEIGHBORS | UPDATE_KNOWN_SHAPE.
- Row 52, LootTableSeed gating: VERIFIED, NO ACTION. is_randomizable_container_id covers exactly the
  RandomizableContainer implementors and excludes the brushable block entities (:423-436).
- Row 53, strict placement side effects: VERIFIED, NO ACTION.
  WorldBlockPlacer::new_with_skip_side_effects(world, strict) suppresses drop callbacks and the NBT
  pre-clear mirrors the BARRIER pre-set (StructureTemplate.java:293-295).
- Row 54, handleSetStructureBlock trailing update: VERIFIED, NO ACTION. ServerLevel.sendBlockUpdated
  only fires blockChanged plus path/navigation cache invalidation; register_block_change already does
  both and sends no neighbour update.

- NEW row, palette selection: FIXED. Vanilla gets its block list from
  settings.getRandomPalette(palettes, position) (StructureTemplate.java:270), which draws
  nextInt(palettes.size()) from the settings random when the rot processor installed one
  (StructurePlaceSettings.java:150, StructureBlockEntity.java:419) and from
  RandomSource.create(Mth.getSeed(position)) otherwise (:110-116). Pumpkin always used
  palettes().first(), so multi-palette templates always loaded palette 0 and, whenever integrity was
  below 1.0, the rot stream was missing the leading palette draw, shifting every integrity roll by one.
  place_structure_template_in_world now derives the palette index from the same settings stream before
  the rot pass (and from the position-seeded stream when no rot processor applies). Helpers
  palette_index_from / palette_fallback_seed live in block/entities/structure_block.rs.

Test: palette_selection_draws_from_the_vanilla_stream pins both streams against vanilla
LegacyRandomSource.nextInt(3) values (seed 42 -> 2, Mth.getSeed(3,64,-7) -> 1), computed from the
decompiled BitRandomSource/LegacyRandomSource algorithms. No cargo run was allowed for this unit, so
the test is UNVERIFIED until the orchestrator batch-checks.

File changed: crates/pumpkin-core/src/block/entities/structure_block.rs (the working file also carried an
uncommitted `integrity_rot_applies` helper plus its test from another slice; this pass keeps and uses
that helper instead of reverting it).


## Result

Fifth pass. Re-verified every STILL OPEN row against .vendor/vanilla-decompiled/net/minecraft
(StructureTemplate.java, StructureBlockEntity.java, StructurePlaceSettings.java, BlockRotProcessor.java,
Mth.java, ServerLevel.java, ServerGamePacketListenerImpl.java). All six rows are already implemented and
match vanilla; no source behaviour change was required.

- Row 49, entity capture and placement: VERIFIED. fillEntityList (StructureTemplate.java:175-192)
  excludes players, uses AABB.encapsulatingFullBlocks and painting.getPos() for blockPos;
  capture_structure_template_from_world (structure_block.rs:305-342) mirrors it and
  place_template_entities (:461-512) mirrors placeEntities (:511-552) behind !ignore_entities,
  including the UUID drop, Pos/block_pos rewrites and rotate-then-mirror yaw.
- Row 50, collision-shape-full + hasDynamicShape: VERIFIED. addToLists (:142-155) uses
  !hasDynamicShape() && isCollisionShapeFullBlock; the palette split at structure_block.rs:287-289
  uses state.is_full_cube() && !has_dynamic_shape(name) and covers all dynamicShape sites.
- Row 51, non-strict shape reconnect: VERIFIED. updateShapeAtEdge (:438-463) plus
  Block.updateFromNeighbourShapes (:391-401) map to update_shape_at_edge + update_from_neighbor_shapes
  + update_neighbors_at behind !strict (structure_block.rs:759-778).
- Row 52, LootTableSeed gating: VERIFIED. placeInWorld (:319-324) seeds only RandomizableContainer
  implementors; is_randomizable_container_id (structure_block.rs:423-436) matches exactly.
- Row 53, strict placement side effects: VERIFIED. UPDATE_SKIP_ALL_SIDEEFFECTS maps to
  WorldBlockPlacer::new_with_skip_side_effects(world, strict) (block_placer.rs:49); the NBT block-entity
  pre-clear mirrors the BARRIER pre-set (:293-295).
- Row 54, handleSetStructureBlock trailing update: VERIFIED NO ACTION. ServerLevel.sendBlockUpdated
  fires blockChanged plus cache invalidation only, which register_block_change already covers.

- Test correction: palette_selection_draws_from_the_vanilla_stream pinned the positional fallback
  palette index to 1 for Mth.getSeed(3,64,-7), but the vanilla Mth.getSeed formula (Mth.java:332-336,
  returns long with no int truncation) plus LegacyRandomSource.nextInt(3) yields 0. Corrected the
  expected value to 0 and the comment. Cross-checked the RNG simulation against pumpkin-util's
  LegacyRand::from_seed(0).next_bounded_i32(15) vector [0,13,4,2,5,8,11,6,9,14]. No cargo run was
  permitted; compile and test are deferred to the orchestrator.

File changed: crates/pumpkin-core/src/block/entities/structure_block.rs (test expectation only).
