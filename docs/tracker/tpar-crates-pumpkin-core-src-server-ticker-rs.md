# Parity rows for crates/pumpkin-core/src/server/ticker.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-server-tic** [partial/OWN-OPEN] own audit: 4 open, 2 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-server-tic.md) docs/tracker/taudit-crates-pumpkin-core-src-server-tic.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)


## Result (third pass, final)

All four audit rows are resolved in source; no further code edit or new test was needed.
The only decompiled server available is Folia/Paper
(`.vendor/vanilla-decompiled/net/minecraft/server/MinecraftServer.java`), which removed the
vanilla "Can't keep up" warning and comments out the region-threaded tickCount increment;
parity for those two points is taken from the surrounding Folia comments and the audit's
Paper reasoning.

- Row 1 (tick-count freeze semantics): FIXED. `server/mod.rs:2332 advances_tick_count`
  gates the increment on `runs_normally() || is_sprinting()`, and ticker.rs:140 calls it at
  the top of the tick so frozen ticks do not advance `tick_count` while ring writes stay
  unconditional. Folia `MinecraftServer.java:1705` (`// this.tickCount++;`) and the
  tickCount-semantics comment at :344-350 confirm the region-threaded removal; the gate
  follows the audit's prescribed design. Covered by `tick_count_advances_only_when_game_runs`.
- Row 2 (overload-warning gate): FIXED. `ticker.rs:232 apply_overload_skip` takes
  `runs_normally`, gates the skip/warn branch on it (`ticker.rs:258`), and `run()` passes
  `manager.runs_normally()` (`ticker.rs:121`); the `this_tick_nanos <= 0` early return covers
  a zero/negative tick length. Folia removed the "Can't keep up" warning (no occurrence in
  `MinecraftServer.java`), so the optional `i > 0` sub-condition is not observable here; the
  core gate is in place and covered by `frozen_overload_does_not_skip_or_warn`.
- Row 3 (1-based tick event numbering): FIXED. With the increment at the top of the tick,
  both `ServerTickStartEvent` (`ticker.rs:216`) and `ServerTickEndEvent` (`ticker.rs:186`)
  read the already-incremented `tick_count`, giving Paper's 1-based numbering; the plugin
  API docs in `server_tick_start.rs`/`server_tick_end.rs` document 1-based ticks. Folia
  `MinecraftServer.java:1388` (`// ++MinecraftServer.currentTick;`) shows the moved increment.
- Row 4 (1m/5m/15m TPS rings): FIXED. `server/mod.rs` keeps `tick_times_1m/5m/15m_nanos`
  rings fed from `update_tick_times` and exposes `get_recent_tps`/`get_tps_1m/5m/15m` through
  `ring_tps` (test `ring_tps_uses_filled_samples_and_caps_at_target`). Folia
  `MinecraftServer.java:333-335` and `getTPS` at :412-414 confirm the three windows.

Verification was read-only: no source edit and no new test was needed this pass. Rustfmt,
cargo, clippy and nextest were not run (forbidden for this worker); the orchestrator's batch
check covers the code.
