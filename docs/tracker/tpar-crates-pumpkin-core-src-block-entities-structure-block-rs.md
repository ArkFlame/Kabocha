# Parity rows for crates/pumpkin-core/src/block/entities/structure_block.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/tblocks-structure-block-328c1d** [partial/OWN-OPEN] own audit: 14 open, 17 fixed, 31 verified, 0 blocked-data (docs/tracker/tblocks-structure-block-328c1d.md) docs/tracker/tblocks-structure-block-328c1d.md
- **own/tblocks-structure-block** [partial/OWN-OPEN] own audit: 9 open, 10 fixed, 1 verified, 0 blocked-data (docs/tracker/tblocks-structure-block.md) docs/tracker/tblocks-structure-block.md


## Result (final, third pass)

Both OWN-OPEN rows are closed. Every row that was ever marked STILL OPEN in the two
referenced audits was re-checked directly against the decompiled vanilla tree at
`/run/media/linsaftw/SDD/GitHub/ArkFlame/Kabocha/.vendor/vanilla-decompiled/net/minecraft/`.
All are already implemented in `crates/pumpkin-core/src/block/entities/structure_block.rs`
(and `net/java/play/set_structure_block.rs`) and match vanilla; no source edit was needed
this pass. Smallest first.

**own/tblocks-structure-block (rows 39-46): all VERIFIED**

- Row 46 (empty palette + entities, S): VERIFIED. Guard at structure_block.rs:685
  (`palette.blocks().is_empty() && (ignore_entities || entity_info_list.is_empty())`)
  matches `placeInWorld`'s early return at
  `world/level/levelgen/structure/templatesystem/StructureTemplate.java:271`.
- Row 44 (structure_void, S/M): VERIFIED, non-gap. Vanilla excludes STRUCTURE_VOID only at
  capture (`StructureBlockEntity.java:348-350`); `placeInWorld` has no void filter
  (`StructureTemplate.java:288-311`). Pumpkin skips void only at capture (:264).
- Row 45 (LootTableSeed, S): VERIFIED. `build_placed_block_nbt` (:778-804) gates the seed on
  `is_randomizable_container_id` (:423-436), matching `StructureTemplate.java:322-324`.
  The list is exactly chest/trapped_chest/barrel/dispenser/dropper/hopper/crafter/
  decorated_pot + all shulker boxes, with brushable blocks excluded.
- Row 41 (addToLists full-cube, S): VERIFIED. `StructureTemplate.java:142-155` uses
  `!hasDynamicShape() && isCollisionShapeFullBlock(...)`; Pumpkin gates the palette split on
  `state.is_full_cube() && !has_dynamic_shape(name)` (:285-293). `has_dynamic_shape` (:199-226)
  covers every `.dynamicShape()` declaration in `world/level/block/Blocks.java`; `is_full_cube`
  derives from extracted collision boxes, so glass sorts as a full block. Pure test
  `palette_full_block_uses_collision_shape_not_occlusion` covers this.
- Row 43 (integrity RNG, M): VERIFIED. Vanilla seeds the palette and placement RNGs
  independently (`StructureBlockEntity.java:419,429`); Pumpkin uses two streams from one seed,
  drops rejected entries with `continue`, and gates on raw integrity < 1.0 so NaN keeps the
  processor off, matching `BlockRotProcessor.processBlock` and `StructureBlockEntity.java:418-429`.
- Row 42 (non-strict neighbour shapes, M): VERIFIED. `StructureTemplate.java:377-411` runs
  `updateShapeAtEdge` then `Block.updateFromNeighbourShapes` writing
  `updateMode & ~UPDATE_NEIGHBORS | UPDATE_KNOWN_SHAPE`, then `updateNeighborsAt`, all behind
  `!getKnownShape`. Pumpkin mirrors this at :747-766 behind `!strict`
  (`setKnownShape(strict)` = `StructureBlockEntity.java:417`).
- Row 39/40 (entity capture/placement, L): VERIFIED. `fillEntityList`
  (`StructureTemplate.java:175-192`) excludes players, uses `AABB.encapsulatingFullBlocks` and
  `painting.getPos()`; `capture_structure_template_from_world` (:228-345) mirrors it.
  `place_template_entities` (:461-512) mirrors `placeEntities`
  (`StructureTemplate.java:511-552`): Pos/block_pos transforms, UUID removal, rotate-then-mirror
  yaw, spawn-with-passengers, behind `!ignore_entities` (:770).

**own/tblocks-structure-block-328c1d (Remaining-gaps rows 49-54): all VERIFIED**

- Old line 52 (LootTableSeed): VERIFIED (see row 45 above).
- Old line 50 (collision-shape-full + hasDynamicShape): VERIFIED (see row 41 above).
- Old line 51 (non-strict shape reconnect): VERIFIED (see row 42 above).
- Old line 53 (strict placement side effects): VERIFIED. Vanilla maps strict to both
  `setKnownShape` (`StructureBlockEntity.java:417`) and
  `UPDATE_SKIP_ALL_SIDEEFFECTS` (`:429`). Pumpkin's `WorldBlockPlacer::new_with_skip_side_effects`
  (`world/block_placer.rs:49-59`, used at structure_block.rs:692) gates `on_block_replaced`
  drops, and the `remove_block_entity` pre-clear for NBT entries mirrors the BARRIER pre-set
  (`StructureTemplate.java:293-295`).
- Old line 49 (entity capture and placement): VERIFIED (see rows 39/40 above).
- Old line 54 (handleSetStructureBlock trailing update): VERIFIED NO ACTION. Vanilla's
  `ServerLevel.sendBlockUpdated` (`world/level/ServerLevel.java`) fires only blockChanged plus
  path/navigation cache invalidation, no neighbour update; `set_structure_block.rs:147-150`
  (`mark_dirty` + `update_block_entity` + `register_block_change`) already covers it.

No STILL OPEN or BLOCKED-DATA rows remain. Pure unit tests in structure_block.rs already cover
the data-only seams (palette selection, dynamic shape, full-cube-vs-occlusion, loot-seed gating,
integrity/NaN gating, identifier validation, detect-size, yaw, edge-neighbour). The `Arc<World>`
placement paths have no pure-test seam. No source file was changed this pass; rustfmt was not
needed. Compile deferred to the orchestrator.
