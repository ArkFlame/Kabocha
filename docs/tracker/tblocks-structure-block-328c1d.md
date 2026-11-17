
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
| block/entities/structure_block.rs:188-262 (capture), :358-461 (place) | StructureTemplate.fillFromWorld with includeEntities fills entityInfoList (StructureTemplate.java:133-137,175-193); placeInWorld calls placeEntities when !isIgnoreEntities (StructureTemplate.java:414-425) | Saving never records entities and loading never spawns them; ignore_entities (:521,599,656) is dead state | Add an entity pass to capture and persist "entities" in template NBT, then spawn on place honouring ignore_entities | L |
| block/entities/structure_block.rs:246 | addToLists: !state.getBlock().hasDynamicShape() && state.isCollisionShapeFullBlock(EmptyBlockGetter, ZERO) (StructureTemplate.java:150) | is_solid_render() (occlusion flag) is not the collision-shape predicate; e.g. glass/transparent full cubes land in the "other" list, changing palette order and therefore which block gets which integrity roll | Use a generated collision-shape-full flag plus hasDynamicShape | M |
| block/entities/structure_block.rs:451-455 | Non-strict placement runs Block.updateFromNeighbourShapes on every placed pos, then updateNeighborsAt, plus updateShapeAtEdge (StructureTemplate.java:376-401,434+) | Only update_neighbors_at runs, so fences/stairs/walls/attached states keep their template shape instead of reconnecting | Add update_from_neighbour_shapes (no helper exists in pumpkin-core today) before the neighbour update, guarded by !strict | M |
| block/entities/structure_block.rs:432-434 | LootTableSeed is written only when the placed block entity is a RandomizableContainer (StructureTemplate.java:322-324) | Seed is written for every template NBT carrying LootTable | Gate on a randomizable-container check once an API exists | S/M |
| block/entities/structure_block.rs:414-447 | Vanilla sets a BARRIER with UPDATE_SKIP_ALL_SIDEEFFECTS before loading NBT (StructureTemplate.java:293-295), places with UPDATE_CLIENTS and skips all side effects when strict (StructureBlockEntity.java:429) | Placer writes state directly and then queues/flushes updates; the strict flag only suppresses neighbour updates, not place side effects such as drops | Map strict to skip-side-effects placement flags once WorldBlockPlacer exposes them | M |
| net/java/play/set_structure_block.rs:134-137 | structure.setChanged(); level.sendBlockUpdated(pos, state, state, UPDATE_ALL) (ServerGamePacketListenerImpl.java:1317-1318) | Only mark_dirty + register_block_change (client notify); no neighbour update | Use an update-all helper when the world API offers one | S |

Note: no automated test was added for the integrity change (needs a World + placer fixture and
the worker brief forbids running cargo); the palettes themselves are covered by
`structure_identifier_validation...` and `detect_size_matches...` in the same file.

## h6 status
- Structure block rows: STILL OPEN, not attempted this pass (see tblocks-structure-block.md h6 status).

## h6 pass 2 status
- LootTableSeed row (block/entities/structure_block.rs:432-434): FIXED d5cd38b63 (seed only for randomizable container ids; test loot_seed_only_for_randomizable_containers).
- Other rows (entity capture/place L, addToLists M, neighbour shape update M, strict side effects M): STILL OPEN, not attempted this pass.
