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


## Result (native-4 cross-file pass)
- Row 3 (stonecutter-only recipe packet): BLOCKED-DATA. Needs generated recipe property sets in pumpkin-data/codegen; generated files must not be hand-edited.
- Row 1 (chat/command off-thread ordering): BLOCKED-L. Needs a per-player ordered dispatch queue in `net/java/` session handling, multi-file.

### Re-verified 2026-11-18 (final pass, main checkout)
- Decompiled ServerCommonPacketListenerImpl#handleKeepAlive (lines 103-144, Paper): accept a reply only when it matches an outstanding challenge (remove it + update latency); disconnect with disconnect.timeout for out-of-order, stale, duplicate or unknown ids. grep found no handleKeepAlive override in ServerGamePacketListenerImpl, so the common listener is authoritative. keep_alive.rs outstanding_keep_alive helper + only_an_outstanding_challenge_is_accepted test already match; mod.rs gates each new challenge on wait_for_keep_alive, so normally one challenge is outstanding and the accept-any-position path is observably equivalent. No further edit required. FIXED.
- Compile pitfalls re-check: no source edits this pass (keep_alive.rs already correct), so no import/helper/macro/initializer/clone/clippy changes to verify.
- Row 1 chat/command/session off-thread (mod.rs:908-977, 1201): STILL OPEN, M - needs a per-player ordered dispatch queue in mod.rs, outside this unit's keep_alive.rs.
- Row 2 config disconnect loses Component (mod.rs:651-670): STILL OPEN, M - needs a CConfigDisconnect protocol change in pumpkin-protocol, outside this unit.
- Row 3 stonecutter-only recipe packet (mod.rs:772-781): STILL OPEN, M - needs generated recipe property sets in pumpkin-data/codegen, outside this unit.

## Result (coder pass, main checkout)
- Re-read `crates/pumpkin-core/src/net/java/play/keep_alive.rs` against decompiled vanilla `ServerCommonPacketListenerImpl#handleKeepAlive` (`.vendor/vanilla-decompiled/net/minecraft/server/network/ServerCommonPacketListenerImpl.java:103-144`, no override in `ServerGamePacketListenerImpl`): accept only a reply whose id matches an outstanding challenge (remove it, update latency/ping, clear wait flag); disconnect with `disconnect.timeout` for unsolicited, stale, duplicate or out-of-order ids. Pumpkin's `outstanding_keep_alive` + pending removal and timeout disconnect match; `mod.rs` sends the next keep-alive only when `wait_for_keep_alive` is false, so only one challenge is normally outstanding and the implementation is observably equivalent to vanilla's single-pending/peek-first behaviour. Existing `only_an_outstanding_challenge_is_accepted` covers the seam. No source edits required. FIXED (in-scope).
- Row 1 chat/command/session spawned off-thread (mod.rs:908-977, 1201): STILL OPEN, M - needs a per-player ordered dispatch queue in `mod.rs`, outside this unit's allowed `keep_alive.rs`.
- Row 2 config disconnect loses Component (mod.rs:651-670): STILL OPEN, M - needs a `CConfigDisconnect` protocol/component change in `pumpkin-protocol`, outside this unit.
- Row 3 stonecutter-only recipe packet (mod.rs:772-781): STILL OPEN, M - needs generated recipe property sets in `pumpkin-data`/codegen, outside this unit and generated files must not be hand-edited.

## Result (second pass, main checkout, cross-file allowed)
- Row 1 chat/command/chat-ack/session ordering: FIXED already in tree. `mod.rs:400-411` `enqueue_ordered_packet` chains each job behind the previous one via `deferred_packet_tail` + `chain_after` (`mod.rs:105-115`), and `SChatAck`, `SChatCommand`, `SChatCommandSigned`, `SChatMessage` and `SPlayerSession` all dispatch through it (`mod.rs:942, 954, 976, 997, 1242`) instead of `spawn_task`, so arrival order is preserved like vanilla `ServerGamePacketListenerImpl#handleChat`/`#handleCommand`. No edit required.
- Row 2 config disconnect component: FIXED already in tree. `CConfigDisconnect` now holds `&TextComponent` and writes it with `write_component` (`crates/pumpkin-protocol/src/java/client/config/config_disconnect.rs:10-31`); both call sites pass a component (`net/java/mod.rs:699`, `net/java/pending.rs:353`). No edit required.
- Row 3 recipe packet: FIXED already in tree. `stonecutter_update_recipes_data` now writes the item property sets before the stonecutter list (`crates/pumpkin-protocol/src/java/client/play/recipe_book_add.rs:1204-1241`), with tests `update_recipes_includes_vanilla_property_sets` and `property_sets_include_datapack_cooking_and_smithing_recipes` covering the seam. No edit required.
- keep_alive.rs: unchanged and still correct; `handle_keep_alive` accepts only the oldest outstanding challenge and disconnects with `disconnect.timeout` otherwise, covered by `only_the_oldest_challenge_is_accepted`.
- No source edits this pass, so no compile-pitfall items to re-check (no new imports, helpers, struct initializers or clippy surfaces). Checks not run (worker cannot compile); orchestrator batch check covers it.
