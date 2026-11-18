# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-net-java-m.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: mod.rs

- Rows: all three closed in the Result section below (final pass, re-verified against current
  source and decompiled vanilla; no remaining open or BLOCKED-DATA rows).
- Row 1 (command/chat/session ordering): FIXED, VERIFIED - see Result; per-connection FIFO chain
  in net/java/mod.rs (`chain_after` / `enqueue_ordered_packet`).
- Row 3 (UPDATE_RECIPES item property sets): FIXED, VERIFIED - see Result; all nine vanilla
  itemSets written by recipe_book_add.rs (`recipe_property_sets`).
- Row 5 (unknown packet id `warn!`): FIXED, VERIFIED - see Result; kick via
  DISCONNECT_PACKETERROR in net/java/mod.rs play dispatcher.

## Result

All three rows were already fixed on disk by earlier passes; this pass re-verified each against
decompiled vanilla and found no remaining gap. No code changes were needed.

- Row 1 (command/chat/session ordering): FIXED, VERIFIED. Vanilla
  `.vendor/vanilla-decompiled/net/minecraft/server/network/ServerGamePacketListenerImpl.java`
  routes every inbound play packet through `PacketUtils.ensureRunningOnSameThread` (e.g. lines
  531, 579, 858, 897-926), so chat/command/session handlers for one connection run strictly in
  arrival order on the server thread. `net/java/mod.rs` now implements the same per-connection
  FIFO: `chain_after` (lines 105-115) awaits the previous handle before polling the next task and
  `enqueue_ordered_packet` (lines 400-415) chains each job onto `deferred_packet_tail` under a lock
  (no `spawn_task` left in these arms). All five arms (`SChatAck` 942, `SChatCommand` 952,
  `SChatCommandSigned` 967, `SChatMessage` 989, `SPlayerSession` 1241) enqueue through it. Pure unit
  test `ordered_packet_tests::a_queued_packet_runs_after_the_previous_one_finished` (1441) covers
  the chain. The only remaining `spawn_task` calls are the outgoing writer and the unrelated
  respawn job, both intentionally unordered.

- Row 3 (UPDATE_RECIPES item property sets): FIXED, VERIFIED. Vanilla
  `.vendor/vanilla-decompiled/net/minecraft/world/item/crafting/RecipePropertySet.java` (lines
  21-29) defines exactly nine property-set keys (smithing_base/template/addition,
  furnace/blast_furnace/smoker/campfire_input, brewing_input/reagent) and
  `.vendor/vanilla-decompiled/net/minecraft/network/protocol/game/ClientboundUpdateRecipesPacket.java`
  (lines 15-24) carries them as the map `itemSets` before the stonecutter list. Pumpkin's
  `pumpkin-protocol/src/java/client/play/recipe_book_add.rs` writes all nine keys in
  `recipe_property_sets` (1167-1181) via `write_recipe_property_sets` (1187-1202) and prepends them
  in `stonecutter_update_recipes_data` (1208-1213). Unit test `update_recipes_includes_vanilla_property_sets`
  (1409) asserts all nine are present. No generated-data field was missing.

- Row 5 (unknown packet id `warn!`): FIXED, VERIFIED. Vanilla has no per-packet warn branch:
  `.vendor/vanilla-decompiled/net/minecraft/network/PacketDecoder.java` decode throws on a codec
  error (lines 27-35) and logs only a debug-level "IN" line (55-64), and unknown ids are rejected
  by `IdDispatchCodec` via
  `.vendor/vanilla-decompiled/net/minecraft/network/protocol/ProtocolCodecBuilder.java` (lines
  9-26), disconnecting the connection. The old `warn!` arm in `net/java/mod.rs` was replaced at
  lines 1417-1424 with a kick using `translation::java::DISCONNECT_PACKETERROR`
  (`disconnect.packetError`, generated in `pumpkin-data/src/generated/translation.rs:8441`), which
  matches vanilla's decoder rejection. No per-packet `warn!` remains in the play dispatcher.

Every PRIOR-OPEN row is now closed as FIXED and VERIFIED; no BLOCKED-DATA rows remain.
