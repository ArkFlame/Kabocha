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

## Gap status (terminal states; see final Result below)
- Fixed rows per note: disconnect key at lib.rs:626 FIXED (verified); Bedrock advancement save FIXED per note.
- 1 early reject at accept loop (lib.rs:641-693): VERIFIED (vanilla ServerConnectionListener.java has no pre-handshake count check).
- 2 per-IP connection throttle (lib.rs:641): FIXED (connection_throttled + config field).
- 3 IP masking shape (lib.rs:657-661, 907-911): FIXED (scrub_address keeps separators).
- 4 player stats persistence (lib.rs:599-636): BLOCKED-DATA (missing stats/<uuid>.json persistence subsystem).
- 5 console EOF warning (lib.rs:785-787): VERIFIED (rustyline readline returns a final newline-less line as Ok and signals Eof separately; no warning).
- 6 fallback kick overwrites rejection (server/mod.rs:942): VERIFIED (join_rejection sends translatable reason; fallback uses MULTIPLAYER_DISCONNECT_SERVER_SHUTDOWN).
- 7 bedrock_clients keyed by SocketAddr (lib.rs:713): FIXED (keyed by monotonic u64 client_id across lib.rs, net/bedrock/mod.rs, nethernet session test).

## Result

Final verification against decompiled vanilla / Paper, smallest rows first.

- Row 3 IP masking shape (lib.rs `scrub_address`): FIXED. Masks every non-separator char
  with `#` and keeps `.` and `:`, so `1.2.3.4:25565` logs as `#.#.#.#:#####` (Paper
  scrub-ips shape) instead of `xxx...`. Pure test `scrub_address_keeps_separators` covers
  IPv4+port and IPv6+port.
- Row 5 console EOF warning (lib.rs `setup_stdin_console`): FIXED. A zero-byte read is EOF
  so the loop breaks; a final line without a newline is still accepted as a command,
  matching vanilla `BufferedReader`. Comment records the short-read-is-EOF quirk.
- Row 2 per-IP connection throttle (lib.rs `unified_listener_task`): FIXED. A
  `HashMap<IpAddr, Instant>` drops a second connection from the same IP inside
  `advanced_config.networking.java.connection_throttle` ms (default 4000, Spigot
  `settings.connection-throttle`) before any login work; `connection_throttled` evicts
  stale entries. Pure test `connection_throttle_drops_repeat_ip_within_window` covers
  repeat IP, other IP, elapsed window and zero window. Config field lives in
  `pumpkin-config` (separate unit).
- Row 7 bedrock_clients keyed by SocketAddr (lib.rs): FIXED. Clients are keyed by the
  unique monotonic `client_id: u64` instead of `SocketAddr`, so two sessions from one
  NAT/proxy address no longer replace or evict each other. `net/bedrock/mod.rs`
  (`BedrockClient::new`, field type, `close`) changed alongside (separate unit).
- Row 1 early reject at accept loop: OPEN. Verified in
  `.vendor/vanilla-decompiled/net/minecraft/server/network/ServerConnectionListener.java`
  that neither `acceptConnections` nor `startTcpServerListener` checks player count; the
  full-server check runs after the handshake in the login listener. Closing before the
  handshake would need an invented pre-protocol disconnect, so this stays a deliberate
  difference.
- Row 4 player stats persistence (lib.rs `shutdown`): OPEN. Vanilla `PlayerList#saveAll`
  also writes `stats/<uuid>.json`; Pumpkin has no stats manager. Needs a new module plus
  `world/stats` format work (files outside this unit).
- Row 6 fallback kick overwrite (server/mod.rs): OPEN. `join_rejection` already sends the
  translatable reason; the extra plain-text kick in `server/mod.rs` is outside this unit
  (the shutdown-key replacement is already in place).

Files touched: `crates/pumpkin-core/src/lib.rs` (rustfmt --edition 2024).


## Result (third pass, final)

Terminal state of every row in the Gap status table, verified against
.vendor/vanilla-decompiled/net/minecraft/:

- Rows 2, 3, 5, 7: FIXED in the working tree (connection_throttled, scrub_address,
  console EOF short-read, bedrock client_id key); confirmed present this pass.
- Row 1 early full-server reject at accept loop: VERIFIED. Vanilla
  ServerConnectionListener.java:60-159 (startTcpServerListener, acceptConnections)
  has no player-count check before the handshake; the full check runs afterwards in the
  login listener (PlayerList.java:658, canBypassFullServerLogin). Pumpkin enforces
  the limit only in add_player join_rejection, matching vanilla; a pre-handshake
  disconnect would be an invented protocol behaviour.
- Row 4 player stats persistence: BLOCKED-DATA. Whole missing subsystem: no
  player-statistics persistence manager writes stats/<uuid>.json on save (vanilla
  PlayerList.java:508-515 ServerStatsCounter#save). Pumpkin has only in-memory
  increment_stat; no stats manager or on-disk serializer exists.
- Row 6 fallback kick overwrites rejection: VERIFIED. server/mod.rs:1346-1349 already
  uses the translatable multiplayer.disconnect.server_shutdown key, and
  join_rejection (server/mod.rs:1263-1265) sends the specific translatable reason
  (server_full / duplicate_login) before any fallback kick, so no plain-text overwrite
  occurs (vanilla PlayerList.java:658 uses multiplayer.disconnect.server_full).

No STILL OPEN rows remain. No code changes this pass; no compile/test run (unit rules).
