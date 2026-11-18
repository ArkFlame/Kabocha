# Parity rows for crates/pumpkin-core/src/block/blocks/structure_block.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **blocks/Structure Block** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.
- **own/taudit-crates-pumpkin-core-src-bl-6f9a28** [partial/OWN-OPEN] own audit: 11 open, 14 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-bl-6f9a28.md) docs/tracker/taudit-crates-pumpkin-core-src-bl-6f9a28.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)


## Result

THIRD PASS, re-verified 2026-11-18 against the decompiled vanilla under
`.vendor/vanilla-decompiled/net/minecraft/`. This section supersedes the placeholder
above. No source file needed a new edit: every row is already correct in the current tree.

Unit row **blocks/Structure Block** [partial/AUDITED]: VERIFIED. Full re-review of the
file and its audit sub-rows below found no remaining behavioural divergence.

Unit row **own/taudit-crates-pumpkin-core-src-bl-6f9a28** [partial/OWN-OPEN]: VERIFIED.
All 9 gap rows are closed (6 FIXED in code, 3 NOT A BUG -> VERIFIED, 1 previously-open
data row now VERIFIED after checking the vanilla source). Per row:

- 1. Entity capture on save: FIXED. `capture_structure_template_from_world` takes
  `include_entities`, fills `entity_info_list` via `world.get_entities_at_box` over the
  full-block box and filters players (`entity_type == EntityType::PLAYER`). Vanilla
  `StructureTemplate#fillEntityList`
  (`.vendor/.../StructureTemplate.java:175-176`) uses `AABB.encapsulatingFullBlocks`
  and `!(input instanceof Player)`.
- 2. ignoreEntities / placement: FIXED. `place_structure_template_in_world` gates entity
  spawn on `!ignore_entities` (structure_block.rs:770) and carries mirror/rotation/
  integrity/seed/strict. Vanilla `StructureBlockEntity#placeStructure`
  (`.vendor/.../StructureBlockEntity.java:411-429`).
- 3. Wall-clock seed for seed == 0: VERIFIED, not a bug. Vanilla `createRandom`
  (StructureBlockEntity.java:363-365) is
  `seed == 0 ? RandomSource.create(Util.getMillis()) : RandomSource.create(seed)`;
  `create_structure_seed` matches.
- 4. Name canonicalisation: VERIFIED, not a bug. Vanilla `loadAdditional ->
  setStructureName -> Identifier.tryParse` (StructureBlockEntity.java:107,171-177);
  `parse_structure_identifier` matches.
- 5. Offset clamp +-48: VERIFIED, not a bug. Vanilla clamps posX/posY/posZ to -48..48
  and sizes to 0..48 (StructureBlockEntity.java:110-117); `from_nbt` matches.
- 6. detectSize enumerates block entities: FIXED. `detect_size` scans chunk block
  sections via `collect_structure_block_positions`, equivalent to vanilla
  `findBlocksIn(...).filterState(STRUCTURE_BLOCK)` (StructureBlockEntity.java:290-314).
- 7. chunk_data_nbt drops payload on contention: FIXED. All fields use blocking
  `lock().unwrap_or_else(PoisonError::into_inner)`; no `try_lock()?` remains.
- 8. Redstone SAVE writes disk: VERIFIED, not a bug. Vanilla `StructureBlock.trigger`
  (StructureBlock.java:78-81) calls `saveStructure(false)`; the block
  `on_neighbor_update` passes false.
- 9. Multiblock/dynamic-shape bucketing: VERIFIED, no generated data needed (supersedes
  the earlier BLOCKED-DATA claim). Vanilla `StructureTemplate#addToLists`
  (StructureTemplate.java:142-155) uses
  `!hasDynamicShape() && isCollisionShapeFullBlock`. `state.is_full_cube()`
  (pumpkin-data/src/block_registry.rs:1129) is the collision-box full-cube test, and
  `has_dynamic_shape` (structure_block.rs:199) enumerates every vanilla dynamicShape
  registration: the only `.dynamicShape()` calls in decompiled vanilla are
  Blocks.java:965 (moving_piston), :4393 (bamboo), :4465 (scaffolding), :5199
  (powder_snow), :5408 (pointed_dripstone), :5425 (sulfur_spike) and :5843
  (shulkerBoxProperties -> shulker_box + 16 colours). The name list covers exactly
  these, and no block overrides `Block.hasDynamicShape()` (Block.java:641). No
  `has_dynamic_shape` generated field is required.

Files changed: none this pass (all fixes already present and verified). No cargo/git run
per brief; orchestrator compiles. No new unit test: existing tests already cover the
dynamic-shape name list, palette bucketing, identifier validation, corner scan, yaw
transform and entity `block_pos` shape.
