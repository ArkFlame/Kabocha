# blocks / Structure Block (RE-REVIEW-UNTRACKED)

Target: `CURRENT_MC_VERSION`. Reference: `.vendor` absent, compared against the decompiled
Paper/Canvas tree in `/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft`
(`world/level/block/StructureBlock.java`, `world/level/block/entity/StructureBlockEntity.java`,
`world/level/levelgen/structure/templatesystem/StructureTemplate.java`,
`server/network/ServerGamePacketListenerImpl.java#handleSetStructureBlock`).

Fresh review of the rewritten source (`block/entities/structure_block.rs`,
`block/blocks/structure_block.rs`, `net/java/play/set_structure_block.rs`). No verified small
gap was found that could be fixed without a wider redesign; no source change in this pass.

Verified against vanilla (no action):

- `saveAdditional`/`loadAdditional` key set, defaults and clamps: `posX/Y/Z` clamped to +-48,
  sizes to 0..48, `ignoreEntities` default true, `showboundingbox` default true, `mode` default
  DATA, `strict` persisted (`StructureBlockEntity.java:81-129` vs `structure_block.rs:554-690`).
- `updateBlockState` uses `Block.UPDATE_CLIENTS` (`StructureBlockEntity.java:131-139`) =
  `BlockFlags::NOTIFY_LISTENERS` (`structure_block.rs:795-809`).
- Redstone trigger: `neighborChanged` + `trigger` (`StructureBlock.java:69-106`) matches
  `structure_block.rs:65-93`, including the `powered` latch and CORNER -> unload.
- `detectSize` bounds (`StructureBlockEntity.java:267-314`: X/Z +-80, full world height, single
  corner also encapsulates self, `delta > 1`, `min-pos+1` / `delta-1`) matches
  `structure_block.rs:965-1055`, `compute_detected_bounds` 474-519.
- `saveStructure` passes no extra ignore blocks and STRUCTURE_VOID is always ignored
  (`StructureBlockEntity.java:320-361`, `StructureTemplate.java:116`) matches
  `structure_block.rs:1064-1091` with `ignored_blocks = &[]` and the void skip at 242.
- `placeStructureIfSameSize` / `loadStructureInfo` (author + size only) match
  `structure_block.rs:1105-1129`.
- `handleSetStructureBlock` field order, `canUseGameMasterBlocks` gate, empty-name ->
  `invalid_structure_name`, save/load/scan message keys and the trailing block update match
  `set_structure_block.rs:60-138`. Vanilla does not set the author from the packet (author is
  only `createdBy` on place), so the current `player_placed` author is correct.

Remaining gaps:

| Location | Vanilla | Gap | Fix | Size |
|---|---|---|---|---|
| `block/entities/structure_block.rs:207-281` (`capture_structure_template_from_world`) | `StructureTemplate.fillFromWorld(level, pos, size, !ignoreEntities, ...)` fills `entityInfoList` (`StructureTemplate.java:133-137,175-193`) | Saving never records entities; `ignore_entities` (`:535,613`) and `packet.ignore_entities()` (`set_structure_block.rs:34-37`) are dead state | Add an entity pass to the capture and persist `entities` in the template NBT | L |
| `block/entities/structure_block.rs:377-472` (`place_structure_template_in_world`) | `placeInWorld` calls `placeEntities` when `!settings.isIgnoreEntities()` (`StructureTemplate.java:414-425`) | Loading never spawns template entities | Spawn from the template entity list, honouring `ignore_entities` | L |
| `block/entities/structure_block.rs:188-205,265` | `addToLists` uses `!state.getBlock().hasDynamicShape() && state.isCollisionShapeFullBlock(EmptyBlockGetter.INSTANCE, BlockPos.ZERO)` (`StructureTemplate.java:150`) | Name-substring heuristic misclassifies blocks (water/lava/cobweb/chest go to the "full" list), so palette order differs from vanilla | Use the generated full-cube state flag (`pumpkin-data/src/block_state.rs:131`); first confirm bit 7 is the collision-shape flag, it is not emitted by `tools/pumpkin-codegen` | S |
| `block/entities/structure_block.rs:464-469` | Non-strict placement runs `Block.updateFromNeighbourShapes` on every placed pos, then `updateNeighborsAt`, plus `updateShapeAtEdge` (`StructureTemplate.java:376-401,434+`) | Only `world.update_neighbors_at` runs, so neighbour-dependent states (fences, stairs, wall connections, attached blocks) are not corrected | Add an `update_from_neighbour_shapes` step before the neighbour update, guarded by `!strict` | M |
| `block/entities/structure_block.rs:405` | `createRandom(seed)` returns `RandomSource.create(...)`; legacy flag is not used for the integrity processor (`StructureBlockEntity.java:363-365,418-420`) | `LegacyRand` LCG decides integrity drops, so `integrity < 1` drops a different block set than vanilla | Switch to the non-legacy `RandomImpl` once exposed by `pumpkin_util::random` | M |
| `block/entities/structure_block.rs:410-412` | `placeInWorld` has no structure_void filter; void entries in the palette are placed (`StructureTemplate.java:288-311`) | Pumpkin skips structure_void placement and keeps whatever block is there | Confirm intended semantic (holes vs overwrite) then align with vanilla | S/M |
| `block/entities/structure_block.rs:446-448` | `LootTableSeed` is only written when the placed block entity is a `RandomizableContainer` (`StructureTemplate.java:322-324`) | Seed written whenever the template NBT carries `LootTable` | Gate on a randomizable-container check when the API exists | S |
| `block/entities/structure_block.rs:393-398` | Placement proceeds when the palette is empty but entities exist (`StructureTemplate.java:271`) | Returns false before entities could be placed | Falls out of the entity work above | S |

Note: `strict` is mapped to `StructurePlaceSettings.setKnownShape` correctly (skip neighbour
updates), but `place_structure_template_in_world` also uses it for `BlockFlags`; vanilla instead
adds `UPDATE_SKIP_ALL_SIDEEFFECTS` to the placement flags (`StructureBlockEntity.java:429`).
