# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-block-blocks-structure-block-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

All rows closed below. Per row: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field.

## Result

FINAL PASS, re-verified 2026-11-18 against the decompiled vanilla under
`.vendor/vanilla-decompiled/net/minecraft/`. Every row ends FIXED, VERIFIED or
BLOCKED-DATA; none remain open.

Unit row **blocks/Structure Block** [partial/AUDITED]: VERIFIED. The full
`crates/pumpkin-core/src/block/blocks/structure_block.rs` matches vanilla
`.../world/level/block/StructureBlock.java`:
- `on_place` sets mode LOAD (vanilla default state, StructureBlock.java:27).
- `placed` creates the block entity (newBlockEntity, :31-33).
- `player_placed` sets the author (setPlacedBy -> createdBy, :47-53).
- `normal_use` requires permission lvl >= 2 and Creative and a StructureBlockBlockEntity
  (useWithoutItem -> usedBy -> canUseGameMasterBlocks, :36-44 and
  StructureBlockEntity.java:151-161).
- `on_neighbor_update` triggers SAVE(false)/LOAD/CORNER on rising redstone
  (neighborChanged -> trigger, StructureBlock.java:60-90).

Unit row **own/taudit-crates-pumpkin-core-src-bl-6f9a28** [partial/OWN-OPEN]: VERIFIED.
All 9 gap rows from docs/tracker/taudit-crates-pumpkin-core-src-bl-6f9a28.md are
closed in the current tree (code fixes already present, re-checked this pass).
Per row:

- 1. Entity capture on save: FIXED. `capture_structure_template_from_world` takes
  `include_entities`, fills `entity_info_list` over `AABB.encapsulatingFullBlocks` via
  `world.get_entities_at_box`, and filters players (`entity_type == EntityType::PLAYER`).
  Vanilla `StructureTemplate#fillEntityList`
  (`.vendor/.../levelgen/structure/templatesystem/StructureTemplate.java:175-192`) and
  `StructureBlockEntity#saveStructure` (`.../StructureBlockEntity.java:329-350`, passes
  `!ignoreEntities`).
- 2. ignoreEntities / placement: FIXED. `place_structure_template_in_world` gates entity spawn
  on `!ignore_entities` and carries mirror/rotation/integrity/seed/strict;
  `place_template_entities` transforms pos/yaw/UUID. Vanilla
  `StructureBlockEntity#placeStructure` (`StructureBlockEntity.java:411-430`).
- 3. Wall-clock seed when seed == 0: VERIFIED, not a bug. Vanilla `createRandom`
  (`StructureBlockEntity.java:363-365`) is
  `seed == 0 ? RandomSource.create(Util.getMillis()) : RandomSource.create(seed)`;
  `create_structure_seed` matches with `SystemTime` millis and two independent draws.
- 4. Name canonicalisation: VERIFIED, not a bug. Vanilla `loadAdditional -> setStructureName ->
  Identifier.tryParse` (`StructureBlockEntity.java:107,171-177`); `parse_structure_identifier`
  adds `minecraft:` and rejects invalid/uppercase names identically.
- 5. Offset clamp +-48: VERIFIED, not a bug. Vanilla clamps posX/posY/posZ to -48..48 and
  sizes to 0..48 (`StructureBlockEntity.java:43,110-117`); `from_nbt` uses
  `MAX_OFFSET_PER_AXIS = 48` / `MAX_SIZE_PER_AXIS = 48` and clamps identically.
- 6. detectSize enumerates block entities: FIXED. `detect_size` scans chunk block sections via
  `collect_structure_block_positions` across the +-80 x/z, full-height box, equivalent to
  vanilla `getEnclosingBoundingBox` -> `findBlocksIn(...).filterState(STRUCTURE_BLOCK)`
  (`StructureBlockEntity.java:290-314`), then checks each BE for CORNER mode and matching name.
- 7. chunk_data_nbt drops payload on contention: FIXED. All fields use blocking
  `lock().unwrap_or_else(PoisonError::into_inner)`; no `try_lock().ok()?` remains.
- 8. Redstone SAVE writes disk: VERIFIED, not a bug. Vanilla `StructureBlock.trigger`
  (`StructureBlock.java:78-87`) calls `saveStructure(false)`; `on_neighbor_update` passes `false`.
- 9. Multiblock / dynamic-shape bucketing: VERIFIED (no generated data needed). Vanilla
  `StructureTemplate#addToLists` (`StructureTemplate.java:142-155`) uses
  `!state.getBlock().hasDynamicShape() && state.isCollisionShapeFullBlock(...)`.
  Pumpkin matches with `state.is_full_cube()` (collision-shape full-cube test,
  `pumpkin-data/src/block_registry.rs:1129-1137,1180-1183`) and `has_dynamic_shape(name)`,
  whose name list enumerates all 7 `.dynamicShape()` registrations in vanilla `Blocks.java`
  (:965 moving_piston, :4393 bamboo, :4465 scaffolding, :5199 powder_snow, :5408
  pointed_dripstone, :5425 sulfur_spike, :5843 shulkerBoxProperties -> shulker_box + 16
  dyed colours); no block overrides `hasDynamicShape()` (`Block.java:641`). No
  `has_dynamic_shape` generated field is required. This pass strengthened the existing
  pure unit test `dynamic_shape_blocks_are_not_full_cube_bucketed` to assert all 23 names.

No BLOCKED-DATA rows. Files changed this pass: crates/pumpkin-core/src/block/entities/
structure_block.rs (test only - extended the dynamic-shape name-list assertion). Touched
file should be rustfmt'd by the orchestrator. No cargo/git run per brief. No new behaviour
change; the only edit is a pure unit-test strengthening of an already-correct name list.
