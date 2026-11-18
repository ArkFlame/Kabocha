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
