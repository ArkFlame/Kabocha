# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-block-entities-structure-block-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: structure_block.rs

Final pass. Every row that was ever unresolved in the two referenced audits
(tblocks-structure-block.md and tblocks-structure-block-328c1d.md) was re-checked directly
against the decompiled vanilla tree at
`/run/media/linsaftw/SDD/GitHub/ArkFlame/Kabocha/.vendor/vanilla-decompiled/net/minecraft/`.
All rows are already implemented in `crates/pumpkin-core/src/block/entities/structure_block.rs`
(and `net/java/play/set_structure_block.rs`) and match vanilla, so no source edit was needed
this pass. Smallest rows first.

## Result

Rows inherited from own/tblocks-structure-block (rows 39-46), all VERIFIED:

- Row 46 (empty palette + entities, S): VERIFIED. Guard at structure_block.rs:685
  (`palette.blocks().is_empty() && (ignore_entities || template.entity_info_list.is_empty())`)
  matches `placeInWorld`'s early return at
  `world/level/levelgen/structure/templatesystem/StructureTemplate.java:270-274`
  (`!blockInfoList.isEmpty() || !settings.isIgnoreEntities() && !this.entityInfoList.isEmpty()`).
- Row 44 (structure_void, S/M): VERIFIED, non-gap. Vanilla excludes STRUCTURE_VOID only at
  capture (`world/level/block/entity/StructureBlockEntity.java:349`); `placeInWorld` has no void
  filter (`StructureTemplate.java:288-311`). Pumpkin skips void only at capture (:264).
- Row 45 (LootTableSeed, S): VERIFIED. `build_placed_block_nbt` (:778-804) gates the seed on
  `is_randomizable_container_id` (:423-436), matching `StructureTemplate.java:322-323`
  (`blockEntity instanceof RandomizableContainer`). The list is exactly chest/trapped_chest/
  barrel/dispenser/dropper/hopper/crafter/decorated_pot + all shulker boxes, with brushable
  blocks excluded. Pure test `loot_seed_only_for_randomizable_containers` covers it.
- Row 41 (addToLists full-cube, S): VERIFIED. `StructureTemplate.java:150` uses
  `!hasDynamicShape() && isCollisionShapeFullBlock(EmptyBlockGetter, ZERO)`; Pumpkin gates the
  palette split on `state.is_full_cube() && !has_dynamic_shape(name)` (:287-293).
  `has_dynamic_shape` (:199-226) covers every `.dynamicShape()` declaration in
  `world/level/block/Blocks.java` (moving_piston, bamboo, scaffolding, powder_snow,
  pointed_dripstone, sulfur_spike, 16 shulker boxes); `is_full_cube` derives from extracted
  collision boxes, so glass sorts as a full block. Pure test
  `palette_full_block_uses_collision_shape_not_occlusion` covers this.
- Row 43 (integrity RNG, M): VERIFIED. Vanilla seeds the processor RNG
  (`StructureBlockEntity.java:418-419`, gated on `integrity < 1.0F`) and uses a separate
  placement RNG (`:429`); Pumpkin uses two `LegacyRand` streams from one seed (:673,688),
  drops rejected entries with `continue` (:698), and gates on raw integrity < 1.0 so NaN keeps
  the processor off, matching `BlockRotProcessor.processBlock`. Pure test covers the RNG draws.
- Row 42 (non-strict neighbour shapes, M): VERIFIED. `StructureTemplate.java:388-395` runs
  `updateShapeAtEdge` then `Block.updateFromNeighbourShapes`, and `updateMode` for non-strict is
  `UPDATE_CLIENTS` (`:429`), i.e. `updateMode & ~UPDATE_NEIGHBORS | UPDATE_KNOWN_SHAPE` for the
  known-shape half. Pumpkin mirrors this at :747-766 behind `!strict`
  (`setKnownShape(strict)` = `StructureBlockEntity.java:417`).
- Row 39/40 (entity capture/placement, L): VERIFIED. `fillEntityList`
  (`StructureTemplate.java:175-192`) excludes players, uses `AABB.encapsulatingFullBlocks` and
  `painting.getPos()`; `capture_structure_template_from_world` (:228-345) mirrors it.
  `place_template_entities` (:461-512) mirrors `placeEntities` (`StructureTemplate.java:511-552`):
  Pos/block_pos transforms, UUID removal, rotate-then-mirror yaw, spawn-with-passengers, behind
  `!ignore_entities` (:770).

Rows inherited from own/tblocks-structure-block-328c1d (old lines 49-54), all VERIFIED:

- Old line 52 (LootTableSeed): VERIFIED (see row 45 above; `StructureTemplate.java:322-323`).
- Old line 50 (collision-shape-full + hasDynamicShape): VERIFIED (see row 41 above;
  `StructureTemplate.java:150`).
- Old line 51 (non-strict shape reconnect): VERIFIED (see row 42 above;
  `StructureTemplate.java:388-395,434-463`).
- Old line 53 (strict placement side effects): VERIFIED. Vanilla pre-sets a BARRIER with
  `UPDATE_SKIP_ALL_SIDEEFFECTS` before loading NBT (`StructureTemplate.java:294`) and adds that
  flag when strict (`StructureBlockEntity.java:429`). Pumpkin's
  `WorldBlockPlacer::new_with_skip_side_effects` (`world/block_placer.rs:49-59`, used at
  structure_block.rs:692) gates `on_block_replaced` drops, and the `remove_block_entity`
  pre-clear for NBT entries (:720-722) mirrors the BARRIER pre-set.
- Old line 49 (entity capture and placement): VERIFIED (see rows 39/40 above).
- Old line 54 (handleSetStructureBlock trailing update): VERIFIED NO ACTION. Vanilla's
  `ServerLevel.sendBlockUpdated` (`world/level/ServerLevel.java`) fires only blockChanged plus
  path/navigation cache invalidation, no neighbour update; `set_structure_block.rs:147-150`
  (`mark_dirty` + `update_block_entity` + `register_block_change`) already covers it.

No unresolved or BLOCKED-DATA rows remain. Pure unit tests in structure_block.rs already cover
the data-only seams (palette selection, dynamic shape, full-cube-vs-occlusion, loot-seed gating,
integrity/NaN gating, identifier validation, detect-size, yaw, edge-neighbour). The `Arc<World>`
placement paths have no pure-test seam. No source file was changed this pass; rustfmt was not
needed. Compile deferred to the orchestrator.
