# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-ne-254fb9.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: keep_alive.rs mod.rs

- Row 1 chat/command/session off-thread (mod.rs:908-977, 1201): VERIFIED. mod.rs chains SChatAck/SChatCommand/SChatCommandSigned/SChatMessage/SPlayerSession through enqueue_ordered_packet (chain_after over deferred_packet_tail), preserving arrival order like vanilla ServerGamePacketListenerImpl#handleChat/#handleChatCommand/#handleChatSessionUpdate. Covered by ordered_packet_tests::a_queued_packet_runs_after_the_previous_one_finished. Vanilla: .vendor/vanilla-decompiled/net/minecraft/server/network/ServerGamePacketListenerImpl.java:2552-3905.
- Row 2 config disconnect loses Component (mod.rs:651-670): VERIFIED. CConfigDisconnect now holds &TextComponent and writes it via write_component (crates/pumpkin-protocol/src/java/client/config/config_disconnect.rs); serialize_disconnect passes the component. Matches vanilla common ClientboundDisconnectPacket(Component) used in the config phase. Vanilla: .vendor/vanilla-decompiled/net/minecraft/network/protocol/common/ClientboundDisconnectPacket.java.
- Row 3 stonecutter-only recipe packet (mod.rs:772-781): VERIFIED. stonecutter_update_recipes_data writes item property sets before the stonecutter list (crates/pumpkin-protocol/src/java/client/play/recipe_book_add.rs:1204-1241), matching vanilla ClientboundUpdateRecipesPacket(itemSets, stonecutterRecipes). Covered by update_recipes_includes_vanilla_property_sets and property_sets_include_datapack_cooking_and_smithing_recipes. Vanilla: .vendor/vanilla-decompiled/net/minecraft/network/protocol/game/ClientboundUpdateRecipesPacket.java.
- keep_alive.rs: VERIFIED against ServerCommonPacketListenerImpl#handleKeepAlive (accept matching oldest challenge, drop pending, update latency; disconnect with timeout otherwise). .vendor/vanilla-decompiled/net/minecraft/server/network/ServerCommonPacketListenerImpl.java:103-144. Covered by only_the_oldest_challenge_is_accepted.

## Result
- Row 1 chat/command/session off-thread: VERIFIED (ServerGamePacketListenerImpl.java:2552-3905, ordering via chain_after).
- Row 2 config disconnect Component: VERIFIED (ClientboundDisconnectPacket common, component carried).
- Row 3 stonecutter recipe property sets: VERIFIED (ClientboundUpdateRecipesPacket.java).
- keep_alive.rs: VERIFIED (ServerCommonPacketListenerImpl.java:103-144).
No source edits this pass; rows already fixed in tree. Checks not run (worker cannot compile).
