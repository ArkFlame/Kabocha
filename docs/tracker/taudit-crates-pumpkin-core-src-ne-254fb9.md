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

Remaining gaps (not fixed here):

1. `mod.rs:908-977` (also `mod.rs:1201`) - `SChatMessage`, `SChatCommand`,
   `SChatCommandSigned` and `SPlayerSession` are each handed to
   `server.spawn_task`. Vanilla runs them inline on the server thread, so two chat
   lines/commands from one client are processed in packet order and the chat-ack
   state cannot race. Pumpkin can reorder them across Tokio workers.
   Vanilla: `ServerGamePacketListenerImpl#handleChat`, `#handleCommand`.
   Fix: per-player ordered dispatch queue (or handle inline and only offload the
   expensive plugin/command work). Size M.

2. `mod.rs:267-297` keep-alive timeout window - the deadline is
   `2 * networked.java.keep_alive_time` (30 s with the default 15) plus an extra
   `last_packet_time` inactivity check that vanilla does not have. Vanilla uses a
   fixed 30 s unanswered-keep-alive window and kicks only for the unanswered
   keep-alive. With a non-default `keep_alive_time` the timeout diverges.
   Vanilla: `ServerGamePacketListenerImpl#tick` (`30000L` / `disconnect.timeout`).
   Fix: use a fixed 30 s deadline and drop the generic inactivity clause. Size S.

3. `mod.rs:651-670` `serialize_disconnect` for the Config state sends
   `reason.get_text()` through `CConfigDisconnect::new(&str)`, dropping the
   component structure (colour, translate key, click events). Vanilla
   `ClientboundDisconnectPacket` carries a `Component` in config as in play.
   Fix: make `CConfigDisconnect` take `&TextComponent` (protocol change). Size M.

4. `mod.rs:1377-1379` unknown play packet id only logs a warning. Vanilla never
   reaches a default arm: an unknown id fails decoding and disconnects as a
   protocol error (`DecoderException` -> `onPacketError`). Fix: kick with the
   vanilla protocol-error reason. Size S, but plugins may rewrite `packet_id` in
   `PacketReceivedEvent`, so confirm that contract first.

5. `mod.rs:772-781` `stonecutter_recipes_packet` sends only the stonecutter list.
   Vanilla `ClientboundUpdateRecipesPacket` carries every `RecipePropertySet`
   (furnace, smithing, etc.), which is what the client uses to grey out recipe
   book entries after a datapack reload. Fix: generate the property sets in
   `pumpkin-data`/`recipe_helper` and send them in the same packet. Size M
   (data/codegen).
