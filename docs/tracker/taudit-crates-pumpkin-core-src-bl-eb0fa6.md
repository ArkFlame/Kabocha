# audit / crates/pumpkin-core/src/block/entities/mod.rs

Target: JavaMinecraftVersion::V_26_3. `.vendor` is absent, so vanilla comparisons use the
decompiled Paper tree knowledge and the surrounding Pumpkin code.

## Fixed in this pass

- `mod.rs:553-560` (`has_block_block_entity`) - compared `block.name` against
  `BLOCK_ENTITY_TYPES`, which is only correct while a block is named after its type.
  Vanilla `PistonBlock.isMovable` asks the block state's block entity type, so signs
  (`oak_sign`), banners (`white_banner`), skulls (`skeleton_skull`), shulker boxes
  (`white_shulker_box`) and wall signs were wrongly reported movable by pistons. Now reads
  `block.default_state.block_entity_type`, matching `on_placed`/`block_entity_name`.
- `mod.rs:378-392` (`block_entity_from_nbt`) - required `x`/`y`/`z`, dropping any block
  entity whose NBT lacked them. Vanilla `NbtHelper.toBlockPos` reads each coordinate with
  `getInt`, defaulting to 0, so the loader now does the same.
- Removed the stale `//TODO: We need a mark_dirty for chests`: chest-like entities already
  carry `dirty`/`comparator_dirty` in `chest_like_block_entity.rs`, and autosave,
  chunk unload and shutdown all call `save_all_block_entities`/`save_block_entities`.
- `mod.rs:530-536` (`block_entity_from_nbt`) - an unknown `id` returned `None` silently.
  Vanilla `BlockEntity.loadStatic` logs `Skipping block entity with id {}` before dropping
  the record, so a corrupt or newer save is now reported at `warn` with id and position.
  No test: the value is the log line, and the `None` result is already covered.

## Audit coverage (pass 2)

- No `TODO`/`FIXME` remain in the file; the six grep markers are doc comments naming the
  vanilla method each trait hook mirrors.
- All 49 entries of `BLOCK_ENTITY_TYPES` plus the Pumpkin-only `minecraft:bed` and
  `minecraft:map` have both a `create_block_entity` arm and a `block_entity_from_nbt` arm
  (checked mechanically), so no vanilla type is dropped or left unconstructable.
- Trait defaults checked against vanilla: `write_internal` = `saveWithFullMetadata`
  (`id`/`x`/`y`/`z` + `saveAdditional`), `preRemoveSideEffects` =
  `on_block_replaced` (`Containers.dropContents` + furnace XP), `client_update_tag`
  = `getUpdateTag`/`saveCustomOnly`, `collect*/apply*_item_components` =
  `collectImplicitComponents`/`applyImplicitComponents` for `CustomName` and `Lock`.
  `block_owns_block_entity` is enforced on chunk load, tick and pending-entity cleanup, as
  vanilla does through `BlockEntityType.isValid`.
- `ContainerMetadata` reads both `lock` (`LockCode`/`ItemPredicate` in 26.3) and legacy
  `Lock` strings, and `CustomName`/`custom_name`, matching vanilla plus its legacy path.

## Remaining gaps

| Location | Vanilla | Gap | Fix | Size |
|---|---|---|---|---|
| `mod.rs:571-590` (`create_block_entity_for_state`) | `PistonBlockEntity(BlockPos, BlockState)` takes the `FACING` property from the block state and starts a fresh extension (`extending = true`, progress 0) | Only hopper, structure block and command block build from the state; `moving_piston` falls through to `create_block_entity` -> `from_nbt(empty)`, so facing defaults to `Down`, `extending = false` and the pushed state is air | Add a `"piston"` arm reading the state facing (`PistonProps`/`PistonHeadLikeProperties`) and constructing the entity the way `piston.rs` does when it starts a move | S |
| `net/java/chunk_data/v1_18.rs:166-175` | Chunk block entity entries carry the `BlockEntityType` registry id | The writer repeats the `BLOCK_ENTITY_TYPES` lookup and falls back to `0` (furnace), and `pending_block_entities` does contain `minecraft:bed`/`minecraft:map` (Pumpkin-only, not in that list), so a Java client is told a furnace block entity sits on every bed and map. `get_id` has the same fallback but is unreachable for those two because neither overrides `chunk_data_nbt` | Filter the entry list before the count is written, dropping ids that are not in `BLOCK_ENTITY_TYPES`, and have `get_id`/the writer share one `Option<u32>` helper | S |
