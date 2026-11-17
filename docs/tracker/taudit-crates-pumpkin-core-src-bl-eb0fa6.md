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

## Remaining gaps

| Location | Vanilla | Gap | Fix | Size |
|---|---|---|---|---|
| `mod.rs:571-590` (`create_block_entity_for_state`) | `PistonBlockEntity(BlockPos, BlockState)` takes the `FACING` property from the block state and starts a fresh extension (`extending = true`, progress 0) | Only hopper, structure block and command block build from the state; `moving_piston` falls through to `create_block_entity` -> `from_nbt(empty)`, so facing defaults to `Down`, `extending = false` and the pushed state is air | Add a `"piston"` arm reading the state facing (`PistonProps`/`PistonHeadLikeProperties`) and constructing the entity the way `piston.rs` does when it starts a move | S |
| `mod.rs:126-136` (`get_id`) | A Java block entity update packet carries the registry id of the `BlockEntityType` | `resource_location()` is looked up in `BLOCK_ENTITY_TYPES` and silently mapped to 0 (furnace) for Pumpkin-only types `minecraft:bed` and `minecraft:map`, which have no Java registry entry | Return `Option<u32>` (or a named constant) instead of 0 and let callers skip the packet; both entities currently avoid `CBlockEntityData` only because they never override `chunk_data_nbt` | XS |
| `mod.rs:378-392` (`block_entity_from_nbt`) | `BlockEntity.createFromNbt` logs the unknown/`null` id before dropping the record | Unknown `id` returns `None` with no log, so corrupt or newer saves lose entities silently | Add a `debug!`/`warn!` with the id and position before `None` | XS |
