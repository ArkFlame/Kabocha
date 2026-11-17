# audit / crates/pumpkin-core/src/lib.rs (DEEP AUDIT)

Target: current master (1.21.x, protocol per CURRENT_MC_VERSION). This file is the server
bootstrap: logger, process flags, Java/Bedrock connection lifecycle, console, listener loop,
shutdown. No TODO/FIXME/todo! markers exist in the file; the two tracker hits were the
vanilla_data: VanillaData field/parameter, not markers.

Reference: MinecraftServer#stopServer (1.21), ServerConnectionListener#startTcpServerListener,
Paper ServerGamePacketListenerImpl/PlayerList#remove, Spigot settings.connection-throttle.

## Fixed in this pass

- lib.rs:619-625 (was TextComponent::text("Server stopped")) - vanilla
  MinecraftServer#stopServer disconnects players with
  Component.translatable("multiplayer.disconnect.server_shutdown") ("Server closed"). Now
  sends the generated translation key via translate_cross, matching every other disconnect
  reason in the tree.
- lib.rs:771-773 - Bedrock disconnect saved player data but not advancements, unlike the
  Java path (lib.rs:271). Vanilla persists advancements per player; both platforms now call
  advancement_manager.save_player on leave, so a crash after a Bedrock quit no longer loses
  progress.

## Remaining gaps

| Location | Vanilla / Paper | Gap | Fix | Size |
|---|---|---|---|---|
| lib.rs:641-693 (accept loop) | Paper ServerConnectionListener checks playerList.getPlayerCount() >= maxPlayers before reading the handshake and disconnects immediately | Limit is only enforced inside add_player after the full login handshake, so a full server still spends login work on every join | Pass the count check into the accept branch and close early | S |
| lib.rs:641 | Spigot settings.connection-throttle rejects a second connection from the same IP within 4000 ms | No per-IP throttle; only a per-connection PacketRateLimiter | Config field + HashMap<IpAddr, Instant> cleaned on a tick | M |
| lib.rs:657-661 + lib.rs:907-911 | Spigot/Paper scrub-ips-style masking keeps separators and masks the rest (1.2.3.4 -> #.#.#.#) | Replaces every non . / : char with x, producing xxx... masks; shape differs from Paper | Mask digits/letters but keep . and : | S |
| lib.rs:599-636 (shutdown) | Vanilla PlayerList#saveAll also writes stats/<uuid>.json | Pumpkin has no player-stat persistence at all (only NBT + advancements), so stats are lost on every quit | New stats manager writing world/stats/<uuid>.json; wire into handle_player_leave and save_all_players | L |
| lib.rs:785-787 | Console reader: vanilla BufferedReader treats a final line without newline as a valid command | Emits a warning at EOF for the last line only | Only warn when the line is not the last read | S |
| server/mod.rs:942 (reached from this file flow) | PlayerList#placeNewPlayer failure disconnects with the reason that was rejected (server_full, not_whitelisted) | Fallback kick overwrites that with plain text "Server stopped" | Use the translatable shutdown/full message or skip the extra kick since join_rejection already sent one | S |
| lib.rs:713 | NetherNet sessions are keyed per remote address by the transport | bedrock_clients map keyed by SocketAddr silently replaces an older session from the same address (NAT/proxy) and then removes the map entry the old client still owns | Key by client id instead | S |

Notes: adjust_file_descriptor_limit (1,048,576 cap with 65,536 fallback) has no vanilla
counterpart and is intentional. Console Ctrl-C (graceful then force-exit) matches vanilla
stopServer plus second-signal behaviour.

## Gap status
- Fixed rows per note: disconnect key at lib.rs:626 FIXED (verified); Bedrock advancement save FIXED per note.
- 1 early reject at accept loop (lib.rs:641-693): STILL OPEN, S.
- 2 per-IP connection throttle (lib.rs:641): STILL OPEN, M.
- 3 IP masking shape (lib.rs:657-661, 907-911): STILL OPEN, S.
- 4 player stats persistence (lib.rs:599-636): STILL OPEN, L.
- 5 console EOF warning (lib.rs:785-787): STILL OPEN, S.
- 6 fallback kick overwrites rejection (server/mod.rs:942): FIXED in part, server/mod.rs:1268 uses the shutdown key; the overwrite itself STILL OPEN, S.
- 7 bedrock_clients keyed by SocketAddr (lib.rs:713): STILL OPEN, S.

## Result

- Row 3 IP masking shape (lib.rs:925-931): FIXED. `scrub_address` now masks every non
  separator char with `#` and keeps `.` and `:`, so `1.2.3.4:25565` logs as
  `#.#.#.#:#####` (Paper scrub-ips shape) instead of `x.x.x.x:xxxxx`. Added the pure unit
  test `scrub_address_keeps_separators`.
- Row 5 console EOF warning (lib.rs:799-801): FIXED. The final-line-without-newline warning
  is gone; vanilla's `BufferedReader` treats such a line as a valid command, so the comment
  now records that a short read is EOF, not a malformed command.
- Row 1 early reject at accept loop: OPEN. Neither vanilla `ServerConnectionListener` nor the
  other vendored sources reject before the handshake; an early close would have to invent a
  login-phase disconnect before the protocol state exists, so it is left as a deliberate
  difference rather than a blind port.
- Row 2 per-IP connection throttle: OPEN. Needs a new `settings.connection-throttle` field in
  `crates/pumpkin-config` plus a tick-cleaned map, i.e. files outside this unit.
- Row 4 player stats persistence: OPEN. New manager module plus `world/stats` format work,
  well beyond this unit's scope.
- Row 6 fallback kick overwrite: OPEN. The remaining change is in `server/mod.rs`, outside
  this unit (the shutdown key replacement noted earlier is already in place).
- Row 7 bedrock_clients keyed by SocketAddr: OPEN. Keying by client id changes
  `net/bedrock/mod.rs` (field type, `new` signature, `close`) together with this file, so it
  is outside this unit.

Files touched: `crates/pumpkin-core/src/lib.rs` (rustfmt --edition 2024, exit 0).

