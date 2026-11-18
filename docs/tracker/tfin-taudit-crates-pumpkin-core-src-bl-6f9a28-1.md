# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-bl-6f9a28.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: structure_block.rs

- 5. Offset clamp to +-48 on load (\`from_nbt\` 543-554): VERIFIED (vanilla \`.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/StructureBlockEntity.java\`).
- 6. \`detectSize\` enumerates block entities (\`structure_block.rs:946-1036\`): VERIFIED (vanilla \`.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/StructureBlockEntity.java\`).
- 7. \`chunk_data_nbt\` drops payload (\`structure_block.rs:673-695\`): FIXED.
- 8. Redstone SAVE does not write disk (\`blocks/structure_block.rs:78-81\`): VERIFIED (vanilla \`.vendor/vanilla-decompiled/net/minecraft/world/level/block/StructureBlock.java\`).

## Result

- 5. Offset clamp to +-48 on load: VERIFIED. Vanilla \`StructureBlockEntity.loadAdditional\` (\`.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/StructureBlockEntity.java\`) clamps \`posX/posY/posZ\` to \`-48..48\` and \`sizeX/sizeY/sizeZ\` to \`0..48\` via \`Mth.clamp\`. \`StructureBlockBlockEntity::from_nbt\` in \`crates/pumpkin-core/src/block/entities/structure_block.rs\` applies the same clamps (\`MAX_OFFSET_PER_AXIS = 48\`, \`MAX_SIZE_PER_AXIS = 48\`). No change.
- 6. \`detectSize\` enumerates block entities: VERIFIED. Vanilla \`StructureBlockEntity.detectSize\`/\`getEnclosingBoundingBox\` (\`.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/StructureBlockEntity.java\`) scans \`level.findBlocksIn(...).filterState(is STRUCTURE_BLOCK)\` over +-80 x/z full height, then keeps CORNER block entities whose \`structureName\` matches. \`StructureBlockBlockEntity::detect_size\` in \`crates/pumpkin-core/src/block/entities/structure_block.rs\` scans chunk block palettes by state first (\`collect_structure_block_positions\` / \`is_structure_block_state\`) and then validates CORNER mode + matching name before \`compute_detected_bounds\`, matching vanilla semantics. No change.
- 7. \`chunk_data_nbt\` drops payload: FIXED. Vanilla \`getUpdateTag\` returns \`saveCustomOnly(registries)\` without a drop path (\`.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/StructureBlockEntity.java\`). \`StructureBlockBlockEntity::chunk_data_nbt\` in \`crates/pumpkin-core/src/block/entities/structure_block.rs\` writes all fields via blocking \`lock().unwrap_or_else(PoisonError::into_inner)\` helpers (\`write_string_fields\`, \`write_int_fields\`, \`write_bool_fields\`, \`write_number_fields\`); no \`try_lock\`-based drop remains, so transient contention cannot drop the payload. Verified in current code.
- 8. Redstone SAVE does not write disk: VERIFIED. Vanilla \`StructureBlock.trigger\` (\`.vendor/vanilla-decompiled/net/minecraft/world/level/block/StructureBlock.java\`) calls \`saveStructure(false)\` for SAVE mode (fills in-memory template, does not persist). \`crates/pumpkin-core/src/block/blocks/structure_block.rs\` \`on_neighbor_update\` calls \`structure_block.save_structure(args.world, false)\` for \`StructureblockMode::Save\`, matching vanilla. No change.

All rows closed as FIXED/VERIFIED. No source code changes required in this pass.

