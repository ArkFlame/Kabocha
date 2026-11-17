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
| 1 | `commands/item.rs:595-620` | 26.3 `item replace` takes a slot source (`SlotSource` argument, `minecraft:slot_source` datapack registry) resolving to one or more `SlotAccess`, and reports the number of replaced slots (`commands.item.block.replace.success`, `commands.item.entity.replace.success.*`) | The command parses `<slots>` with the single-slot `SlotArgumentType`, so ranges (`container.*`, `hotbar.*`, `weapon.*`, `armor.*`) fail with `slot.only_single_allowed` and only one slot is ever modified. `SlotsArgumentType` already exists and is unused. Needs the executors to iterate the resolved slots and to count failures (`commands.item.target.failed*`). | M |
| 2 | `commands/item.rs:595-620` | 26.3 has an `item set` subcommand (single slot) separate from slot-source `item replace`; translations `commands.item.block.set.success`, `commands.item.entity.set.success.single/multiple` exist for it | `item set` is not registered at all; its translation keys are dead. The current `replace` implementation is the old single-slot "set" behaviour. | M |
| 3 | `commands/item.rs:317-452` | `Entity#getSlot` ids 300-307 (mob inventory), 499 (`player.cursor` / `horse.chest`), 500-503 (player crafting), 500-514 (horse inventory) map to entity inventories/crafting grids | `read_entity_slot`/`write_entity_slot` and the duplicated inline player/entity branches only handle 0-35, 98-106 and 200-226. Horse/chest-boat inventories, mob inventories and the crafting grid fall through to "no such slot". | M |
| 4 | `commands/item.rs:484-524` | `applyItemModifier` runs the full loot-function set (`set_name`, `set_lore`, `set_components`, `set_enchantments`, `enchant_randomly`, `set_custom_model_data`, ...) | Only `minecraft:set_damage` and `minecraft:set_count` are implemented; every other function is silently skipped with a debug log, so a datapack modifier is partially applied. | L |
| 5 | `commands/item.rs:527-532` | Number providers support constant, uniform, binomial, score, storage and sum forms | Only a plain number or `{value:...}` is read. `{type:"uniform",min,max}` yields `None` -> defaults to `1.0`, silently producing the wrong count/damage. | S |
| 6 | `commands/item.rs:507-516` | `SetCountFunction` calls `stack.setCount(i)` for a non-add provider, which does not clamp to the item's max stack size (only `grow` clamps) | Both branches clamp to `get_max_stack_size()`, so `set_count` with a large value differs from vanilla. | S |
| 7 | `commands/item.rs:95`, `455-480` | `ItemCommand.setBlockItem`/`modifyBlockItem` call `container.setChanged()` so the block entity is saved | No dirty flag is set; `BlockEntity` still lacks `mark_dirty` (`block/entities/mod.rs:76` TODO), so `item replace`/`item modify` changes to container block entities can be lost on save. | M |
| 8 | `commands/item.rs:554-586` | 26.3 sends `commands.item.block.modify.success` and `commands.item.entity.modify.success.single/multiple` | Neither `item modify` executor sends feedback. | S |
| 9 | `commands/item.rs:122-281` | — (maintenance) | `EntityReplaceExecutor` reimplements the player/living slot mapping inline instead of using `read_entity_slot`/`write_entity_slot`, so fixes must be applied twice. No behavioural difference today. | S |

## Checked and matching

- Slot 98/99 -> selected hotbar slot / offhand 40 and 100-103 -> 36-39 (feet..head) match `LivingEntity#getSlot` + `Player#getSlot`.
- Ender chest mapping 200-226 -> 0-26 with a size check.
- `count` is applied to the stack before writing; `IntegerArgumentType::new(1, 99)` matches vanilla.
- Block target requires a loaded position and a block entity exposing an inventory; error args are x, y, z.
- `set_damage` clamps the durability fraction to 0..=1 and inverts it into damage like `SetDamageFunction`.
