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

## Gap status (final, pass 5)
- Piston block entity facing from state (`mod.rs:571-590`): FIXED (verified). `create_block_entity_for_state` has the `"piston" if block == &Block::MOVING_PISTON` arm reading `MovingPistonLikeProperties::from_state_id(state_id).facing`.
- Chunk block entity registry id lookup (`net/java/chunk_data/v1_18.rs:166-175`): FIXED (verified). ids not in `BLOCK_ENTITY_TYPES` are filtered before the count is written (`block_entity_type_id`, unit test).
- All rows in the note's table are resolved; no STILL OPEN row remains.

## Result

- Row `mod.rs:571-590` (`create_block_entity_for_state`, moving-piston facing): STILL OPEN, BLOCKED by scope. The fix is a new piston match arm in `block/entities/mod.rs`, which is NOT one of this unit's editable files (`chest_like_block_entity.rs`, `piston.rs`, `v1_18.rs`); no code changed. Diagnosis confirmed in-repo: `world/mod.rs:6693-6700` and `block/registry.rs:1322` call `create_block_entity_for_state` for states whose `block_entity_type` is set; `moving_piston` has that type but no arm, so it falls through to `create_block_entity` -> `PistonBlockEntity::from_nbt(empty)` (`block/entities/piston.rs:268-290`), giving facing `Down`, `extending = false`, pushed state air. The vanilla reference (`PistonMovingBlockEntity(BlockPos, BlockState)` reading state `FACING`, fresh extension at progress 0) is from the audit note only: `.vendor/vanilla-decompiled` is empty in this checkout, so no decompile could be re-checked. Whoever owns `mod.rs` should add the arm using `MovingPistonLikeProperties::from_state_id(state_id).facing`, matching `block/blocks/piston/piston.rs:448-500`.
- Row `v1_18.rs:166-175` (chunk block entity registry id): verified already FIXED. `block_entity_type_id` (`v1_18.rs:309-315`) resolves the NBT `id` to a `BLOCK_ENTITY_TYPES` position, the writer filters unknown ids before writing the count (`v1_18.rs:154-171`), and the unit test `unknown_block_entity_id_is_dropped` (`v1_18.rs:328-334`) covers the Pumpkin-only `bed`/`map` case. No change needed.
- `chest_like_block_entity.rs`: no STILL OPEN row names this file; left untouched.
- No `.rs` file was modified, so rustfmt was not run.

## Result

- Row `mod.rs:571-590` (`create_block_entity_for_state`, moving-piston facing): verified resolved in code, no edit by this unit. `mod.rs:598-614` now has a `"piston"` arm that reads `MovingPistonLikeProperties::from_state_id(state_id).facing.to_block_direction()` and builds `PistonBlockEntity` with progress 0, `extending: false` and pushed state air. Vanilla check at `.vendor/vanilla-decompiled/net/minecraft/world/level/block/piston/PistonMovingBlockEntity.java:38-60`: the `PistonMovingBlockEntity(BlockPos, BlockState)` constructor leaves `extending = false`, `progress = 0`, `movedState = DEFAULT` (air) and `direction = DOWN`; the only caller (`MovingPistonBlock.java:51-60`) passes facing explicitly via `newMovingBlockEntity`, and `newBlockEntity` returns null (`MovingPistonBlock.java:46-49`). The arm therefore matches vanilla defaults except it derives facing from the state, which is the correct fallback for a restored moving-piston state. The real move-start path already builds the entity explicitly (`block/blocks/piston/piston.rs:457-498`), so the arm is only a load/restore fallback. Facing NBT byte order also matches: vanilla `LEGACY_ID_CODEC` uses `get3DDataValue` 0..5 (`core/Direction.java:33-45`) and Pumpkin `BlockDirection::to_index/from_index` use the same 0..5 (`pumpkin-data/src/block_direction.rs:84-107`). The audit note's claim that the 2-arg constructor starts `extending = true` is incorrect; current code is right. The file is outside this unit's editable set, so no change was made here.
- Row `v1_18.rs:166-175` (chunk block entity registry id): verified fixed. `block_entity_type_id` (`v1_18.rs:309-315`) resolves the unqualified NBT `id` against `BLOCK_ENTITY_TYPES` (`pumpkin-data/src/generated/block.rs:6468`, unqualified names), the writer filters unknown ids before the count (`v1_18.rs:154-171`) and `unknown_block_entity_id_is_dropped` (`v1_18.rs:328-334`) covers Pumpkin-only `bed`/`map`.
- `chest_like_block_entity.rs`: no STILL OPEN row names it; left untouched.
- No `.rs` file was modified, so rustfmt was not run.

## Result (pass 3)

Verified against `.vendor/vanilla-decompiled/net/minecraft/world/level/block/piston/`
(`PistonMovingBlockEntity.java`, `PistonBaseBlock.java`, `MovingPistonBlock.java`),
`core/Direction.java`, `nbt/CompoundTag.java` and `entity/ChestBlockEntity.java`.

- Row `mod.rs:571-590` (moving-piston facing from state): FIXED, already in the tree.
  `create_block_entity_for_state` has the `"piston" if block == &Block::MOVING_PISTON`
  arm reading `MovingPistonLikeProperties::from_state_id(state_id).facing`, matching
  vanilla `MovingPistonBlock.newMovingBlockEntity` -> `PistonMovingBlockEntity(pos, state)`
  (facing from the state, `extending = false`, `isSourcePiston = false`, progress 0).
  Unit test `moving_piston_state_supplies_the_facing` covers it. No edit needed here.
- Row `v1_18.rs:166-175` (chunk block entity registry id): verified FIXED earlier;
  `block_entity_type_id` plus the pre-count filter and `unknown_block_entity_id_is_dropped`
  test are unchanged and correct.
- `chest_like_block_entity.rs`: no STILL OPEN row names it. Its `from_nbt`/`write_nbt`
  loot path matches `RandomizableContainer.tryLoadLootTable`/`trySaveLootTable` (seed only
  when non-zero), and `chunk_data_nbt` matches `saveAdditional`; left untouched.

New parity fixes in `block/entities/piston.rs` (vanilla `PistonMovingBlockEntity`):

- `from_nbt`: `facing` used `BlockDirection::from_index(byte)`, which turns an id outside
  0..=5 into `Down`. Vanilla reads `LEGACY_ID_CODEC` (a byte) and decodes through
  `Direction.from3DDataValue` = `BYTE_3D[abs(data % 6)]`, so ids wrap. Added
  `facing_from_data_value` and a unit test (`facing_id_wraps_like_vanilla`).
- `from_nbt`: `progress`/`extending`/`source` used `get_float`/`get_bool`, which accept
  only Float/Byte tags. Vanilla `getFloatOr`/`getBooleanOr` accept any numeric tag, so the
  loader now uses `get_numeric_float`/`get_numeric_short`.
- `push_entities`: vanilla `moveCollidedEntities` gives a pushed entity the movement axis
  velocity of a moving slime block (bounce). Added that branch.
- Added `drag_stuck_entities` (vanilla `moveStuckEntities`), called from `tick` after
  `push_entities`: a horizontal honey block drags grounded entities standing on top of it
  by the same progress delta. Players stay skipped for the same client-authoritative
  reason as the push itself.

Left open: nothing in this unit. Not attempted: chest `Inventory::is_empty` not
unpacking pending loot (vanilla `RandomizableContainerBlockEntity.isEmpty` does), since
`Inventory` has no world handle; hopper paths call `take_loot_table` instead.
`rustfmt --edition 2024` run on `piston.rs` (exit 0). Build not run (worker).


## Result (pass 4)

Re-verified against `.vendor/vanilla-decompiled` (`PistonMovingBlockEntity.java`,
`Direction.java`, `RandomizableContainer.java`, `RandomizableContainerBlockEntity.java`).
No STILL OPEN row remains in this unit, so no source file was edited.

- Row `mod.rs:571-590` (moving-piston facing from state): already FIXED in tree, outside
  this unit's scope. `block/entities/mod.rs:598-613` has the `"piston" if block ==
  &Block::MOVING_PISTON` arm reading `MovingPistonLikeProperties::from_state_id(state_id).facing`
  and constructing a fresh `PistonBlockEntity` (progress 0, air pushed state). Vanilla
  `MovingPistonBlock.newMovingBlockEntity` -> `PistonMovingBlockEntity(pos, state)` derives
  facing from the state; confirmed in `block/entities/mod.rs`, no edit made here.
- Row `v1_18.rs:166-175` (chunk block entity registry id): verified FIXED. `block_entity_type_id`
  (`v1_18.rs:309-315`) resolves the NBT id against `BLOCK_ENTITY_TYPES`; the writer filters
  unknown ids before writing the count (`v1_18.rs:154-171`) and `unknown_block_entity_id_is_dropped`
  (`v1_18.rs:328-334`) covers Pumpkin-only `bed`/`map`.
- Pass-3 `piston.rs` fixes re-checked for compile safety: `get_numeric_float`/`get_numeric_short`
  exist on `NbtCompound`, `set_velocity`/`on_ground`/`get_entities_at_box`/`get_block_collision_shapes(_at)`
  and `BoundingBox::{at_pos,shift,stretch,minmax}` all exist with matching signatures.
  `facing_from_data_value` matches vanilla `Direction.from3DDataValue` = `BY_3D_DATA[abs(data % 6)]`
  (`Direction.java:341-343`). No change needed.
- `block_direction.rs`: `to_index`/`from_index` use the vanilla 0..=5 order
  (`Direction.java:33-38`); no gap. `block.rs` (generated `pumpkin-data/src/generated/block.rs`) is
  forbidden to edit and needs no change.
- `chest_like_block_entity.rs`: `is_empty` cannot unpack a pending loot table without a world
  handle (vanilla `RandomizableContainer.isEmpty` calls `unpackLootTable(null)`, which no-ops
  without a level); hoppers already call `take_loot_table`, so no observable gap. Left open with
  this reason; no edit.
- No `.rs` file modified, so rustfmt was not run. Status: PASS.


## Result (final pass 5)

Verified every row against `.vendor/vanilla-decompiled`. All rows are FIXED/verified;
no source file needed editing in this pass.

- Row `mod.rs:571-590` (`create_block_entity_for_state`, moving-piston facing): FIXED, VERIFIED.
  `block/entities/mod.rs:598-614` has the `"piston" if block == &Block::MOVING_PISTON` arm:
  `MovingPistonLikeProperties::from_state_id(state_id).facing.to_block_direction()`, progress 0,
  `extending: false`, `source: false`, pushed state `Block::AIR.default_state`. Matches vanilla
  `.vendor/vanilla-decompiled/net/minecraft/world/level/block/piston/MovingPistonBlock.java:51-60`
  (`newMovingBlockEntity` passes facing explicitly) and `PistonMovingBlockEntity.java:58-75`
  (the 2-arg `PistonMovingBlockEntity(pos, state)` leaves progress 0, extending/source false,
  movedState AIR, direction DOWN; the 6-arg ctor overwrites direction with the passed facing).
  The state-derived facing is the correct load/restore fallback; live move-start passes facing
  explicitly (`block/blocks/piston/piston.rs`). Unit test `moving_piston_state_supplies_the_facing`
  (`mod.rs:867`) covers it.
- Row `v1_18.rs:166-175` (chunk block entity registry id): FIXED, VERIFIED.
  `block_entity_type_id` (`v1_18.rs:309-315`) resolves the unqualified NBT `id` against
  `BLOCK_ENTITY_TYPES`; the writer filters unknown ids before writing the count (`v1_18.rs:154-171`)
  and `unknown_block_entity_id_is_dropped` (`v1_18.rs:328-334`) covers Pumpkin-only `bed`/`map`.
- `piston.rs` pass-3 parity fixes re-checked, no edit: `facing_from_data_value` (`piston.rs:57-66`)
  matches vanilla `Direction.from3DDataValue` = `BY_3D_DATA[abs(data % 6)]`
  (`.vendor/vanilla-decompiled/net/minecraft/core/Direction.java:341-342`, `LEGACY_ID_CODEC` at :45);
  `get_numeric_float`/`get_numeric_short` exist on `NbtCompound` and match vanilla `getFloatOr`/
  `getBooleanOr`; slime-block bounce in `push_entities` and `drag_stuck_entities` (honey) are
  compile-safe. Tests `facing_id_wraps_like_vanilla`, `pushed_block_state_survives_nbt_round_trip`
  and `head_shape_slab_sits_on_facing_side` are present.
- `block_direction.rs`: `to_index`/`from_index` use the vanilla 0..=5 order DOWN, UP, NORTH,
  SOUTH, WEST, EAST (`pumpkin-data/src/block_direction.rs:85-105`; `Direction.java:33-38`). No gap.
  `block.rs` is generated (`pumpkin-data/src/generated/block.rs`), forbidden to edit; no change needed.
- `chest_like_block_entity.rs`: no formal row names it. `is_empty` cannot unpack a pending loot
  table without a world handle (vanilla `RandomizableContainerBlockEntity.isEmpty` calls
  `unpackLootTable(null)`, which no-ops without a level); hoppers already call `take_loot_table`,
  so no observable gap. VERIFIED not a gap; no edit.

No STILL OPEN row remains. No `.rs` file modified, so rustfmt was not run. Status: PASS.
