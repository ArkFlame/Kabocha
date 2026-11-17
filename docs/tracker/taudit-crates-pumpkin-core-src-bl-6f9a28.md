# audit / crates/pumpkin-core/src/block/entities/structure_block.rs

Source reviewed: `crates/pumpkin-core/src/block/entities/structure_block.rs` (branch
kab/taudit-crates-pumpkin-core-src-bl-6f9a28), plus its callers
`block/blocks/structure_block.rs` and `net/java/play/set_structure_block.rs`.
Vanilla reference: `net.minecraft.world.level.block.entity.StructureBlockEntity`,
`StructureTemplate` (1.21.x). No `.vendor/` decompile in this worktree, so parity was checked
against the vanilla class behaviour. The file contains no TODO/FIXME markers; the three "vanilla"
hits are comments on `parse_structure_identifier`, `compute_detected_bounds` and a test.

## Fixed in this pass

- **Template palette bucketing used a block-name heuristic.** `capture_structure_template_from_world`
  split entries with `is_full_cube_template_block(name)` (substring match on "air", "slab", "glass"
  was not even listed), so water/lava/ice/glass/leaves and every non-full-cube block whose id lacks a
  keyword were written into the "full blocks" list. Vanilla
  `StructureTemplate#fillFromWorld` uses `blockState.isSolidRender(level, pos)` for that split.
  Now uses `BlockState::is_solid_render()` (`structure_block.rs:246`), the generated flag documented
  as vanilla `isSolidRender`, and the heuristic was deleted. Observable as different palette /
  block-index ordering in every saved `.nbt` (and in `Palette::blocks()` iteration order).

## Remaining gaps

1. **Entities are never captured when saving** (`structure_block.rs:188-262`,
   `save_structure` 1045-1072). Vanilla `StructureBlockEntity#saveStructure` calls
   `StructureTemplate#fillFromWorld(..., !this.ignoreEntities, ...)`, so entities inside the area are
   stored under `entities` unless "Ignore Entities" is set; both `ignoreEntities` and `strict` are
   loaded/saved here (594, 653) but only `strict` is ever read (place path). The runtime type already
   exists: `StructureTemplate::entities` / `TemplateEntity` (`pumpkin-world/src/generation/structure/
   template/structure_template.rs:207-215, 592`) and `save()` writes `ENTITIES_TAG`. Fix: add
   `include_entities` to `capture_structure_template_from_world`, query world entities inside the
   box, build `TemplateEntity` entries, and pass `!self.ignore_entities` from `save_structure`.
   Vanilla class `StructureBlockEntity#saveStructure`, `StructureTemplate#fillFromWorld`. Size M.
2. **Placement ignores `ignoreEntities` and bypasses `StructurePlaceSettings`**
   (`structure_block.rs:358-453`, `place_loaded_template` 1122). Vanilla builds a
   `StructurePlaceSettings` with mirror/rotation/ignoreEntities/integrity/seed and calls
   `StructureTemplate#placeInWorld(level, pos, pos, settings, random, 2)`; Pumpkin re-implements a
   minimal placer over `WorldBlockPlacer` and never reads `ignore_entities`, so a template with
   entities (imported from a datapack/vanilla world) always spawns them and no processor /
   waterlogging / known-shape handling runs. `StructurePlaceSettings` already exists in
   `pumpkin-world/src/generation/structure/template/structure_template.rs:35-60`. Size M.
3. **Placement seed is wall-clock based when `seed == 0`** (`structure_block.rs:327-334`). Vanilla
   derives the integrity random from `StructurePlaceSettings`/level random, not `SystemTime`; with
   the same saved structure block (seed 0, integrity < 1) two loads here give different results and
   cannot be reproduced, while vanilla is only as random as the level RNG. Fix: take the RNG from
   the caller/level when seed is 0. Vanilla `StructureBlockEntity#placeStructure`,
   `StructurePlaceSettings#getRandom`. Size S.
4. **Structure name is canonicalised and invalid names are dropped** (`from_nbt` 535-541,
   `set_structure_name` 800-809, `write_nbt` 613). Vanilla `StructureBlockEntity#loadAdditional`
   stores `tag.getString("name")` verbatim (`StructureTemplateManager` resolves it later), so a
   hand-written/lowercase-invalid name survives a chunk reload here only as "". Round-tripping a
   block entity whose `name` contains uppercase or `:` twice loses data. Fix: keep the raw string in
   the field and validate only at the save/load call sites. Vanilla class
   `StructureBlockEntity#loadAdditional`. Size S.
5. **Offsets are clamped to ±48 on load** (`from_nbt` 543-554). Vanilla clamps only
   `sizeX/Y/Z` to `0..48`; `posX/posY/posZ` are read unclamped (`getInt`). A datapack / NBT-editor
   offset of 100 is silently reduced here. Fix: drop the clamp on the three offsets, keep it on
   sizes. Vanilla class `StructureBlockEntity#loadAdditional`. Size S.
6. **`detectSize` enumerates block entities, not blocks** (`structure_block.rs:946-1036`). Vanilla
   scans `BlockPos.betweenClosed` in the ±80 x/z box and tests `level.getBlockState(pos)` plus the
   block entity; this walks `world.block_entities` and pending chunk BEs, so a structure block whose
   BE has not been inserted yet (or is queued for removal) is invisible and detection can differ
   around chunk borders. Behaviour matches for normal play; fix only if a differential test shows a
   miss. Vanilla `StructureBlockEntity#detectSize` / `filterRelatedBlocks`. Size S.
7. **`chunk_data_nbt` can drop the whole payload** (`structure_block.rs:673-695`). It uses
   `try_lock().ok()?` per field, so if any mutex is briefly held the block entity saves/exports
   nothing instead of block data. Not a vanilla-parity issue (vanilla copies plain fields, no locks)
   but the same file-local locking style; consider a single lock or `lock()` with poison recovery
   like the getters. Size S.
8. **Redstone SAVE does not write to disk** (`block/blocks/structure_block.rs:78-81` calls
   `save_structure(world, false)`). Vanilla 1.21 `StructureBlockEntity#saveStructure` persists the
   template through `StructureTemplateManager` on every trigger, so a redstone-saved structure is
   available to `placeStructure`/other blocks after a restart. Verify intended (likely deliberate to
   avoid disk writes in the tick) before changing. Size S-M.

## Checked, no change needed

- `compute_detected_bounds` single-corner/multi-corner rules, `-1` shrink and offset `+1` match
  vanilla `detectSize` and are covered by `detect_size_matches_single_and_multi_corner_vanilla_rules`.
- `mode/mirror/rotation` name and packet-id tables, NBT key spelling (`showair`,
  `showboundingbox`, `ignoreEntities`, `strict`), default `posY = 1` and default modes.
- `generated/<namespace>/structure/<path>.nbt` layout, gzip writing, in-memory `global_cache`
  registration and `structure void` skipping in `fillFromWorld`.
- Redstone LOAD/CORNER handling, `mode` block-state sync on NBT load, `save_structure_area` mode
  guard, `place_structure_if_same_size` "prepare" (size mismatch) flow and the translation keys used
  by `handle_set_structure_block`.
