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

- **Integrity handling diverged from vanilla `IntegrityProcessor`** (`structure_block.rs:390-419`).
  The loop rolled the RNG for every block (including block entities) and `continue`d on rejection,
  leaving the existing world blocks untouched. Vanilla rolls only for blocks whose
  `StructureBlockInfo.nbt() == null`, and replaces a rejected block with `Blocks.AIR`, so the placed
  region is cleared. Now the RNG is consumed only for NBT-less blocks (`next_f32() >= integrity`),
  matching vanilla ordering, and rejected positions are written as air. Behaviour difference was
  visible as leftover terrain inside a LOAD area with integrity < 1.

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

9. **Multiblock blocks are bucketed by `is_solid_render` alone** (`structure_block.rs:246`).
   Vanilla `fillFromWorld` puts a block in the "full blocks" list when
   `blockState.isSolidRender(...) || blockState.isMultiblock()`, and only non-solid non-multiblock
   states in the "other blocks" list. A non-full-cube multiblock (bed, door, piston head) is
   therefore ordered differently in the saved palette here; template content is identical, only the
   palette ordering can differ. Fix only if a byte-diff on saved `.nbt` shows the divergence matters
   (needs an `is_multiblock` flag, not present in `pumpkin-data`). Vanilla class
   `StructureTemplate#fillFromWorld`. Size S.

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

## Gap status
- 1. Entities never captured on save (`structure_block.rs:188-262`): STILL OPEN (M).
- 2. Placement ignores `ignoreEntities`/`StructurePlaceSettings`: STILL OPEN (M).
- 3. Wall-clock seed when `seed == 0` (`structure_block.rs:327-334`): STILL OPEN (S).
- 4. Structure name canonicalisation (`from_nbt` 535-541): STILL OPEN (S).
- 5. Offset clamp to +-48 on load (`from_nbt` 543-554): STILL OPEN (S).
- 6. `detectSize` enumerates block entities (`structure_block.rs:946-1036`): STILL OPEN (M).
- 7. `chunk_data_nbt` drops payload (`structure_block.rs:673-695`): STILL OPEN (S).
- 8. Redstone SAVE does not write disk (`blocks/structure_block.rs:78-81`): STILL OPEN (M).
- 9. Multiblock bucketing by `is_solid_render` (`structure_block.rs:246`): STILL OPEN (S).

## Result

Verified each STILL OPEN row against .vendor/vanilla-decompiled/net/minecraft/
(StructureBlockEntity, templatesystem/StructureTemplate, BlockState) on 2026-11-17.

- 1. Entity capture on save: OPEN. Vanilla fillFromWorld calls fillEntityList to save live entities
  unless ignoreEntities. Needs entity-NBT capture plus entity creation on placement; spawning/save
  APIs are outside these files (cross-crate).
- 2. ignoreEntities/StructurePlaceSettings: OPEN. Pumpkin never spawns template entities at all,
  so honouring the flag has no observable effect until cross-crate entity placement exists.
- 3. Wall-clock seed when seed == 0: NOT A BUG. Vanilla StructureBlockEntity.createRandom is exactly
  seed == 0 ? RandomSource.create(Util.getMillis()) : RandomSource.create(seed); Pumpkin matches. No change.
- 4. Name canonicalisation: NOT A BUG. Vanilla loadAdditional -> setStructureName ->
  Identifier.tryParse, which adds minecraft: and rejects invalid/uppercase names as null/empty.
  parse_structure_identifier matches this. No change.
- 5. Offset clamp +-48 on load: NOT A BUG. Vanilla loadAdditional clamps posX/posY/posZ to -48..48.
  Pumpkin matches. No change.
- 6. detectSize enumerates block entities: OPEN. Vanilla uses
  level.findBlocksIn(...).filterState(STRUCTURE_BLOCK); no equivalent block-iteration API here, and
  the doc says fix only on a differential miss. Left open.
- 7. chunk_data_nbt drops payload: FIXED. Per-field try_lock().ok()? replaced with the blocking
  lock().unwrap_or_else(std::sync::PoisonError::into_inner) already used by the getters, so transient
  contention can no longer make an entity save nothing. File: block/entities/structure_block.rs.
- 8. Redstone SAVE does not write disk: FIXED. Vanilla no-arg saveStructure() calls
  saveStructure(true); on_neighbor_update now passes true. File: block/blocks/structure_block.rs.
- 9. Multiblock/hasDynamicShape bucketing: OPEN. Vanilla addToLists uses
  !hasDynamicShape() && isCollisionShapeFullBlock; Pumpkin is_full_cube() covers the second half,
  but pumpkin-data exposes no has_dynamic_shape flag (generated data, outside this unit).

Files changed: crates/pumpkin-core/src/block/blocks/structure_block.rs,
crates/pumpkin-core/src/block/entities/structure_block.rs. Both run through
rustfmt --edition 2024. No cargo/git run (per brief). No new unit test: neither fix has a pure seam
(mutex-contention timing / redstone tick) testable without a live world.


## Result

Re-verified on 2026-11-18 against .vendor/vanilla-decompiled/net/minecraft/
(StructureBlockEntity, StructureBlock, templatesystem/StructureTemplate). This section
supersedes the earlier Result block: the code has since been reworked well past that
snapshot (entity capture, placement, block-section scan and blocking payload locks are
all already present). Per-row outcome:

- 1. Entity capture on save: FIXED in code. `capture_structure_template_from_world` takes
  `include_entities`, fills `entity_info_list` over `AABB.encapsulatingFullBlocks` via
  `world.get_entities_at_box`, and derives `block_pos` from the three-int `block_pos` codec.
  This pass closed one remaining divergence: vanilla `fillEntityList` filters
  `!(input instanceof Player)` but `get_entities_at_box` also returns players, so players
  inside the captured box were saved into the template. Added a player-type filter in
  `block/entities/structure_block.rs` (new import `pumpkin_data::entity::EntityType`).
- 2. ignoreEntities / StructurePlaceSettings: FIXED in code. `place_template_entities` is
  gated by `!ignore_entities` and transforms pos/yaw/UUID like vanilla `placeEntities`;
  placement settings carry mirror/rotation/integrity/seed/strict. No change.
- 3. Wall-clock seed when seed == 0: NOT A BUG. Vanilla `createRandom` is
  `seed == 0 ? RandomSource.create(Util.getMillis()) : RandomSource.create(seed)`;
  `create_structure_seed` matches, including two independent draws for integrity and loot
  as vanilla makes two `createRandom` calls. No change.
- 4. Name canonicalisation: NOT A BUG. Vanilla `loadAdditional -> setStructureName ->
  Identifier.tryParse`; `parse_structure_identifier` adds `minecraft:` and rejects invalid
  or uppercase names identically. No change.
- 5. Offset clamp +-48: NOT A BUG. Vanilla clamps posX/posY/posZ to -48..48; Pumpkin uses
  `MAX_OFFSET_PER_AXIS = 48`; sizes clamp 0..48 as vanilla. No change.
- 6. detectSize enumerates block entities: FIXED in code. `detect_size` now reads chunk
  block sections (`collect_structure_block_positions`) across the +-80 x/z, full-height
  box, the equivalent of vanilla `findBlocksIn(...).filterState(STRUCTURE_BLOCK)`, then
  checks each BE for CORNER mode and a matching name. No change needed.
- 7. chunk_data_nbt drops payload: FIXED in code. Every field uses
  `lock().unwrap_or_else(PoisonError::into_inner)` like the getters; no `try_lock().ok()?`
  remains. No change.
- 8. Redstone SAVE does not write disk: NOT A BUG. Vanilla `StructureBlock.trigger` calls
  `saveStructure(false)`, so the redstone path deliberately does not persist to disk;
  Pumpkin passes `false` and registers the template to the in-memory cache. The earlier
  Result block's claim that redstone passes `true` was wrong and is superseded. No change.
- 9. Multiblock bucketing: OPEN (needs generated data). Vanilla `addToLists` uses
  `!hasDynamicShape() && isCollisionShapeFullBlock(...)`; Pumpkin matches the second half
  with `is_full_cube()` and approximates `hasDynamicShape` via a name list. No
  `has_dynamic_shape` flag exists in pumpkin-data, so exact parity needs regenerating
  generated data (outside this unit).

Files changed this pass: crates/pumpkin-core/src/block/entities/structure_block.rs
(player filter + `EntityType` import). `rustfmt --edition 2024` run on it. No cargo/git run
(per brief); the orchestrator compiles. No new unit test: the only new behaviour lives in
a live-`World` capture path with no pure seam, and the existing tests already cover palette
bucketing, identifier validation, the corner scan, yaw transform and entity `block_pos` shape.
