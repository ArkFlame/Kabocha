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

## Gap status
- 1 SChatMessage/SChatCommand/SPlayerSession spawned off-thread (net/java/mod.rs:908-977, 1201): STILL OPEN, M, needs per-player ordered queue.
- 2 config disconnect loses Component (mod.rs:651-670): STILL OPEN, M, protocol change.
- 3 stonecutter-only recipe packet (mod.rs:772-781): STILL OPEN, M, data/codegen.
- Keep-alive pending, unknown id disconnect, fixed 30 s timeout (mod.rs:107 KEEP_ALIVE_TIMEOUT verified): FIXED per note.

## Result

- Row 1 (chat/command/session spawned off-thread, mod.rs): STILL OPEN, M, needs a per-player ordered dispatch queue in `mod.rs`, outside this unit's `keep_alive.rs`.
- Row 2 (config disconnect loses Component, mod.rs 651-670): STILL OPEN, M, protocol change in `pumpkin-protocol`, outside this unit.
- Row 3 (stonecutter-only recipe packet, mod.rs 772-781): STILL OPEN, M, needs generated recipe property sets in `pumpkin-data`/`recipe_helper`, outside this unit.
- `play/keep_alive.rs`: collapsed the duplicated "pending list" vtable path and the fallback `keep_alive_id` comparison into one `outstanding_keep_alive` helper. Only one keep-alive can be outstanding now, so the two branches were identical and the fallback allowed accepting a reply after the pending entry had been trimmed. Verified against `ServerCommonPacketListenerImpl#handleKeepAlive`: any id that is not an outstanding challenge disconnects with `disconnect.timeout`. Added unit test `only_an_outstanding_challenge_is_accepted`. FIXED.

### Re-verified 2026-11-18 (main checkout, re-audit pass)
- Row 1 chat/command/session off-thread (mod.rs:908-977, 1201): STILL OPEN, M - needs a per-player ordered dispatch queue in mod.rs, outside this unit's keep_alive.rs.
- Row 2 config disconnect loses Component (mod.rs:651-670): STILL OPEN, M - needs a CConfigDisconnect protocol change in pumpkin-protocol, outside this unit.
- Row 3 stonecutter-only recipe packet (mod.rs:772-781): STILL OPEN, M - needs generated recipe property sets in pumpkin-data/codegen, outside this unit.
- keep_alive.rs: re-verified against Paper decompiled ServerCommonPacketListenerImpl#handleKeepAlive (lines 103-144): accept only a reply matching an outstanding challenge (remove it + update ping), disconnect with timeout for unsolicited/stale/duplicate ids. The outstanding_keep_alive helper + only_an_outstanding_challenge_is_accepted test already match; no further change needed. FIXED.
- Final pass (main checkout): re-read decompiled ServerCommonPacketListenerImpl#handleKeepAlive lines 103-144 and confirmed keep_alive.rs matches (accept matching id, drop pending, update latency; disconnect with timeout otherwise). No new edit required. Rows 1-3 remain STILL OPEN, all in files outside this unit (mod.rs, pumpkin-protocol, pumpkin-data).
