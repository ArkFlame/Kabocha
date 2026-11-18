# Final rows from docs/tracker/tblocks-structure-block-328c1d.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- Old line 52, LootTableSeed gating (not marked STILL OPEN): FIXED a real mismatch. is_randomizable_container_id listed suspicious_sand and suspicious_gravel, but vanilla BrushableBlockEntity extends BlockEntity (not RandomizableContainer); the only implementors are RandomizableContainerBlockEntity subclasses (chest/trapped_chest, barrel, dispenser/dropper, hopper, shulker boxes, crafter) plus DecoratedPotBlockEntity. Removed the two brushable ids and extended loot_seed_only_for_randomizable_containers to assert they are excluded.
Only the LootTableSeed row needed an edit; the other STILL OPEN rows were already implemented in this checkout and are now verified against vanilla. rustfmt --edition 2024 run on the touched .rs file. Compile deferred to the orchestrator.
Third pass. Re-verified every STILL OPEN row against .vendor/vanilla-decompiled/net/minecraft
Fourth pass. Re-checked every STILL OPEN row against .vendor/vanilla-decompiled/net/minecraft

## Result
No STILL OPEN rows remain. All six "Remaining gaps" rows verified against `.vendor/vanilla-decompiled/net/minecraft/`:

- Row 49, entity capture and placement: VERIFIED. `capture_structure_template_from_world` (structure_block.rs:228-345) mirrors `fillEntityList` (StructureTemplate.java:175-192): players excluded, `AABB.encapsulatingFullBlocks`, painting uses `getPos()` for blockPos, other entities use `BlockPos.containing(pos)`. `place_template_entities` (structure_block.rs:461-512) mirrors `placeEntities` (StructureTemplate.java:511-552) behind `!ignore_entities`, including UUID removal, Pos/block_pos rewrites and rotate-then-mirror yaw. Verified against `.vendor/vanilla-decompiled/net/minecraft/world/level/levelgen/structure/templatesystem/StructureTemplate.java:175-192,511-552`.

- Row 50, collision-shape-full + hasDynamicShape: VERIFIED. Palette split uses `state.is_full_cube() && !has_dynamic_shape(name)` (structure_block.rs:285-293), matching `addToLists` (StructureTemplate.java:142-155). `has_dynamic_shape` (structure_block.rs:199-226) covers all seven `.dynamicShape()` sites in `Blocks.java` (moving_piston, bamboo, scaffolding, powder_snow, pointed_dripstone, sulfur_spike, shulkerBoxProperties). `is_full_cube` derives from extracted collision boxes, so glass sorts as a full block (test `palette_full_block_uses_collision_shape_not_occlusion`). Verified against `.vendor/vanilla-decompiled/net/minecraft/world/level/levelgen/structure/templatesystem/StructureTemplate.java:142-155` and `net/minecraft/world/level/block/Blocks.java`.

- Row 51, non-strict shape reconnect: VERIFIED. `!strict` pass runs `update_shape_at_edge` then per-position `update_from_neighbor_shapes` writing `NOTIFY_LISTENERS | UPDATE_KNOWN_SHAPE`, then `update_neighbors_at` (structure_block.rs:747-766), equal to `updateMode & ~UPDATE_NEIGHBORS | UPDATE_KNOWN_SHAPE` for non-strict (StructureTemplate.java:376-401, StructureBlockEntity.java:417,429). Verified against `.vendor/vanilla-decompiled/net/minecraft/world/level/levelgen/structure/templatesystem/StructureTemplate.java:376-401`.

- Row 52, LootTableSeed gating: FIXED (and verified). `is_randomizable_container_id` (structure_block.rs:423-436) lists exactly the `RandomizableContainer` implementors (chest/trapped_chest, barrel, dispenser/dropper, hopper, shulker boxes, crafter, decorated_pot) and excludes brushable block entities; `build_placed_block_nbt` seeds `LootTableSeed` only for those ids. Covered by `loot_seed_only_for_randomizable_containers`. Verified against `.vendor/vanilla-decompiled/net/minecraft/world/level/levelgen/structure/templatesystem/StructureTemplate.java:319-324`.

- Row 53, strict placement side effects: VERIFIED. `WorldBlockPlacer::new_with_skip_side_effects(world, strict)` (block_placer.rs:49-59) gates `on_block_replaced` drop callbacks, and `remove_block_entity` runs before every NBT-bearing write, mirroring the BARRIER pre-set (StructureTemplate.java:293-295) and `UPDATE_SKIP_ALL_SIDEEFFECTS` (StructureBlockEntity.java:429). Verified against `.vendor/vanilla-decompiled/net/minecraft/world/level/levelgen/structure/templatesystem/StructureTemplate.java:293-295`.

- Row 54, handleSetStructureBlock trailing update: VERIFIED, NO ACTION. `ServerLevel.sendBlockUpdated` (ServerLevel.java:3498-3574) only fires `blockChanged` plus path/navigation cache invalidation; `set_structure_block.rs` ends with `mark_dirty + update_block_entity + send_block_updated` (set_structure_block.rs:147-150), which already covers it. Verified against `.vendor/vanilla-decompiled/net/minecraft/server/network/ServerGamePacketListenerImpl.java:1317-1318` and `net/minecraft/world/level/ServerLevel.java:3498-3574`.

Out-of-scope observation (not a row): vanilla keeps `LiquidSettings.APPLY_WATERLOGGING` on and waterlogs a placed waterloggable block sitting in source water (StructureTemplate.java:276,291,332-341); Pumpkin does not read the fluid on this path. Left as a separate row.

No new source edits this pass; compile deferred to the orchestrator.
