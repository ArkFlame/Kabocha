# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-server-ticker-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

All four rows are closed. Per-row results are in the Result section below.

## Result
All four audit rows are resolved in the current tree. Verification was read-only
(no source edit or new test needed this pass) against the only decompiled server
available here, Folia/Paper
`.vendor/vanilla-decompiled/net/minecraft/server/MinecraftServer.java`.

- Row 1 (tick-count freeze semantics): FIXED / VERIFIED. `ticker.rs:140` gates the
  increment with `super::advances_tick_count(manager.runs_normally(), manager.is_sprinting())`
  and `server/mod.rs:2332` defines the `runs_normally || is_sprinting` helper, so
  frozen non-stepping ticks do not advance `tick_count` while the ring writes in
  `update_tick_times` stay unconditional; `sync_game_time` reads `tick_count % 20`.
  Vanilla paths: `MinecraftServer.java:1705` (`// this.tickCount++;`) and the
  tickCount-semantics comment at `:344-351`. Covered by
  `tick_count_advances_only_when_game_runs`.
- Row 2 (overload-warning gate): FIXED / VERIFIED. `ticker.rs:232 apply_overload_skip`
  takes `runs_normally`, gates the skip/warn branch at `:258`, and `run()` passes
  `manager.runs_normally()` at `:121`; the `this_tick_nanos <= 0` early return at
  `:244` covers a zero/negative tick length. Vanilla gates the warning on
  `runsNormally()`; Folia removed the "Can't keep up" warning (no occurrence in
  `MinecraftServer.java`), keeping the overload constants at `:197-200`. Covered by
  `frozen_overload_does_not_skip_or_warn`.
- Row 3 (1-based tick event numbering): FIXED / VERIFIED. The increment at
  `ticker.rs:140` runs before both events, so the `tick_count.load` reads at `:186`
  (end) and `:216` (start) report Paper's 1-based tick. Core and plugin API docs
  (`server_tick_*.rs`) document 1-based ticks. Vanilla paths:
  `MinecraftServer.java:1657` (`ServerTickStartEvent`) and `:1753`
  (`ServerTickEndEvent`), both using the 1-based region current tick.
- Row 4 (1m/5m/15m TPS rings): FIXED / VERIFIED. `server/mod.rs` keeps
  `tick_times_1m/5m/15m_nanos` rings (`:171-173`, initialized `:428-431`, written
  `:2094-2116`) and exposes `get_recent_tps` and `get_tps_1m/5m/15m`
  (`:2184-2214`) through `ring_tps` (`:2295`). Vanilla paths:
  `MinecraftServer.java:333-335` (tickTimes1m/5m/15m) and `computeTPS`/`getTPS` at
  `:409-415`. Covered by `ring_tps_uses_filled_samples_and_caps_at_target`.

No BLOCKED-DATA rows: no generated `pumpkin-data` field or missing subsystem is
required. Rustfmt/cargo/clippy/nextest were not run (forbidden for this worker);
the orchestrator's batch check covers the code.
