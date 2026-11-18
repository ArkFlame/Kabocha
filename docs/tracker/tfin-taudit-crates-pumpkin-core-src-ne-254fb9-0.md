# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-ne-254fb9.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: keep_alive.rs mod.rs

- 1 SChatMessage/SChatCommand/SPlayerSession spawned off-thread (net/java/mod.rs:908-977, 1201): STILL OPEN, M, needs per-player ordered queue.
- 2 config disconnect loses Component (mod.rs:651-670): STILL OPEN, M, protocol change.
- 3 stonecutter-only recipe packet (mod.rs:772-781): STILL OPEN, M, data/codegen.
- Row 1 (chat/command/session spawned off-thread, mod.rs): STILL OPEN, M, needs a per-player ordered dispatch queue in `mod.rs`, outside this unit's `keep_alive.rs`.

## Result
- Row 1 chat/command/chat-ack/session off-thread ordering: FIXED, VERIFIED. `net/java/mod.rs` routes `SChatAck` (:942), `SChatCommand` (:958), `SChatCommandSigned` (:980), `SChatMessage` (:1001) and `SPlayerSession` (:1246) through `enqueue_ordered_packet` (:404), which chains each job behind the previous `deferred_packet_tail` via `chain_after` (:107) so one client's packets run in arrival order. Vanilla handles these inline on the server thread: `.vendor/vanilla-decompiled/net/minecraft/server/network/ServerGamePacketListenerImpl.java:2552` (`handleChat`) and `:2582` (`handleChatCommand`). No edit required.
- Row 2 config disconnect loses Component: FIXED, VERIFIED. `CConfigDisconnect` holds `&TextComponent` and writes it with `write_component` (`pumpkin-protocol/src/java/client/config/config_disconnect.rs:10-31`); both call sites pass a component (`net/java/mod.rs:703`, `net/java/pending.rs:353`). Vanilla: `.vendor/vanilla-decompiled/net/minecraft/network/protocol/common/ClientboundDisconnectPacket.java:10` `record ClientboundDisconnectPacket(Component reason)`. No edit required.
- Row 3 stonecutter-only recipe packet: FIXED, VERIFIED. `stonecutter_update_recipes_data` writes the item property sets before the stonecutter list via `write_recipe_property_sets` (`pumpkin-protocol/src/java/client/play/recipe_book_add.rs:1187-1241`), covered by tests `update_recipes_includes_vanilla_property_sets` (:1409) and `property_sets_include_datapack_cooking_and_smithing_recipes` (:1431). Vanilla: `.vendor/vanilla-decompiled/net/minecraft/network/protocol/game/ClientboundUpdateRecipesPacket.java:16` carries `Map<ResourceKey<RecipePropertySet>, RecipePropertySet> itemSets`. No generated pumpkin-data field or codegen change needed. No edit required.
- `play/keep_alive.rs`: FIXED, VERIFIED. `is_oldest_challenge` accepts only the oldest outstanding id (remove it, update ping, clear wait flag), otherwise disconnects with `disconnect.timeout`; covered by `only_the_oldest_challenge_is_accepted`. Vanilla: `.vendor/vanilla-decompiled/net/minecraft/server/network/ServerCommonPacketListenerImpl.java:103-144` (`handleKeepAlive`, `pendingKeepAlives.peek()`), no override in `ServerGamePacketListenerImpl`. No edit required.
- Compile-pitfall re-check: no source edits this pass, so no import/helper/macro/struct-initializer/clone/clippy surfaces introduced. Checks not run (worker cannot compile); orchestrator batch check covers them.

### Final pass verification (main checkout)
All four items were re-read against the current tree and their decompiled vanilla classes (paths cited inline above). All are FIXED in tree with no remaining STILL OPEN rows. No source files were modified this pass; only this Result section was populated.
