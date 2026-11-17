# net / java / mod.rs (AUDIT)

Source: `crates/pumpkin-core/src/net/java/mod.rs` (1376 lines), plus the play handlers it
dispatches to (`crates/pumpkin-core/src/net/java/play/*.rs`).
Reference: vanilla `ServerGamePacketListenerImpl` / `ServerCommonPacketListenerImpl` /
`ClientboundUpdateRecipesPacket`, Paper's `ServerGamePacketListenerImpl` (same packet flow on the
main thread, no extra packet-level gates) from knowledge; no decompiled jar is vendored in this
worktree.

## Verdict

No `TODO`/`FIXME` markers exist. The tracker note counts 4 occurrences of the word "vanilla", all
in doc comments (mod.rs:133-134, 149, 765) that describe matching behaviour; two of them document
real gaps (keep-alive, UPDATE_RECIPES property sets) listed below.

Fixed in this audit (commit):
- Config-state disconnect dropped the component: `CConfigDisconnect` wrapped a flattened
  `get_text()` string in `TextComponent::text`, so colour and translation keys were lost (a
  `translate_cross!` kick showed the raw key in the config phase). Vanilla's config
  `ClientboundDisconnectPacket` carries a full `Component`. Now the packet holds
  `&TextComponent` and writes it with `write_component`, like `CPlayDisconnect`
  (`crates/pumpkin-protocol/src/java/client/config/config_disconnect.rs`,
  `crates/pumpkin-core/src/net/java/pending.rs:338`,
  `crates/pumpkin-core/src/net/java/mod.rs:653-656`).

## Remaining gaps

1. **Commands, chat and session updates run concurrently, not in arrival order** —
   `mod.rs:905-970` (`SChatCommand`, `SChatCommandSigned`, `SChatMessage`) and
   `mod.rs:1194-1206` (`SPlayerSession`) each `server.spawn_task` a separate async handler.
   Vanilla runs every packet of one connection on the server thread through a FIFO, so two
   `/command` packets execute strictly in order and never interleave. Here `/tp` and `/give`
   sent back to back can run in either order and mutate world/player state concurrently.
   Vanilla class: `ServerGamePacketListenerImpl#handleChatCommand` via
   `PacketUtils.ensureRunningOnSameThread`. Fix (M): give `Player` a per-connection ordered
   job queue drained by one task (or an ordered `tokio` `mpsc` with a single consumer), enqueue
   these handlers instead of spawning, and await the previous one before the next.
2. **Keep-alive tracks many outstanding ids instead of vanilla's single challenge** —
   `mod.rs:281-302` sends a new keep-alive every interval while previous ones are unanswered and
   keeps up to 16 in `pending_keep_alives`; `play/keep_alive.rs:10-28` accepts *any* pending id and
   clears `wait_for_keep_alive`. Vanilla has one `keepAliveChallenge` per connection: a reply with
   any other id is logged and ignored, and `keepAliveTime` (reset on send) triggers
   `disconnect.timeout` 15 s after the send, not after 2x the send interval.
   Vanilla class: `ServerCommonPacketListenerImpl#sendKeepAlive/#handleKeepAlive/#tick`.
   Fix (S): store one outstanding id + send instant, accept only that id, kick when it is older
   than the configured timeout, and generate the id with a random long per connection instead of
   the send timestamp (`mod.rs:281-286`).
3. **UPDATE_RECIPES omits the item property sets** — `mod.rs:765-774` and
   `play/recipe_book_add.rs:974` send "no item property sets, then the stonecutter list". Vanilla
   `ClientboundUpdateRecipesPacket` carries `itemSets` (furnace fuel/input, smithing base/additions,
   etc. built by `RecipeManager#createRecipePropertySets`). Without them the client cannot grey out
   or auto-place recipes that need properties. Fix (M): build the property sets from the recipe
   data in `pumpkin-data`/`recipe_helper.rs` and prepend them to the payload.
4. **Per-tick inbound packet cap has no vanilla counterpart** —
   `crates/pumpkin-core/src/entity/player.rs:3229` `MAX_PACKETS_PER_TICK = 64`. Vanilla drains the
   whole queued batch each tick. A burst above 64 is delayed to later ticks (latency, not loss).
   Fix (S): raise/drop the cap and bound by bytes handled instead.
5. **Unknown packet id logs at `warn` per packet** — `mod.rs:1370-1372`. Vanilla has no such
   branch (the codec rejects unknown ids), so a client can spam the log. Fix (XS): `debug!`.

Verified equal on this path: `suspend_flushing`/`resume_flushing`/`flush_channel`
(`ServerCommonPacketListenerImpl`), `close`/`kick`/`try_kick` ordering, packet-sequence block
acknowledgement, `SClientTickEnd` zeroing movement when no movement packet arrived
(`handleClientTickEnd`), brand channel read bound of 256 (`readUtf(256)`), `register`/`unregister`
splitting on `\0`, no disconnect packet in handshake/status (`serialize_disconnect` -> None),
`PacketReceivedEvent`/`PacketSentEvent` fire points (server thread / writer task).
