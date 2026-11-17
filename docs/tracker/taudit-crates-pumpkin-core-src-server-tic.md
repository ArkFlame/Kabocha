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
