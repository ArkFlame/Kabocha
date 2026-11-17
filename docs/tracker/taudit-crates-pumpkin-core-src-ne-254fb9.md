# audit: crates/pumpkin-core/src/net/java/mod.rs

Audited 2026-11-17 against vanilla `ServerGamePacketListenerImpl` /
`ServerCommonPacketListenerImpl` behaviour. No TODO/FIXME markers exist in the file
(the four "vanilla" hits are doc comments that already describe parity).

Fixed (branch kab/taudit-crates-pumpkin-core-src-ne-254fb9):

- `mod.rs` keep-alive timer: only one keep-alive may be outstanding, like vanilla
  `keepAlivePending`; previously a new challenge was queued every interval even when
  the previous one was unanswered (commit "send the next java keep-alive only after
  the previous reply").
- `play/keep_alive.rs`: an unsolicited or stale keep-alive id now disconnects with
  `disconnect.timeout` instead of being ignored, matching
  `ServerGamePacketListenerImpl#handleKeepAlive` (commit "kick java clients that
  answer an unknown keep-alive id").
- `mod.rs` keep-alive deadline is now a fixed 30 s (`KEEP_ALIVE_TIMEOUT`, Spigot
  `settings.timeout-time`) and the non-vanilla generic `last_packet_time` inactivity
  kick was dropped; the configured interval no longer scales the timeout (commit
  "use a fixed 30s java keep-alive timeout").
- `mod.rs` unknown play packet id now disconnects with `disconnect.packetError` like
  vanilla's decoder instead of only logging; note that a plugin rewriting
  `PacketReceivedEvent.packet_id` to an unknown id now closes the connection (commit
  "kick java clients that send an unknown play packet id").

Remaining gaps (not fixed here):

1. `mod.rs:908-977` (also `mod.rs:1201`) - `SChatMessage`, `SChatCommand`,
   `SChatCommandSigned` and `SPlayerSession` are each handed to
   `server.spawn_task`. Vanilla runs them inline on the server thread, so two chat
   lines/commands from one client are processed in packet order and the chat-ack
   state cannot race. Pumpkin can reorder them across Tokio workers.
   Vanilla: `ServerGamePacketListenerImpl#handleChat`, `#handleCommand`.
   Fix: per-player ordered dispatch queue (or handle inline and only offload the
   expensive plugin/command work). Size M.

2. `mod.rs:651-670` `serialize_disconnect` for the Config state sends
   `reason.get_text()` through `CConfigDisconnect::new(&str)`, dropping the
   component structure (colour, translate key, click events). Vanilla
   `ClientboundDisconnectPacket` carries a `Component` in config as in play.
   Fix: make `CConfigDisconnect` take `&TextComponent` (protocol change). Size M.

3. `mod.rs:772-781` `stonecutter_recipes_packet` sends only the stonecutter list.
   Vanilla `ClientboundUpdateRecipesPacket` carries every `RecipePropertySet`
   (furnace, smithing, etc.), which is what the client uses to grey out recipe
   book entries after a datapack reload. Fix: generate the property sets in
   `pumpkin-data`/`recipe_helper` and send them in the same packet. Size M
   (data/codegen).
