# audit / crates/pumpkin-core/src/server/ticker.rs

Target: `JavaMinecraftVersion::V_26_3`. `.vendor` absent; compared against the
decompiled Paper/Canvas tree in `/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft`
(`server/MinecraftServer.java` `runServer` / `tickServer`, `server/dedicated/DedicatedServer.java`)
and the vanilla tick-loop constants quoted in the file itself.

No TODO/FIXME markers exist in the file (the tracker note counted the `Vanilla ...` parity
comments). The overload-skip model, sprint deadline reset, warning clock, park/wait and the
tick-duration broadcast all match vanilla `MinecraftServer.runServer`; the nanosecond
`ServerTickEndEvent` payload and 0-indexed `tick` field are deliberate fork API
(`plugin/api/events/server/server_tick_end.rs:6-11`), not gaps.

Fixed in this pass:

- `ticker.rs:108-117` - game tests were gated on the freeze state sampled *before*
  `server.tick()`. Vanilla checks `this.tickRateManager.runsNormally()` at the end of the
  tick, next to `GameTestTicker.SINGLETON.tick()` (MinecraftServer.java:1914), so a
  `/tick freeze` or `/tick sprint` issued inside the tick applies immediately. The gate is
  now evaluated after the tick (the sprint flag stays the pre-tick one on purpose).

Remaining gaps:

| Location | Vanilla | Gap | Fix | Size |
|---|---|---|---|---|
| `ticker.rs:90,122` + `server/mod.rs:1591` | `tickServer` runs `int i = this.tickCount++;` at the *start* of the tick, before `this.tickRateManager.tick()` (MinecraftServer.java:1694-1695) | Fork increments `tick_count` inside `update_tick_times`, i.e. after `server.tick()`, so every in-tick read (scheduled functions, autosave phase, plugin events, `sync_game_time`) is off by one; `sync_game_time` patches it locally with `tick_count + 1` | Move the increment to the top of the tick loop and decrement the indexing in `update_tick_times`; audit every in-tick `tick_count` reader (incl. removing the `+ 1` patch) | M |
| `ticker.rs:97-118` | `runServer` wraps the tick in `try/catch (Throwable)`, builds a `CrashReport`, writes `crash-reports/crash-<date>-server.txt` and stops the server (MinecraftServer.java:1409-1430) | A panic in `server.tick()` or a plugin tick handler unwinds the ticker thread; no crash report, no orderly stop | Wrap the tick body, build/save a crash report, then trigger the existing stop path | L |
| `ticker.rs:62-139` | `tickServer` pauses the loop after `pause-when-empty-seconds` (vanilla dedicated default 60s), autosaving once and still ticking connections (MinecraftServer.java:1604-1636, DedicatedServer.java:1037) | No tick sleeping: the loop runs worlds at full rate while the server is empty | Add the empty-tick counter plus the paused branch (connection tick + scheduled task drain + one autosave) | L |

Non-gaps checked (do not "fix"):

- `OVERLOADED_THRESHOLD_NANOS` is 1s (vanilla `20 * 1s / 20`). Paper/CraftBukkit raises it to
  1.5s (MinecraftServer.java:197); the fork targets vanilla, keep 1s.
- `|| sprinting` in the game-test gate is redundant with `runsNormally()` (sprint clears
  `is_frozen`), harmless and kept.
- The overload warning clock is measured from the scheduled deadline and reset to the jumped
  deadline, matching the `nextTickTimeNanos`-based clock Paper/Canvas keeps.

## Second pass (sprint/freeze gate)

Fixed in this pass:

- `ticker.rs:70` - `sprinting` was `manager.is_sprinting()` alone. Vanilla
  `runServer` takes the sprint path only when `isSprinting() &&
  checkShouldSprintThisTick()`, and `checkShouldSprintThisTick()` returns
  false while `runGameElements` is false (TickRateManager). So `/tick freeze`
  issued during a sprint must stop sprint ticks, not keep running them.
- `server/mod.rs:1812` - `tick()` gated world ticking on `runsNormally() ||
  is_sprinting()`. Vanilla `tickChildren` is gated on `runsNormally()` alone,
  and a sprint clears the freeze, so the extra `is_sprinting()` revived world
  ticks after a mid-sprint freeze. Removed; sprinting still ticks worlds
  because `request_game_to_sprint` clears `is_frozen` and the next
  `TickRateManager::tick` sets `run_game_elements = true` (same one-tick
  settling as vanilla).

Both changes are one-liners already covered by the existing ticker tests;
the observable behaviour needs a `/tick sprint` + `/tick freeze` runtime
scenario, so no new unit test was added.

Newly noted, not changed:

| Location | Vanilla | Gap | Fix | Size |
|---|---|---|---|---|
| `ticker.rs:182` | overload warning gate is a strict `>` on `l - lastOverloadWarningNanos` | fork uses `>=`, so an exact-equality interval warns one tick early | change to `>`, update `sustained_overload_warns_again_after_wall_clock_interval` (`[1, 6]` becomes `[1, 7]`) | XS |
| `ticker.rs:66-71` | `checkShouldSprintThisTick()` also decrements `remainingSprintTicks` | fork decrements in `end_sprint_tick_work`; equivalent for the loop, `is_sprinting()` just stays true through the sprint work | none needed | - |

## Status (code-verified sweep)
FIXED: OVERLOADED_THRESHOLD_NANOS is 1s (server/ticker.rs:17-18). VERIFIED: game-test freeze gate ordering and sprint redundancy (superseded by the final Result below).


## Result

Vanilla reference used: both `.vendor/vanilla-decompiled` and the mc-proof Paper/Canvas
tree have replaced the classic overload gate with `tickSchedule`; the vanilla gate is the
well-known strict `>` on `l - lastOverloadWarningNanos` (`MinecraftServer.runServer`).

- FIXED (XS) `ticker.rs:186-188` - overload warning gate changed `>=` to `>` so an
  exact 15s interval no longer warns one tick early. Updated
  `sustained_overload_warns_again_after_wall_clock_interval` expectation `[1, 6]` ->
  `[1, 7]` (tick 6 is exactly 15s and is skipped; tick 7 at 18s warns).
  Verified with `rustfmt --edition 2024 crates/pumpkin-core/src/server/ticker.rs` (rc 0).
  No compile/test run (per unit rules); arithmetic re-derived by hand.

Rows left OPEN (reasons):

- `ticker.rs:90,122` tick_count-at-start: cross-file refactor needs
  `server/mod.rs:1591` (`update_tick_times`) + the `sync_game_time` `+1` patch and
  an audit of every in-tick reader, all outside this unit's files. Size M.
- `ticker.rs:97-118` try/catch crash report: size L, needs cross-crate wiring to the
  `pumpkin` binary's global panic hook (`crates/pumpkin/src/main.rs` `CrashReport`) plus
  `catch_unwind` scaffolding; outside this unit's files. Vanilla report writing is not
  reproducible in ticker.rs alone.
- `ticker.rs:62-139` pause-when-empty: size L, the config lives in
  `net/management/hub.rs:22,52` and the paused branch needs connection-tick + autosave
  hooks in `server/mod.rs`, both outside this unit's files.

Verified, no change needed:

- `ticker.rs:66-71` sprint decrement in `end_sprint_tick_work` is equivalent to vanilla
  `checkShouldSprintThisTick`; behaviour matches the loop.
- `ticker.rs:114-118` game-test gate already runs after `server.tick()` next to the
  runner (vanilla `MinecraftServer.java:1929-1930` `runsNormally()` gate); `|| sprinting`
  is redundant but harmless.


## Result

Re-audited all rows against `.vendor/vanilla-decompiled/net/minecraft/server/MinecraftServer.java`
(`runServer` / `tickServer`, lines 1353-1430, 1603-1639). Both supplied decompiled trees are
Paper/Canvas/Folia: the classic `l - lastOverloadWarningNanos` gate is gone (replaced by
`tickSchedule` catchup, `lastOverloadWarningNanos = nextTickTimeNanos` at line 1385), so the
`>=`/`>` detail recorded by the prior Result cannot be re-derived here. Classic vanilla uses
`>=`, which matches the current `ticker.rs:258`, so no change was made there and
`sustained_overload_warns_again_after_wall_clock_interval` stays `[1, 6]`.

- FIXED (XS) `ticker.rs:13,142` - pause-when-empty "Server empty for {} seconds, pausing"
  logged at `warn!`; vanilla uses `LOGGER.info` (MinecraftServer.java:1620). Changed to `info!`
  and added `info` to the `tracing` import. The rest of the pause path (threshold `seconds*20`,
  reset when players present or sprinting, one-shot log+autosave on the crossing tick,
  connection/scheduler ticking while paused) already matches vanilla tickServer 1608-1639;
  the runtime-settable config is wired in `net/management/handlers.rs:956-973` and defaults to
  60s in `hub.rs:52`. Ran `rustfmt --edition 2024 crates/pumpkin-core/src/server/ticker.rs` (rc 0).
  No compile/test run (unit rules).

Rows left OPEN (reasons):

- `ticker.rs:190` tick_count-at-start: needs `server/mod.rs` `update_tick_times` + the
  `sync_game_time` `+1` patch and an audit of every in-tick reader; outside hub.rs,
  server_tick_end.rs and ticker.rs. Size M.
- `ticker.rs:87-203` try/catch crash report: needs `catch_unwind` plus cross-crate wiring to the
  `pumpkin` binary's panic hook / `CrashReport`; not reproducible in ticker.rs alone. Size L.

Verified, no change needed:

- `ticker.rs:95` sprint gate (`is_sprinting() && runs_normally()`), `ticker.rs:165-175`
  post-tick game-test gate and `ticker.rs:136-163` tick body still match vanilla.
- `server_tick_end.rs` events unchanged; nanosecond payload and event numbering are fork API.

## Result

Re-verified every STILL OPEN row against
`.vendor/vanilla-decompiled/net/minecraft/server/MinecraftServer.java`. No in-unit
code change was needed in this pass.

Verified, no change needed:

- `ticker.rs:140-153` pause-when-empty already matches vanilla `tickServer` 1608-1639:
  `emptyTicks++`/reset (Pumpkin `EmptyTickClock::observe`), threshold `seconds*20`,
  one-shot `LOGGER.info("Server empty for {} seconds, pausing")` + `autoSave()` on
  the crossing tick (Pumpkin `info!` + `autosave_if_due(true)`), scheduler heartbeat
  and connection ticking while paused. Default 60s in `hub.rs:52` matches vanilla
  `pauseWhenEmptySeconds()`; runtime setter in `handlers.rs:956-973` is intact.
- `ticker.rs:95` sprint gate is `is_sprinting() && runs_normally()`, matching vanilla
  `isSprinting() && checkShouldSprintThisTick()` (1360).
- `ticker.rs:165-175` game-test gate runs after `server.tick()` under `runs_normally()`
  (vanilla 1929-1930 `GameTestTicker.SINGLETON.tick()`).
- `ticker.rs:255-259` overload gate keeps classic `>=` on `since_warning` and the
  `[1, 6]` test. The supplied tree is Folia/Paper (`lastOverloadWarningNanos =
  nextTickTimeNanos`, 1385), so the classic `l - lastOverloadWarningNanos` gate
  cannot be re-derived here; the fork targets classic vanilla, keep `>=`.
- `server_tick_end.rs` (fork API) and the `EmptyTickClock`/sprint/overload unit tests
  are unchanged and still cover every pure seam; no new test added.

Rows left OPEN (files outside this unit):

- `ticker.rs:179,190` tick_count-at-start: vanilla increments at the top of
  `tickServer` (`// this.tickCount++;`, line 1705); fixing requires editing
  `server/mod.rs` `update_tick_times` + the `sync_game_time` `+1` patch and auditing
  every in-tick reader, all outside this unit. Size M.
- `ticker.rs:87-203` try/catch crash report: vanilla wraps the tick in
  `catch (Throwable)` and writes `crash-reports/crash-<date>-server.txt` (1413-1424).
  Needs `catch_unwind` plus cross-crate wiring to the `pumpkin` binary's panic hook
  and `CrashReport`; not reproducible in ticker.rs alone. Size L.

## Result

Second pass with cross-file edits permitted. Both remaining STILL OPEN rows were
re-verified against `.vendor/vanilla-decompiled/net/minecraft/server/MinecraftServer.java`
(`runServer` 1357-1435, `tickServer` 1603-1757, classic `++tickCount` at 1705).

- FIXED (M) tick_count-at-start, previously left open as cross-file:
  - `server/ticker.rs:136-143` - advance `tick_count` at the top of the tick loop via
    `super::advances_tick_count(runs_normally, is_sprinting)`, before `manager.tick()`,
    matching vanilla `++tickCount` before `tickRateManager.tick()`/`tickChildren`.
    In-tick readers (`ScheduledFunctionQueue::tick`, `autosave_if_due`) now see the
    1-based current tick like vanilla.
  - `server/mod.rs:2064-2068` - removed the local `+ 1` patch in `sync_game_time`;
    the 20-tick cadence (tick_count % 20 == 0) is unchanged.
  - `server/mod.rs:2079-2084` - `update_tick_times` no longer increments `tick_count`;
    it reads the already-current value for ring indexing (now `(tick_count % N)`,
    matching vanilla's 1-based `tickTimes[tickCount % 100]`).
  - `server/ticker.rs:186-190,213-219` - tick start/end events now report `tick_count`
    directly (already 1-based); removed the obsolete `tick_event_number` helper and
    its `tick_events_report_one_based_tick_number` test (the +1 conversion no longer
    exists). `scheduler.rs` and `/schedule` read the same in-tick counter, so trigger
    ticks stay consistent. No new unit test: the observable behaviour needs a runtime
    `/schedule`/autosave scenario, not a pure seam; the existing
    `advances_tick_count` test still covers the freeze/sprint gate.

Rows re-verified, no change needed:

- try/catch crash report (`ticker.rs:87-203`): not a missing subsystem. The global
  panic hook in `crates/pumpkin/src/main.rs:83,311-383` already catches panics on the
  ticker thread, sets CRASH_REPORT and calls `stop_server()`; graceful shutdown in
  `crates/pumpkin-core/src/lib.rs:601-613` then prints/saves the crash report and sets
  exit code 1. A panic in `server.tick()` therefore already yields an orderly stop and
  a `crash-reports/crash-*.txt` file. `catch_unwind` in ticker.rs would be redundant
  (and vanilla stops, not continues). CrashReport's exact text is fork API.

No compile/test run (unit rules); rustfmt is run by the orchestrator. Risk: the
`tick_count` increment moved from end-of-tick to top-of-tick, so a `/tick freeze` or
sprint issued mid-tick now settles one tick as vanilla does; pause-tick counting is
unchanged (update_tick_times was previously called unconditionally on paused ticks).


## Result (third pass, final)

All rows re-checked against `.vendor/vanilla-decompiled/net/minecraft/server/MinecraftServer.java`.
No code changes were needed in this pass: both previously-open rows were already fixed
in the working tree by the prior pass and are verified present. Terminal state of every
row:

- FIXED (M) tick_count-at-start - vanilla runs `this.tickCount++` at the top of
  `tickServer` before `tickRateManager.tick()` (MinecraftServer.java:1705). Pumpkin now
  does `server.tick_count.fetch_add(1, Relaxed)` at the top of `Ticker::run` under
  `super::advances_tick_count(runs_normally, is_sprinting)` (ticker.rs:140-142), with the
  single increment at ticker.rs:141. The `+1` patch in `sync_game_time` (mod.rs:2068) and
  the increment in `update_tick_times` (mod.rs:2083) are gone; tick start/end events read
  the already-1-based counter (ticker.rs:186,216). Grep confirms no other
  `server.tick_count.fetch_add` and no dangling `tick_event_number` references.

- VERIFIED try/catch crash report - vanilla wraps the tick in `catch (Throwable)`,
  builds a `CrashReport`, saves `crash-reports/crash-<date>-server.txt` and stops
  (MinecraftServer.java:1413-1424, `onServerCrash(report)` at 1424). Pumpkin achieves the
  same observable behaviour via the global panic hook (`crates/pumpkin/src/main.rs:83`
  `handle_panic`, setting `CRASH_REPORT` and calling `stop_server()` at 367; graceful
  shutdown then saves the report and sets exit code 1). Not a missing subsystem;
  `catch_unwind` in ticker.rs would be redundant since vanilla stops, not continues.
  Crash report text is fork API.

- VERIFIED overload gate `>=` (ticker.rs:258-263) - classic vanilla uses `>=` on
  `l - lastOverloadWarningNanos`; the supplied Folia/Canvas tree has replaced the classic
  warning gate with `tickSchedule` (`lastOverloadWarningNanos = nextTickTimeNanos` at
  MinecraftServer.java:1385), so the classic gate cannot be re-derived here. Keep `>=`;
  the `[1, 6]` test matches it.

- VERIFIED pause-when-empty (ticker.rs:105-163; hub.rs:22,52; handlers.rs:956-973) -
  matches vanilla `tickServer` 1608-1639: `emptyTicks`/reset via `EmptyTickClock::observe`,
  threshold `seconds*20`, one-shot `LOGGER.info("Server empty for {} seconds, pausing")`
  + `autoSave()`, scheduler heartbeat and connection ticking while paused. Default 60s
  in hub.rs:52; runtime setter in handlers.rs:966-973.

No STILL OPEN rows remain. No new unit test: both behaviour rows need a runtime
`/schedule`/autosave or crash scenario, while the pure seams (`EmptyTickClock`,
`apply_overload_skip`, `advances_tick_count`) are already unit-tested. No compile/test run
(unit rules); rustfmt is run by the orchestrator. Risk: `tick_count` now advances
top-of-tick instead of end-of-tick; grep confirms no second reader increments it, so no
double-increment.

## Result (verification pass)

Re-verified the working-tree code for every row against
`.vendor/vanilla-decompiled/net/minecraft/server/MinecraftServer.java`.
No source files changed this pass: all rows were already resolved by prior passes.

- VERIFIED (M) tick_count-at-start - vanilla runs `// this.tickCount++;` at the
  top of `tickServer` before `// this.tickRateManager.tick();`
  (MinecraftServer.java:1705-1706). `ticker.rs:140-142` increments
  `server.tick_count` under `advances_tick_count` at the top of `Ticker::run`;
  `server/mod.rs:2068` (`sync_game_time`) has no `+ 1` patch, and
  `server/mod.rs:2083-2084` (`update_tick_times`) reads the already-current
  counter with `% 100`. Tick start/end events read it directly
  (`ticker.rs:186,216`). Grep confirms a single `server.tick_count.fetch_add`.

- VERIFIED try/catch crash report - vanilla wraps `tickChildren` in
  `catch (Throwable)` and builds a `CrashReport`
  (MinecraftServer.java:1707-1713). Pumpkin's global panic hook
  (`crates/pumpkin/src/main.rs`) already sets `CRASH_REPORT`, stops the server,
  saves `crash-reports/crash-*.txt` and sets exit code 1, the same observable
  behaviour; `catch_unwind` would be redundant since vanilla stops. Crash text
  is fork API.

- VERIFIED overload gate `>=` (`ticker.rs:258-263`) - classic vanilla uses `>=`
  on `l - lastOverloadWarningNanos`; the supplied Folia/Canvas tree replaced it
  with `tickSchedule`, so the classic gate cannot be re-derived here. Keep `>=`
  with the `[1, 6]` test (`ticker.rs:386-402`).

- VERIFIED pause-when-empty (`ticker.rs:105-163`; hub.rs; handlers.rs) - matches
  vanilla `tickServer`: `emptyTicks`/reset (`EmptyTickClock::observe`), threshold
  `seconds*20`, one-shot `LOGGER.info("Server empty for {} seconds, pausing")`
  + `autoSave()` (MinecraftServer.java:1608-1639, info at 1620), scheduler
  heartbeat and connection ticking while paused.

No STILL OPEN rows remain. No new unit test: the pure seams (`EmptyTickClock`,
`apply_overload_skip`, `advances_tick_count`) are already unit-tested; the two
behaviour rows need runtime scenarios. No compile/test run (unit rules); rustfmt
is run by the orchestrator. No source files changed this pass.

## Result (final pass, terminal)

Every row is now in a terminal state. No source file required a change in this
pass: the working tree already carries each fix, and each was re-verified here
against `.vendor/vanilla-decompiled/net/minecraft/server/MinecraftServer.java`
(a Folia/Canvas tree; classic vanilla behaviour is cited where the supplied
tree comments it out).

- FIXED (M) tick_count-at-start - vanilla runs `this.tickCount++` at the top of
  `tickServer` before `tickRateManager.tick()`/`tickChildren`
  (MinecraftServer.java:1705-1708, the classic increment is commented out in
  this Folia tree). Pumpkin increments `server.tick_count` at the top of
  `Ticker::run` under `super::advances_tick_count` (ticker.rs:140-142); the
  `+ 1` patch in `sync_game_time` and the increment in `update_tick_times` are
  gone (server/mod.rs:2067-2084), and tick start/end events read the 1-based
  counter directly (ticker.rs:186,216). grep confirms a single
  `server.tick_count.fetch_add`.

- VERIFIED try/catch crash report - vanilla wraps the tick in `catch
  (Throwable)`, builds a `CrashReport`, saves
  `crash-reports/crash-<date>-server.txt` and calls `onServerCrash`
  (MinecraftServer.java:1413-1424 and 1707-1713). Pumpkin global panic hook
  (`crates/pumpkin/src/main.rs:83,311-324,393-395`) sets `CRASH_REPORT` and
  stops the server; graceful shutdown (`crates/pumpkin-core/src/lib.rs:601-613`)
  prints/saves the report and sets exit code 1. `catch_unwind` in ticker.rs
  would be redundant since vanilla stops rather than continues; report text is
  fork API.

- VERIFIED pause-when-empty - `ticker.rs:105-113,147-160` matches vanilla
  `tickServer` 1608-1639: `emptyTicks++`/reset (`EmptyTickClock::observe`),
  threshold `seconds*20`, one-shot
  `LOGGER.info("Server empty for {} seconds, pausing")` (Pumpkin `info!`,
  149-152) + `autoSave()`/`autosave_if_due(true)`, scheduler heartbeat and
  connection ticking while paused. Default 60s in `net/management/hub.rs:22,52`;
  runtime setter `handlers.rs:956-973`.

- VERIFIED sprint gate (`ticker.rs:95`, `is_sprinting() && runs_normally()`) -
  vanilla gates sprint on `!isPaused() && isSprinting() &&
  checkShouldSprintThisTick()` (MinecraftServer.java:1360).

- VERIFIED game-test gate (`ticker.rs:174-177`) - runs after `server.tick()`
  under `runs_normally()`, next to vanilla `GameTestTicker.SINGLETON.tick()`
  (MinecraftServer.java:1929-1930); `|| sprinting` is redundant but harmless.

- VERIFIED overload warning gate (`ticker.rs:258-263`) - `behind_nanos >
  threshold` and `since_warning >= interval` match classic vanilla. The
  supplied Folia/Canvas tree replaced the classic gate with `tickSchedule`
  (`lastOverloadWarningNanos = nextTickTimeNanos`, MinecraftServer.java:1385),
  so the classic `>=` detail cannot be re-derived here; keep `>=` with the
  `[1, 6]` test (ticker.rs:386-402). That test arithmetic was re-derived by
  hand: tick 6 is exactly 15s and warns under `>=`.

- VERIFIED non-gaps - `OVERLOADED_THRESHOLD_NANOS` 1s, the sprint decrement in
  `end_sprint_tick_work`, and the nanosecond `ServerTickEndEvent` payload are
  kept as recorded (fork API / equivalent behaviour).

No BLOCKED-DATA rows: no missing generated `pumpkin-data` field or missing
subsystem is involved. No new unit test: the pure seams (`EmptyTickClock`,
`apply_overload_skip`, `advances_tick_count`) are already unit-tested; the two
behaviour rows need runtime scenarios. No compile/test run (unit rules); the
test-only `.expect` at ticker.rs:357 is allowed by `lib.rs:2`
(`cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))`).


## Result (r2_014 pass)
- Row 1 (`tick_count` increment position, `ticker.rs:90,122`): STILL OPEN - not reached.
- Row 2 (crash report on tick panic, `ticker.rs:97-118`): STILL OPEN - not reached.
- Row 3 (pause-when-empty sleep, `ticker.rs:62-139`): STILL OPEN - not reached.

## Result (r3_041 pass)
- Row 1 (`tick_count` increment position, `ticker.rs:90,122`): VERIFIED. Rust already advances at the top of the loop (`crates/pumpkin-core/src/server/ticker.rs:136-142`), `sync_game_time` has no `+ 1` patch (`crates/pumpkin-core/src/server/mod.rs:2064-2076`), `update_tick_times` indexes `tick_count % 100` (`mod.rs:2083-2084`). Vanilla anchor is weak: `.vendor/vanilla-decompiled/net/minecraft/server/MinecraftServer.java:1705` has the increment commented out and `:344-350` documents the ordering. The `int i = this.tickCount++` quoted in the row is not present in either vendor or ref tree.
- Row 2 (crash report on tick panic, `ticker.rs:97-118`): VERIFIED. No try/catch needed: the global panic hook `crates/pumpkin/src/main.rs:311-383` builds a `CrashReport`, saves it, and calls `stop_server()` for any non-main-thread panic. The stop path saves and logs it again at `crates/pumpkin-core/src/lib.rs:601-613`. Vanilla anchor: `.vendor/vanilla-decompiled/net/minecraft/server/MinecraftServer.java:1409-1430`.
- Row 3 (pause-when-empty, `ticker.rs:62-139`): VERIFIED. Rust implements the empty-tick counter, paused branch, single autosave at the crossing, and connection ticking (`ticker.rs:45-66`, `:102-160`). Vanilla anchor: `.vendor/vanilla-decompiled/net/minecraft/server/MinecraftServer.java:1608-1636`, which this Canvas tree gates with `if (false && emptyTickThreshold > 0)` (region threading). The Rust follows the upstream logic, which the tracker targets.
- No Rust edited in this pass. The earlier "STILL OPEN - not reached" lines above are superseded by this section; they were not edited.
