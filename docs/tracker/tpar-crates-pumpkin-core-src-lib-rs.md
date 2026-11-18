# Parity rows for crates/pumpkin-core/src/lib.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-li-03c9dd** [partial/OWN-OPEN] own audit: 7 open, 6 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-li-03c9dd.md) docs/tracker/taudit-crates-pumpkin-core-src-li-03c9dd.md
- **own/taudit-crates-pumpkin-core-src-se-870386** [partial/OWN-OPEN] own audit: 3 open, 4 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-se-870386.md) docs/tracker/taudit-crates-pumpkin-core-src-se-870386.md

## Result
Third pass, final. Both own-audit rows are CLOSED. No source files changed this
pass: every remaining row was already fixed in earlier passes or resolves as a
deliberate vanilla difference or missing subsystem.

- own/taudit-crates-pumpkin-core-src-se-870386 (server/ticker.rs): CLOSED. All rows
  are FIXED/VERIFIED in that doc final Result (tick_count-at-start, crash report,
  overload gate, pause-when-empty, sprint gate, game-test gate); no STILL OPEN rows.
- own/taudit-crates-pumpkin-core-src-li-03c9dd (lib.rs): CLOSED. Rows 2, 3, 5, 7 are
  FIXED in the working tree (connection throttle, scrub_address, console EOF, bedrock
  client id key). Remaining rows:
  - Row 1 (early full-server reject at accept loop): VERIFIED. Vanilla
    ServerConnectionListener.startTcpServerListener/acceptConnections never check
    player count before the handshake; the full check runs in the login listener
    (PlayerList.canBypassFullServerLogin, .vendor/vanilla-decompiled/net/minecraft/
    server/players/PlayerList.java:658). Pumpkin enforces only in add_player
    join_rejection, matching vanilla.
  - Row 4 (player stats persistence): BLOCKED-DATA. Requires a whole missing subsystem:
    a player-statistics persistence manager writing stats/<uuid>.json on save (vanilla
    PlayerList#save -> ServerStatsCounter#save, PlayerList.java:508-515). Pumpkin has
    only in-memory increment_stat, no stats manager or on-disk serializer.
  - Row 6 (fallback kick overwrites rejection, server/mod.rs): VERIFIED. The fallback
    already uses the translatable multiplayer.disconnect.server_shutdown key
    (server/mod.rs:1346-1349), and join_rejection sends the specific translatable
    reason (server_full, duplicate_login) at server/mod.rs:1263-1265 before any fallback,
    so no plain-text overwrite occurs (vanilla PlayerList.java:658 uses
    multiplayer.disconnect.server_full).

No code changes this pass. No compile/test run (unit rules). Risk: none; row 4 is a
genuine missing subsystem, not a generated-data gap.


## Result (final pass, authoritative)

All rows re-verified against decompiled vanilla at .vendor/vanilla-decompiled/net/minecraft/.
No source change was needed in this pass: crates/pumpkin-core/src/server/ticker.rs
already matches the reference at every site, so no rustfmt run was required.
Terminal state of every row, in order:

own/taudit-crates-pumpkin-core-src-se-870386 (server/ticker.rs):

- FIXED tick_count-at-start: vanilla runs "this.tickCount++" at the top of tickServer
  before tickRateManager.tick() (.vendor/vanilla-decompiled/net/minecraft/server/
  MinecraftServer.java:1705-1706). Pumpkin advances server.tick_count at the top of
  Ticker::run under advances_tick_count (ticker.rs:140-142); the +1 patch in
  sync_game_time (mod.rs:2068) and the increment in update_tick_times (mod.rs:2083)
  are gone.
- VERIFIED try/catch crash report: vanilla wraps tick work in catch (Throwable),
  builds a CrashReport and stops (MinecraftServer.java:1707-1713). Pumpkin's global
  panic hook (crates/pumpkin/src/main.rs) sets CRASH_REPORT, stops the server, saves
  crash-reports/crash-*.txt and sets exit code 1; same observable result.
- VERIFIED pause-when-empty: vanilla emptyTicks/reset, seconds*20 threshold, one-shot
  LOGGER.info("Server empty for {} seconds, pausing") + autoSave() and connection
  ticking while paused (MinecraftServer.java:1608-1639, log at 1620). Pumpkin
  EmptyTickClock::observe + info! + autosave_if_due(true) (ticker.rs:105-163) matches.
- VERIFIED sprint gate (ticker.rs:95, is_sprinting() && runs_normally()) mirrors
  vanilla isSprinting() && checkShouldSprintThisTick().
- VERIFIED game-test gate (ticker.rs:174-182) runs after server.tick() under
  runs_normally() next to GameTestTicker.SINGLETON.tick() (MinecraftServer.java).
- VERIFIED overload gate (ticker.rs:258-263) keeps classic vanilla ">="; the supplied
  Folia tree uses tickSchedule, so the classic gate is represented by the constant
  gate and the [1, 6] test.

own/taudit-crates-pumpkin-core-src-li-03c9dd (lib.rs):

- Row 1 early full-server reject: VERIFIED. ServerConnectionListener.java
  (.vendor/vanilla-decompiled/net/minecraft/server/network/) has no pre-handshake
  player-count check; the check runs in the login listener
  (.vendor/vanilla-decompiled/net/minecraft/server/players/PlayerList.java:658,
  canBypassFullServerLogin). Pumpkin enforces only in add_player join_rejection.
- Row 2 per-IP connection throttle: FIXED (connection_throttled +
  advanced_config.networking.java.connection_throttle, Spigot settings.connection-throttle).
- Row 3 IP masking shape: FIXED (scrub_address keeps . and :, Paper scrub-ips shape).
- Row 4 player stats persistence: BLOCKED-DATA. Missing whole subsystem: no
  ServerStatsCounter / stats/<uuid>.json persistence manager exists on save (vanilla
  PlayerList.java:508-515); Pumpkin has only in-memory increment_stat and no stats
  serializer, so no generated field can close this row.
- Row 5 console EOF warning: VERIFIED. A final newline-less line is accepted as a
  command and a zero-byte read is EOF, matching vanilla BufferedReader.
- Row 6 fallback kick overwrite: VERIFIED. join_rejection sends the specific
  translatable reason (multiplayer.disconnect.server_full) before any fallback, which
  uses multiplayer.disconnect.server_shutdown
  (.vendor/vanilla-decompiled/net/minecraft/server/players/PlayerList.java:658).
- Row 7 bedrock client id key: FIXED (keyed by monotonic u64 client_id).

No STILL OPEN rows remain. No generated pumpkin-data field, WIT change or
cross-crate refactor is outstanding. No compile/test run (unit rules).
Risk: none; row 4 is a genuine missing subsystem, not a generated-data gap.
