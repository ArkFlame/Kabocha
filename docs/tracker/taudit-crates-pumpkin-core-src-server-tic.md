# Audit: crates/pumpkin-core/src/server/ticker.rs

Deep audit of the tick loop against vanilla `MinecraftServer.runServer` / `waitUntilNextTick`
(1.20.3+ TickRateManager) and Paper.

No `TODO`/`FIXME` markers in the file; the 2 tracker markers are the "Vanilla ..." doc comments
on the overload constants and on the skip helper.

## Fixed

- `ticker.rs:170-186` Overload warning interval was measured from the scheduled deadline
  (`next_tick_nanos - last_overload_warning_nanos`) and stamped with the jumped deadline.
  Vanilla measures `l - this.lastOverloadWarningNanos` (wall clock) and sets
  `this.lastOverloadWarningNanos = l`. Under sustained overload the deadline advances only one
  tick per iteration while wall time advances by the whole (long) iteration, so the old code
  suppressed re-warnings far longer than vanilla (e.g. 3s ticks: 600s instead of 15s).
  Fixed and covered by `sustained_overload_warns_again_after_wall_clock_interval`.

## Remaining gaps

1. `ticker.rs:128` + `server/mod.rs:1590` `update_tick_times` increments `tick_count`
   unconditionally, including frozen ticks. Vanilla increments `MinecraftServer.tickCount` only
   when `tickRateManager.runsNormally()` (inside `tickServer`). Knock-on effects:
   `sync_game_time` `% 20`, `scheduled_functions.tick` index, autosave cadence, and the tick
   numbers carried by `ServerTickStartEvent`/`ServerTickEndEvent` all advance while frozen.
   Fix: move the increment into the `runs_normally() || is_sprinting()` branch and keep the
   tick-times ring update unconditional. Size: M (many call sites read `tick_count`).
2. `ticker.rs:174` The overload warning has no `tickRateManager.runsNormally()` guard. Vanilla
   1.20.3+ gates the warning on it (a frozen server doing no game work should not report
   overload). Needs a decompiled 1.20.5+ `MinecraftServer` to confirm `i > 0` is also part of
   the gate before writing it. Size: S.
3. `ticker.rs:90,120` `ServerTickStartEvent`/`ServerTickEndEvent` carry 0-indexed tick numbers
   (documented as such in `plugin/api/events/server/server_tick_*.rs`). Paper passes
   `this.tickCount + 1` to `ServerTickStartEvent` and `this.tickCount` to `ServerTickEndEvent`,
   i.e. 1-based tick start. Plugin-visible off-by-one; changing it is an API semantic change.
   Size: S.
4. `ticker.rs:128` + `server/mod.rs:1631-1655` Only one 100-tick ring (`tick_times_nanos`) feeds
   both MSPT and TPS. Vanilla keeps `tickTimes5s` (100), `tickTimes10s` (200), `tickTimes60s`
   (1200) and Bukkit derives 1m/5m/15m TPS from those; `/tps` and plugins reading
   `get_tps()` therefore cannot match Paper. Fix: add the three rings and expose 1m/5m/15m.
   Size: M.

## Gap status
- 1 STILL OPEN: tick-count freeze semantics, M, many call sites.
- 2 STILL OPEN: overload-warning gate needs decompiled 1.20.5+ server, S.
- 3 STILL OPEN: plugin-visible event numbering is an API semantic change, S.
- 4 STILL OPEN: 1m/5m/15m rings, M.

## Result

Re-verified on the resumed pass against the committed tree (HEAD) and the available
decomposed vanilla. Every ticker.rs-local fix is already landed; only the two rows that
need server/mod.rs remain open.

- Row 1 (tick-count freeze semantics): LEFT OPEN. The increment lives in
  `update_tick_times` at `server/mod.rs:1590`, a file outside this unit. Moving it into the
  `runs_normally() || is_sprinting()` branch cannot be done from ticker.rs.
- Row 2 (overload-warning gate): FIXED (already in HEAD). `apply_overload_skip` takes
  `runs_normally: bool`, the skip/warn branch is gated on it, `run()` passes
  `manager.runs_normally()`, and the pure unit test `frozen_overload_does_not_skip_or_warn`
  covers it. The audit's optional `i > 0` sub-condition remains unverified: the only
  decompiled server here is Folia, which removed the vanilla "Can't keep up" warning.
  The `this_tick_nanos <= 0` early return already covers a zero/negative tick length.
- Row 3 (1-based tick event numbering): FIXED (already in HEAD, commit `98981fcbf`).
  `tick_event_number` makes both start and end events report Paper's 1-based tick, and the
  plugin API event docs (`server_tick_*.rs`) now document 1-based ticks too. The earlier
  "LEFT OPEN / API semantic change" note in the previous Result section was stale.
- Row 4 (1m/5m/15m TPS rings): LEFT OPEN. Needs three tick-time rings and new getters in
  `server/mod.rs:1631-1655`, a file outside this unit.

Verification: `rustfmt --edition 2024 crates/pumpkin-core/src/server/ticker.rs` ran cleanly
(the file was already formatted; no new source edit this pass). Cargo/clippy/nextest were
not run (forbidden for this worker). Note: an uncommitted `warn!` -> `info!` change to the
"Server empty for ... seconds, pausing" log line is present in ticker.rs; it is unrelated to
this audit and was left untouched.

## Result (re-verification pass, current tree)

The prior Result section was stale. Re-checked every row against the current tree and the
available decompiled server (Folia/Paper `MinecraftServer.java`); all four rows are now
resolved. No ticker.rs source edit was needed this pass.

- Row 1 (tick-count freeze semantics): RESOLVED in `server/mod.rs` (sibling worker).
  `update_tick_times` now gates the increment via `advances_tick_count(runs_normally,
  is_sprinting)` so frozen ticks do not advance `tick_count`, while the ring writes stay
  unconditional; `sync_game_time` uses `tick_count + 1`. `update_tick_times`'s signature is
  unchanged, so ticker.rs:190 needed no edit. Folia comments out `this.tickCount++`
  (MinecraftServer.java:1705), so exact vanilla placement cannot be observed here, but the
  helper matches the audit's prescribed design.
- Row 2 (overload-warning gate): RESOLVED in ticker.rs (HEAD). `apply_overload_skip` takes
  `runs_normally`, the skip/warn branch is gated on it, `run()` passes
  `manager.runs_normally()`, and `frozen_overload_does_not_skip_or_warn` covers it. The
  optional `i > 0` sub-condition is unverifiable from the Folia decompile (the "Can't keep up"
  warning was removed), but the `this_tick_nanos <= 0` early return already covers a
  zero/negative tick length.
- Row 3 (1-based tick event numbering): RESOLVED in ticker.rs (HEAD). `tick_event_number`
  makes both start and end events report Paper's 1-based tick; `tick_events_report_one_based_tick_number`
  covers it and the plugin API event docs document 1-based ticks.
- Row 4 (1m/5m/15m TPS rings): RESOLVED in `server/mod.rs` (sibling worker). It now keeps
  `tick_times_1m/5m/15m_nanos` rings fed from `update_tick_times`, plus `get_recent_tps`,
  `get_tps_1m/5m/15m` and the `ring_tps` helper/test. Paper's `tickTimes1m/5m/15m`
  (MinecraftServer.java:333-335) confirms the three windows. No ticker.rs change needed.

Verification: source read-only this pass (no edits). Rustfmt/cargo/clippy/nextest were not run
(forbidden for this worker); the orchestrator's batch check covers the sibling-worker
server/mod.rs changes.
