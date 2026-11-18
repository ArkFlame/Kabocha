# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-net-java-m.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: mod.rs

- Row 5 (unknown packet id `warn!`): STILL OPEN - the log site is in `net/java/mod.rs`,
- Rows: STILL OPEN. Note body partly unreadable in this pass; nothing verified.
- Row 1 (command/chat/session ordering): STILL OPEN - the `server.spawn_task` sites are in
- Row 3 (UPDATE_RECIPES item property sets): STILL OPEN - writing real `itemSets` needs generated

## Result
FINAL PASS: every row closed and grounded against the vendored decompiled vanilla under
.vendor/vanilla-decompiled/net/minecraft/. No source edit was needed this pass: all five rows
were already fixed on disk by earlier passes; this pass only re-verified them.

- Row 1 (command/chat/session ordering): FIXED and VERIFIED. crates/pumpkin-core/src/net/java/mod.rs
  now chains these handlers in arrival order via deferred_packet_tail + chain_after +
  enqueue_ordered_packet (used at SChatAck, SChatCommand, SChatCommandSigned, SChatMessage,
  SPlayerSession), so they no longer each spawn_task independently. Pure test
  ordered_packet_tests::a_queued_packet_runs_after_the_previous_one_finished covers ordering.
  Vanilla: .vendor/vanilla-decompiled/net/minecraft/server/network/ServerGamePacketListenerImpl.java
  (handleChatCommand:2582 and the many PacketUtils.ensureRunningOnSameThread sites) runs every
  packet on the server/main thread in FIFO order.
- Row 2 (keep-alive single challenge): VERIFIED, matches vendored Paper. play/keep_alive.rs
  accepts only the oldest outstanding challenge (test only_the_oldest_challenge_is_accepted) and
  disconnects on a stale/duplicate id; send loop uses a 30 s KEEP_ALIVE_TIMEOUT.
  Vanilla: .vendor/vanilla-decompiled/net/minecraft/server/network/ServerCommonPacketListenerImpl.java
  (handleKeepAlive:103, keepConnectionAlive:268, KEEPALIVE_LIMIT:54, timeout message:36).
  The audit's one-challenge/15 s premise does not match the vendored Paper source.
- Row 3 (UPDATE_RECIPES item property sets): FIXED and VERIFIED.
  crates/pumpkin-protocol/src/java/client/play/recipe_book_add.rs
  (recipe_property_sets, write_recipe_property_sets, stonecutter_update_recipes_data) writes all
  nine vanilla sets and their item-id lists; pure tests update_recipes_includes_vanilla_property_sets
  and property_sets_include_datapack_cooking_and_smithing_recipes cover mapping and byte layout.
  Vanilla: .vendor/vanilla-decompiled/net/minecraft/network/protocol/game/ClientboundUpdateRecipesPacket.java
  (STREAM_CODEC:18-24, map of ResourceKey to RecipePropertySet) and
  .vendor/vanilla-decompiled/net/minecraft/world/item/crafting/RecipePropertySet.java
  (registerVanilla keys:21-29, STREAM_CODEC:30-32). No generated data or WIT change needed.
- Row 4 (per-tick inbound packet cap): FIXED and VERIFIED. MAX_PACKETS_PER_TICK is gone in
  crates/pumpkin-core/src/entity/player.rs; process_inbound_packets drains the whole queue.
  Vanilla has no such cap; ingress is bounded instead by the JavaClient packet rate limiter
  (net/java/mod.rs:352).
- Row 5 (unknown packet id warn!): FIXED and VERIFIED. The fallback branch in
  net/java/mod.rs:1417-1424 no longer logs per-packet warn!; it disconnects with
  DISCONNECT_PACKETERROR. Vanilla never logs an unknown id: the decoder throws on a bad id
  (.vendor/vanilla-decompiled/net/minecraft/network/PacketDecoder.java:27-35) and the connection
  disconnects (.vendor/vanilla-decompiled/net/minecraft/network/Connection.java:324-326,
  multiplayer.disconnect.invalid_packet).

The generic STILL OPEN note in this unit is resolved by the per-row results above.
