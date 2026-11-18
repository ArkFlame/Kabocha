# command / item — deep audit

Sources: `crates/pumpkin-core/src/command/commands/item.rs`, `crates/pumpkin-command/src/argument_types/item_modifier.rs`, `crates/pumpkin-command/src/argument_types/slot.rs`, `crates/pumpkin-data/src/generated/slot_ranges.rs`, `crates/pumpkin-data/src/generated/translation.rs`.
Compared against vanilla `ItemCommand`, `SlotAccess`, `LivingEntity#getSlot`/`Player#getSlot`, `SetCountFunction`/`SetDamageFunction` (no decompiled reference in `.vendor/`; translation keys and the generated slot-source table are the 26.3 ground truth available in-tree).

Markers: the tracker note counts 3 markers, all are doc comments that mention "vanilla" (lines 301, 316, 348); there are no TODO/FIXME comments in the file. The only leftover marker is a `tracing::debug!` for unsupported modifier functions.

## Fixed in this review

- `item replace entity` reported success with `commands.item.entity.set.success.single/multiple` ("Replaced a slot on ..."), the single-slot `item set` wording, while `item replace block` had already been migrated to `commands.item.block.replace.success.known_item`. Switched to `commands.item.entity.replace.success.single.known_item` / `.multiple.known_item`, with the modified-slot count for a single target and the target count for multiple targets, matching vanilla's `replace` messages (`commands/item.rs:250-278`).
- `item modify` accepted a named modifier (`ItemModifier::Reference`) and then returned `Ok(0)` without touching the stack. Vanilla resolves the id through the `minecraft:item_modifier` registry. The reference is now looked up in `server.datapack_manager.get_item_modifier`, converted with `json_to_nbt_tag` and applied; an unknown id fails with `argument.id.unknown` (`commands/item.rs:534-552`). Inline modifiers still bypass the registry.

## Remaining gaps

| # | Location | Vanilla | Gap | Size |
|---|---|---|---|---|
| 1 | `commands/item.rs:595-620` | 26.3 `item replace` takes a slot source (`SlotSource` argument, `minecraft:slot_source` datapack registry) resolving to one or more `SlotAccess`, and reports the number of replaced slots (`commands.item.block.replace.success`, `commands.item.entity.replace.success.*`) | The command parses `<slots>` with the single-slot `SlotArgumentType`, so ranges (`container.*`, `hotbar.*`, `weapon.*`, `armor.*`) fail with `slot.only_single_allowed` and only one slot is ever modified. `SlotsArgumentType` already exists and is unused. Needs the executors to iterate the resolved slots and to count failures (`commands.item.target.failed*`). | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 2 | `commands/item.rs:595-620` | 26.3 has an `item set` subcommand (single slot) separate from slot-source `item replace`; translations `commands.item.block.set.success`, `commands.item.entity.set.success.single/multiple` exist for it | `item set` is not registered at all; its translation keys are dead. The current `replace` implementation is the old single-slot "set" behaviour. | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 3 | `commands/item.rs:317-452` | `Entity#getSlot` ids 300-307 (mob inventory), 499 (`player.cursor` / `horse.chest`), 500-503 (player crafting), 500-514 (horse inventory) map to entity inventories/crafting grids | `read_entity_slot`/`write_entity_slot` and the duplicated inline player/entity branches only handle 0-35, 98-106 and 200-226. Horse/chest-boat inventories, mob inventories and the crafting grid fall through to "no such slot". | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 4 | `commands/item.rs:484-524` | `applyItemModifier` runs the full loot-function set (`set_name`, `set_lore`, `set_components`, `set_enchantments`, `enchant_randomly`, `set_custom_model_data`, ...) | Only `minecraft:set_damage` and `minecraft:set_count` are implemented; every other function is silently skipped with a debug log, so a datapack modifier is partially applied. | L, STILL OPEN: size L, skipped per wave brief |
| 5 | `commands/item.rs:527-532` | Number providers support constant, uniform, binomial, score, storage and sum forms | Only a plain number or `{value:...}` is read. `{type:"uniform",min,max}` yields `None` -> defaults to `1.0`, silently producing the wrong count/damage. | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 6 | `commands/item.rs:507-516` | `SetCountFunction` calls `stack.setCount(i)` for a non-add provider, which does not clamp to the item's max stack size (only `grow` clamps) | Both branches clamp to `get_max_stack_size()`, so `set_count` with a large value differs from vanilla. | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 7 | `commands/item.rs:95`, `455-480` | `ItemCommand.setBlockItem`/`modifyBlockItem` call `container.setChanged()` so the block entity is saved | No dirty flag is set; `BlockEntity` still lacks `mark_dirty` (`block/entities/mod.rs:76` TODO), so `item replace`/`item modify` changes to container block entities can be lost on save. | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 8 | `commands/item.rs:554-586` | 26.3 sends `commands.item.block.modify.success` and `commands.item.entity.modify.success.single/multiple` | Neither `item modify` executor sends feedback. | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 9 | `commands/item.rs:122-281` | — (maintenance) | `EntityReplaceExecutor` reimplements the player/living slot mapping inline instead of using `read_entity_slot`/`write_entity_slot`, so fixes must be applied twice. No behavioural difference today. | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |

## Checked and matching

- Slot 98/99 -> selected hotbar slot / offhand 40 and 100-103 -> 36-39 (feet..head) match `LivingEntity#getSlot` + `Player#getSlot`.
- Ender chest mapping 200-226 -> 0-26 with a size check.
- `count` is applied to the stack before writing; `IntegerArgumentType::new(1, 99)` matches vanilla.
- Block target requires a loaded position and a block entity exposing an inventory; error args are x, y, z.
- `set_damage` clamps the durability fraction to 0..=1 and inverts it into damage like `SetDamageFunction`.


## Result

- Row 5 (number providers): FIXED for context-free forms. Verified against vanilla SetItemCountFunction / SetItemDamageFunction and the ContextIntProvider / ContextFloatProvider codecs in .vendor/vanilla-decompiled. number_provider now strips the minecraft: namespace (real SNBT uses namespaced type ids, so the old plain uniform match silently fell through to the 1.0 default), accepts all numeric NBT tag kinds, and supports uniform (nested min/max), binomial (n, p) and sum (inputs). score, storage and the remaining provider types require a LootContext (scoreboard / storage / level) that the item command does not build; that plumbing is outside this unit. Added 3 pure unit tests.
- Row 6 (set_count clamp): FIXED. Vanilla ItemStack.setCount(int) assigns the raw int (world/item/ItemStack.java:1305) without clamping to max stack size. Removed the get_max_stack_size() clamp; only the u8 representable range is kept.
- Row 8 (modify feedback): FIXED. Both item modify executors now send commands.item.block.modify.success (count, x, y, z) and commands.item.entity.modify.success.single/multiple (count + target name, or entity count), matching the generated translation strings.
- Row 1: OPEN - slot-source iteration needs the executor rewrite that resolves one-to-many slots and emits commands.item.target.failed*; M-sized, deferred.
- Row 2: OPEN - separate item set subcommand with its own single-slot executors and set-success feedback; M-sized, deferred.
- Row 3: OPEN - mob/horse/chest-boat inventories and the crafting grid need inventory APIs outside this unit's files.
- Row 4: OPEN (L) - the rest of the loot-function set needs component, enchantment and data-component APIs outside this unit.
- Row 7: OPEN - BlockEntity::mark_dirty lives in block/entities/mod.rs (outside this unit); item.rs cannot set the saved flag.
- Row 9: OPEN - pure maintenance dedup with no behavioural difference; skipped to avoid risk during parallel edits.

## Result

Pass 2 (verified against decompiled vanilla 26.3). The only supported version is `V_26_3` (`pumpkin-data/src/generated/packet.rs:3-6`), and generated `slot_ranges.rs` matches vanilla `SlotRanges.java` exactly. Rows 5/6/8 stay as recorded above; they were not re-audited in this pass.

- Row 1 (M): OPEN for the executor rewrite. One-to-many slot-source iteration lives in `commands/item.rs` (already using `SlotsArgumentType` at item.rs:85/223/1293/1303), which is outside this unit. Within this unit, `resolve_slot_range` in `argument_types/slot.rs` accepted stale pre-26.3 names (`villager.0-7` -> 300-307, `horse.saddle` -> 400, `horse.armor` -> 401) that vanilla 26.3 rejects; vanilla uses `mob.inventory.*`, `saddle` and `armor.body`, all present in the generated table. Removed the stale names and added 2 pure tests (reject old names, resolve vanilla equipment/crafting slots). `rustfmt --edition 2024` applied.
- Row 2 (M): OPEN - `item set` executors and Set-success feedback are in `commands/item.rs` (`SuccessFeedback::Set` at item.rs:85/87/223/225/1311/1316); outside this unit.
- Row 3 (M): OPEN - entity slot 300-307/499/500-514 mapping needs inventory APIs and lives in `commands/item.rs`; outside this unit.
- Row 4 (L): OPEN - remaining loot functions need component/enchantment/data-component APIs outside this unit.
- Row 7 (M): OPEN - `BlockEntity::mark_dirty` lives in `block/entities/mod.rs`; outside this unit.
- Row 9 (S): OPEN - maintenance dedup inside `commands/item.rs`, no behavioural difference; outside this unit.

## Result

Pass 3 (verified against decompiled vanilla 26.3 in `.vendor/vanilla-decompiled`). No source changes this pass: every remaining STILL OPEN row needs a file outside this unit (`commands/item.rs`, `block/entities/mod.rs`) or cross-crate inventory/component APIs. Verified the two editable argument types are already vanilla-correct, so no new edits or tests were added (would duplicate the existing tests).

- Row 1 (M): OPEN - one-to-many slot-source iteration and `target.failed*` counting live in the `commands/item.rs` executors (outside this unit). In this unit `SlotsArgumentType`/`resolve_slot_range` already match vanilla `SlotRanges.read`/`SlotsArgument.parse` (`SlotRanges.java`, `SlotArgument.java`, `SlotsArgument.java`); stale pre-26.3 names were removed in pass 2. Generated `slot_ranges.rs` matches vanilla `SlotRanges.java` exactly.
- Row 2 (M): OPEN - `item set` subcommand executors and Set-success feedback are in `commands/item.rs` (`SuccessFeedback::Set`); outside this unit.
- Row 3 (M): OPEN - entity slot 300-307/499/500-514 mapping needs inventory/crafting-grid APIs and lives in `commands/item.rs`; outside this unit. The slot-name table itself is verified correct.
- Row 4 (L): OPEN - the remaining loot functions need component/enchantment/data-component APIs in pumpkin-core (cross-crate). `ItemModifierArgumentType` here already parses reference-or-inline like vanilla `ResourceOrIdArgument.parse` (`ResourceOrIdArgument.java`); verified.
- Row 5 (S): already FIXED in pass 1 (number providers); not re-audited this pass.
- Row 6 (S): already FIXED in pass 1 (set_count does not clamp to max stack size); not re-audited this pass.
- Row 7 (M): OPEN - `BlockEntity::mark_dirty` lives in `block/entities/mod.rs` (outside this unit); `item.rs` cannot set the saved flag.
- Row 8 (S): already FIXED in pass 1 (modify feedback); not re-audited this pass.
- Row 9 (S): OPEN - maintenance dedup inside `commands/item.rs`, no behavioural difference; outside this unit.

## Result (native-1)

- Row 1 (replace slot-source one-to-many, failed counting): FIXED (block and entity paths) in `commands/item.rs` (`BlockReplaceExecutor` ~L79-190, `EntityReplaceExecutor` ~L213+). Emits `target.failed*` when no slot accepts the item.
- Row 2 (`item set` subcommand and set feedback): FIXED. `SuccessFeedback::Set` in `commands/item.rs:69-72` drives `COMMANDS_ITEM_BLOCK_SET_SUCCESS` and single-slot executors.
- Row 3 (entity slot 300-307 / 499 / 500-514): FIXED for mob and horse inventories (`read_mob_slot`, `write_mob_slot`, horse `500..=514` arm). Cursor and crafting-grid (499, 500-503 player) remain unmapped: NOT FIXED, BLOCKED-L (needs player menu/crafting-grid API).
- Row 4 (remaining loot functions): BLOCKED-L. Needs component, enchantment and data-component APIs outside this unit.
- Row 5 (number providers): FIXED. `number_provider` in `commands/item.rs` handles constant, uniform, binomial and sum (vanilla `NumberProvider`). Stale row.
- Row 6 (`set_count` clamp): NOT A GAP. Vanilla `SetItemCountFunction.run` calls `ItemStack.setCount` with no clamp (`.vendor/vanilla-decompiled/net/minecraft/world/level/storage/loot/functions/SetItemCountFunction.java:45`, `world/item/ItemStack.java:1305`). Pumpkin clamps only to the u8 storage range (`item.rs` set_count arm). Representation limit, not a parity gap.
- Row 7 (`mark_dirty` on container write): FIXED. `inventory.mark_dirty()` is called at `commands/item.rs:152` and `:834`. Stale row.
- Row 8 (`item modify` feedback): FIXED. `BlockModifyExecutor` and `EntityModifyExecutor` send `COMMANDS_ITEM_BLOCK_MODIFY_SUCCESS` and `COMMANDS_ITEM_ENTITY_MODIFY_SUCCESS_*` (`commands/item.rs` ~L1212-1280). Stale row.
- Row 9 (maintenance dedup of entity mapping): SKIPPED. Pure refactor with no behaviour difference; not a parity gap.



## Result

Pass 4 (verified against decompiled vanilla 26.3 in `.vendor/vanilla-decompiled`). No source changes in this unit: the two editable argument types (`argument_types/slot.rs`, `argument_types/item_modifier.rs`) are already vanilla-correct with adequate pure tests, so no edits or duplicate tests were added. Rows fixed elsewhere by native-1 (`commands/item.rs`) are noted; only cross-crate/API-bound gaps remain.

- Row 1 (M): addressed by native-1 in `commands/item.rs` (block/entity replace executors now iterate slot sources and emit `target.failed*`). In this unit `SlotArgumentType`/`SlotsArgumentType` match vanilla `SlotArgument.parse`/`SlotsArgument.parse` (`SlotRanges.read`, single-slot guard, `slot.only_single_allowed`, `SLOT_RANGE_SINGLE_SLOT_NAMES`/`SLOT_RANGE_ALL_NAMES` suggestions). Generated `slot_ranges.rs` matches vanilla `SlotRanges.java`.
- Row 2 (M): addressed by native-1 in `commands/item.rs` (`SuccessFeedback::Set` and single-slot executors). Argument parsing in this unit is vanilla-correct.
- Row 3 (M): partially addressed by native-1 (mob/horse inventory mapping). Cursor (499) and player crafting grid (500-503) remain BLOCKED-L (needs player menu/crafting-grid API outside this unit). The slot-name table in this unit was verified correct.
- Row 4 (L): OPEN/BLOCKED-L - remaining loot functions need component/enchantment/data-component APIs in pumpkin-core (cross-crate). `ItemModifierArgumentType` here already parses reference-or-inline like vanilla `ResourceOrIdArgument.parse` (`ResourceOrIdArgument.java`); verified.
- Row 5 (S): already FIXED in pass 1 (number providers); not re-audited.
- Row 6 (S): already FIXED in pass 1 (set_count does not clamp to max stack size); not re-audited.
- Row 7 (M): addressed by native-1 (`inventory.mark_dirty()` now called in `commands/item.rs`).
- Row 8 (S): already FIXED in pass 1 (modify feedback); not re-audited.
- Row 9 (S): SKIPPED - maintenance dedup inside `commands/item.rs`, no behavioural difference.


## Result (pass 4)

Re-verified every STILL OPEN row against decompiled vanilla 26.3 in `.vendor/vanilla-decompiled`. No behavioural fix was needed: all rows are either fixed/stale (1, 2, 5, 6, 7, 8) or need files/APIs outside this unit or generated data (3, 4, 9).

- Row 1: verified `SlotArgumentType`/`SlotsArgumentType` in `slot.rs` match vanilla `SlotArgument.parse` (reject multi-slot ranges with `slot.only_single_allowed`) and `SlotsArgument.parse`; `resolve_slot_range` reads until space like `SlotRanges.read`. Generated `slot_ranges.rs` matches `SlotRanges.java` exactly (mob.inventory.*, saddle, armor.body, weapon.*, player.crafting.*, no stale pre-26.3 names). One-to-many execution lives in `commands/item.rs` (outside this unit) and is already fixed.
- Row 2: `item set` executors/Set feedback live in `commands/item.rs` (outside this unit); already fixed. Generated `translation.rs` keys verified present.
- Row 3: OPEN - cursor (499) / player crafting-grid (500-503) mapping needs player menu APIs in `commands/item.rs` (outside this unit and this unit's files). Slot-name table itself is verified correct.
- Row 4: OPEN (L) - remaining loot functions need component/enchantment/data-component APIs in pumpkin-core (cross-crate). Parsing in `item_modifier.rs` already matches vanilla `ResourceOrIdArgument` (reference-or-inline SNBT).
- Row 5/6/8: already FIXED/stale in earlier passes; not re-audited.
- Row 7: OPEN - `BlockEntity::mark_dirty` lives in `block/entities/mod.rs` (outside this unit); recorded as FIXED in native-1 pass via `inventory.mark_dirty()`.
- Row 9: OPEN - maintenance dedup in `commands/item.rs`, no behavioural difference; outside this unit.

Added one pure unit test to `item_modifier.rs` (it had none) guarding the vanilla reference-vs-inline parse decision: an id resolves to `ItemModifier::Reference`, an SNBT compound to `ItemModifier::Inline`. No duplicate of the existing `slot.rs` tests. Cannot compile here (worker shell denied); orchestrator runs the gate.


## Result (third pass, final)

Verified every row against decompiled vanilla 26.3 in `.vendor/vanilla-decompiled`. The earlier "outside this unit" notes are stale: the cross-file fixes already landed in `crates/pumpkin-core/src/command/commands/item.rs`. No behavioural fix was needed: the cross-file rows were already resolved. Added one pure unit test (`sequence_runs_children_then_limit_count_clamps`) in `commands/item.rs` covering the previously untested `sequence`/`limit_count` seams; it does not duplicate any existing test.

- Row 1 (M) - FIXED/VERIFIED. `BlockReplaceExecutor`/`EntityReplaceExecutor` (`commands/item.rs:74-330`) parse slot sources with `SlotsArgumentType` for `replace` (single `SlotArgumentType` for `set`), iterate every resolved slot per target, and emit `commands.item.target.failed.known_item` when no slot accepts the stack (`item.rs:125-149`, `:254-283`); `response_counts` (`item.rs:192-211`) implements vanilla `ElementType.NON_ZERO` dispatch. In this unit `slot.rs` matches vanilla `SlotArgument.parse`/`SlotsArgument.parse` and generated `slot_ranges.rs` matches `SlotRanges.java`.
- Row 2 (M) - FIXED/VERIFIED. `SuccessFeedback::Set` (`commands/item.rs:68-72`) drives the separate `item set` subcommand with single-slot executors and `commands.item.block.set.success` / `commands.item.entity.set.success.single/multiple` feedback (`item.rs:175-184`, `:308-317`).
- Row 3 (M) - FIXED/VERIFIED. `read_player_menu_slot`/`write_player_menu_slot` (`item.rs:392-443`) map 499 to `containerMenu.getCarried()` and 500..=503 to `inventoryMenu.getCraftSlots()` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/player/Player.java:1893-1931`); `read_horse_slot`/`write_horse_slot` (`item.rs:445-501`) map horse chest/inventory (`.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/equine/AbstractHorse.java:81-82,980-983`); `read_mob_slot`/`write_mob_slot` (`item.rs:503-602`) map 300..=307 mob inventories; `read_container_slot`/`write_container_slot` (`item.rs:604-625`) cover chest boats/minecarts. Cursor and crafting grid are no longer blocked.
- Row 4 (L) - BLOCKED-DATA. Partial: `apply_item_modifier` (`item.rs:838-931`) now implements set_count, set_damage, set_name, set_lore, set_enchantments, set_components, enchant_randomly, set_custom_model_data, set_potion, sequence and limit_count, checked against the vanilla classes under `.vendor/vanilla-decompiled/net/minecraft/world/level/storage/loot/functions/` (`SetItemCountFunction.java`, `SetItemDamageFunction.java`, `SetNameFunction.java`, `SetLoreFunction.java`, `SetEnchantmentsFunction.java`, `SetComponentsFunction.java`, `EnchantRandomlyFunction.java`, `SetCustomModelDataFunction.java`, `SetPotionFunction.java`, `SequenceFunction.java`, `LimitCount.java`). The remaining vanilla functions (set_attributes, set_armor_trim, set_banner_pattern, set_book_cover, set_container_contents, set_custom_data, copy_components, copy_custom_data, copy_name, fill_player_head, exploration_map, set_fireworks, set_firework_explosion, set_instrument, set_ominous_bottle_amplifier, set_random_dyes, set_stew_effect, set_writable_book_pages, set_written_book_pages, toggle_tooltips, enchant_with_levels, ...) need a whole missing subsystem: data-component codecs in `read_data` plus a `LootContext` (scoreboard/storage/level) that `item modify` never builds. Several also need generated `pumpkin-data` component ids. Too large for a minimal edit.
- Row 5 (S) - FIXED/VERIFIED (pass 1). `number_provider` (`item.rs:1174-1224`) handles constant/uniform/binomial/sum; tests at `item.rs:1400-1452`.
- Row 6 (S) - FIXED/VERIFIED. Vanilla `ItemStack.setCount` assigns the raw int (`world/item/ItemStack.java:1305`, `SetItemCountFunction.java:45`) with no max-stack clamp; Pumpkin clamps only to the u8 storage range (`item.rs:859-860`), a representation limit, not a parity gap.
- Row 7 (M) - FIXED/VERIFIED. `inventory.mark_dirty()` at `commands/item.rs:152` and `:834` mirrors vanilla `Container.setChanged()`.
- Row 8 (S) - FIXED/VERIFIED. Modify executors send `commands.item.block.modify.success` / `commands.item.entity.modify.success.single/multiple` (`item.rs:1257`, `:1295`, `:1304`).
- Row 9 (S) - VERIFIED (no gap). Maintenance dedup only; both executors now share `write_entity_slot` (`item.rs:246`, `:1283`), so the original double-maintenance concern is resolved behaviourally. No parity impact.
## Result (coder pass, rows 8 only)
- 8. VERIFIED: vanilla sends `commands.item.block.modify.success` (slot count, x, y, z) at `.vendor/vanilla-decompiled/net/minecraft/server/commands/item/BlockItemAccessor.java:36` and `commands.item.entity.modify.success.single/multiple` at `EntityItemAccessor.java:40-41`. Rust `BlockModifyExecutor` sends the block message and `EntityModifyExecutor` sends single/multiple via `send_feedback` in `crates/pumpkin-core/src/command/commands/item.rs` (block executor ~l.1348-1375, entity executor ~l.1379-1421). Row was stale; no code change.
- Rows 1, 2, 3, 4, 5, 6, 7, 9: STILL OPEN, not reached in this pass (unchanged).

## Result (r3_018 re-derive, Gap table rows 1-9)

Re-derived from the Gap table against vanilla 26.3 in `.vendor/vanilla-decompiled`. No Rust edits this pass: every row checked already matches or is fixed in `crates/pumpkin-core/src/command/commands/item.rs`.

- Row 1 (slot-source replace): VERIFIED. `SlotsArgumentType` for replace (`item.rs:89`, `:227`); failed-target reporting (`item.rs:137`, `:266`). Vanilla `ItemCommands` replace/`SlotArgument.parse` (`.vendor/vanilla-decompiled/net/minecraft/server/commands/item/ItemCommands.java`).
- Row 2 (`item set` subcommand): VERIFIED. `SuccessFeedback::Set` (`item.rs:90`, `:228`, `:312`, `:317`).
- Row 3 (entity/player/horse slots 300-307, 499, 500-514): VERIFIED for the mapped slots. Cursor 499 and crafting 500-503 in `read_player_menu_slot`/`write_player_menu_slot` (`item.rs:398-443`); horse in `read_horse_slot` (`item.rs:451-461`); mob inventories via `read_mob_slot`/`write_mob_slot`. Matches `Player.java:1893-1931` and `AbstractHorse.java:81-82,980-983`.
- Row 4 (remaining loot functions): BLOCKED-DATA. `apply_item_modifier` (`item.rs:844-938`) now covers set_damage, set_count, set_name, set_lore, set_enchantments, set_components, enchant_randomly, set_custom_model_data, set_potion, sequence, limit_count. Still missing: the other loot function types need data-component and enchantment registry APIs that are not in pumpkin-core (missing subsystem). Unknown ids still hit `tracing::debug!` at `item.rs:928`.
- Row 5 (number providers): VERIFIED. `number_provider`/`number_provider_with` handle uniform, binomial, sum (`item.rs:1268-1299`).
- Row 6 (set_count clamp): VERIFIED. `set_count` assigns the raw value (`item.rs:855-863`), matching `SetItemCountFunction.java:45`. The post-modifier `min(max_stack)` at `item.rs:936` mirrors vanilla `ItemCommands.java:289` `limitSize`.
- Row 7 (container dirty flag): VERIFIED. `inventory.mark_dirty()` at `item.rs:156` and `:838`.
- Row 8 (modify feedback): VERIFIED. `COMMANDS_ITEM_BLOCK_MODIFY_SUCCESS` (`item.rs:1362`), `COMMANDS_ITEM_ENTITY_MODIFY_SUCCESS_SINGLE/MULTIPLE` (`item.rs:1404`, `:1413`).
- Row 9 (entity slot dedup): VERIFIED. Replace executor uses the shared `write_entity_slot` (`item.rs:250`, `:1392`); no behavioural gap.

Files edited: none. Checks: none run (cargo forbidden in this brief).
