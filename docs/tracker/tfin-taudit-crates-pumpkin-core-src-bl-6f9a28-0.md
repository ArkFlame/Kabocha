# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-bl-6f9a28.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: structure_block.rs

- 1. Entities never captured on save (`structure_block.rs:188-262`): FIXED / VERIFIED.
- 2. Placement ignores `ignoreEntities`/`StructurePlaceSettings`: FIXED / VERIFIED.
- 3. Wall-clock seed when `seed == 0` (`structure_block.rs:327-334`): VERIFIED (already matches vanilla).
- 4. Structure name canonicalisation (`from_nbt` 535-541): FIXED / VERIFIED.

## Result
1. Entities never captured on save: FIXED and VERIFIED. `capture_structure_template_from_world` gates entity capture on `include_entities`, uses `AABB.encapsulatingFullBlocks` (min..max+1), excludes players, stores relative `pos` and painting `block_pos` / floored `BlockPos.containing` block_pos. Matches `.vendor/vanilla-decompiled/net/minecraft/world/level/levelgen/structure/templatesystem/StructureTemplate.java` `fillFromWorld`/`fillEntityList` (lines 101-192). Callers pass `!ignoreEntities` (`StructureBlockEntity.java` saveStructure line 348-350).

2. Placement ignores `ignoreEntities`/`StructurePlaceSettings`: FIXED and VERIFIED. `StructurePlacementSettings` carries mirror, rotation, integrity, seed, strict and ignore_entities; `place_structure_template_in_world` checks `!ignore_entities` before spawning entities, skips an empty palette only when entities are also ignored, and honours strict via `new_with_skip_side_effects`. Matches `StructurePlaceSettings.java` (isIgnoreEntities / getRandomPalette lines 110-152) and `StructureTemplate.java` placeInWorld line 271 and placeEntities lines 521-548.

3. Wall-clock seed when `seed == 0`: VERIFIED (already correct). `create_structure_seed` uses `SystemTime::now()` millis when seed == 0 and the seed value otherwise, exactly matching vanilla `StructureBlockEntity.createRandom` (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/StructureBlockEntity.java` lines 363-365: `seed == 0L ? RandomSource.create(Util.getMillis()) : RandomSource.create(seed)`). No change needed.

4. Structure name canonicalisation (`from_nbt`): FIXED and VERIFIED. `from_nbt` (and `set_structure_name`) store `parse_structure_identifier`'s canonical `namespace:path`, defaulting invalid/empty names to "" and injecting `minecraft:` for unqualified names. Matches `.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/StructureBlockEntity.java` `setStructureName`/`getStructureName` (lines 163-177) using `Identifier.tryParse`.
