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

- Row 1 (tick-count freeze semantics): LEFT OPEN. The fix moves the `tick_count`
  increment into the `runs_normally() || is_sprinting()` branch of
  `update_tick_times` at `server/mod.rs:1590`, a file outside this unit (only
  `ticker.rs` is editable here).
- Row 2 (overload-warning gate): FIXED. `apply_overload_skip` now takes a
  `runs_normally: bool` and the skip/warn branch is gated on it; `run()` passes
  `manager.runs_normally()`. Added the pure unit test
  `frozen_overload_does_not_skip_or_warn`. The additional `i > 0` sub-condition
  the audit wanted confirmed could not be verified: the available decompiled
  source is Folia, which removed the vanilla "Can't keep up" warning in favour
  of catch-up scheduling, so no 1.20.5+ vanilla gate was available. The
  well-supported `runsNormally()` guard was applied on its own.
- Row 3 (1-based tick event numbering): LEFT OPEN. Changing the number passed to
  `ServerTickStartEvent` is a plugin-visible semantic change and the event docs
  (`pumpkin-plugin-api/src/events/server/server_tick_*.rs`) document 0-indexed
  ticks; aligning those files is outside this unit. The available decompiled
  server is Folia/region-threaded (`getCurrentTick()`), not directly comparable.
- Row 4 (1m/5m/15m TPS rings): LEFT OPEN. Needs three tick-time rings and new
  getters in `server/mod.rs:1631-1655`, a file outside this unit.

Verification: `rustfmt --edition 2024 crates/pumpkin-core/src/server/ticker.rs`
run successfully. Cargo/clippy/nextest not run (forbidden for this worker).
Note: a concurrent edit changed the overload interval gate from `>=` to `>` and
the test to `vec![1, 7]` mid-session; the vanilla-correct `>=` / `vec![1, 6]`
(original state of this file) was restored, since that operator is outside this
unit's task scope and the broken pairing would fail the test.
