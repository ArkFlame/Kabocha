# TODO left open: protocol, command, inventory, util, data

Resolved: `snbt/mod.rs` clean_and_append (iterator extend).

Left open, with reason:
- `pumpkin-protocol/src/java/client/play/spawn_entity.rs:17` unpack_degrees helper: no caller in this file, would be dead code.
- `pumpkin-util/src/random/mod.rs:51,92` unit tests: the functions are formula-mirroring, which AGENTS.md disallows.
- `pumpkin-inventory/src/player/player_screen_handler.rs:36,109,135`: Rust has no craftingResultInventory, and no offhand onEquipStack callback exists.
- `pumpkin-inventory/src/player/player_inventory.rs:11` NBT load: large, needs a design.
- `pumpkin-inventory/src/player/player_screen_handler.rs:49` Fully implement: open-ended.
- `pumpkin-protocol/src/codec/data_component.rs:371` MAX_LORE_LINES: cosmetic.
- `pumpkin-protocol/src/lib.rs:335` Style write: needs a Style encoder.
- `pumpkin-protocol/src/java/client/play/player_position.rs:55`, `teleport_entity.rs:44`: custom impl need requires an audit.
- `pumpkin-data/src/data_component_impl/combat.rs:348,374,1598`: needs typed block predicate and trim registries.
- `pumpkin-data/src/item_stack/mod.rs:477,490`: needs callsite audit.
- Bedrock enum TODOs (about 20): each needs a new enum type; not done in this pass.
- Java `dialog.rs:59`, `player_chat_message.rs:89`, `player_info_update.rs:64`: need an audit or ItemStack NBT serialization.
