# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-se-870386.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

Re-verified every STILL OPEN row against
Second pass with cross-file edits permitted. Both remaining STILL OPEN rows were
No STILL OPEN rows remain. No new unit test: both behaviour rows need a runtime
No STILL OPEN rows remain. No new unit test: the pure seams (`EmptyTickClock`,

## Result
FINAL PASS - every row terminal. Re-verified the working-tree code against
`.vendor/vanilla-decompiled/net/minecraft/server/MinecraftServer.java` (a
Folia/Canvas tree; classic vanilla lines are commented out but present). No
source file required a change this pass: each fix is already in the tree.

- FIXED (M) tick_count-at-start - vanilla runs `// this.tickCount++;` at the top
  of `tickServer` before `// this.tickRateManager.tick();`
  (MinecraftServer.java:1705-1706). `ticker.rs:140-142` increments
  `server.tick_count` under `super::advances_tick_count(runs_normally,
  is_sprinting)` at the top of `Ticker::run`; `server/mod.rs:2068`
  (`sync_game_time`) has no `+ 1` patch and `server/mod.rs:2083-2084`
  (`update_tick_times`) reads the 1-based counter with `% 100`. Tick start/end
  events read it directly (ticker.rs:186,216). grep confirms a single
  `server.tick_count.fetch_add` and no dangling `tick_event_number`.

- VERIFIED try/catch crash report - vanilla wraps `tickChildren` in
  `catch (Throwable)`, builds a `CrashReport`, saves
  `crash-reports/crash-<date>-server.txt` and calls `onServerCrash`
  (MinecraftServer.java:1413-1424, 1707-1713, onServerCrash at 1592). Pumpkin's
  global panic hook (crates/pumpkin/src/main.rs:83,311-395) sets `CRASH_REPORT`
  and calls `stop_server()`; graceful shutdown
  (crates/pumpkin-core/src/lib.rs:601-613) prints/saves the report and sets exit
  code 1. Same observable behaviour; `catch_unwind` would be redundant since
  vanilla stops rather than continues. Report text is fork API.

- VERIFIED pause-when-empty - `ticker.rs:105-160` matches vanilla `tickServer`
  1608-1639: `emptyTicks++`/reset (`EmptyTickClock::observe`), threshold
  `seconds * 20` (MinecraftServer.java:1608), one-shot
  `LOGGER.info("Server empty for {} seconds, pausing")` (Pumpkin `info!` at
  ticker.rs:149-152; vanilla 1620) + `autoSave()`/`autosave_if_due(true)`,
  scheduler heartbeat and connection ticking while paused (vanilla 1624-1636).
  Default 60s in `net/management/hub.rs:52`; runtime setter
  `handlers.rs:956-973`.

- VERIFIED sprint gate (`ticker.rs:95`, `is_sprinting() && runs_normally()`) -
  vanilla gates sprint on `!isPaused() && isSprinting() &&
  checkShouldSprintThisTick()` (MinecraftServer.java:1360).

- VERIFIED game-test gate (`ticker.rs:174-177`) - runs after `server.tick()`
  under `runs_normally()`, next to vanilla `GameTestTicker.SINGLETON.tick()`
  under `runsNormally()` (MinecraftServer.java:1929-1930); `|| sprinting` is
  redundant but harmless.

- VERIFIED overload warning gate (`ticker.rs:258-263`) - classic vanilla uses
  `>=` on `l - lastOverloadWarningNanos`; the supplied Folia/Canvas tree replaced
  it with `tickSchedule` (`lastOverloadWarningNanos = nextTickTimeNanos`,
  MinecraftServer.java:1385), so the classic gate cannot be re-derived here.
  Keep `>=` with the `[1, 6]` test (ticker.rs:386-402); arithmetic re-derived by
  hand (tick 6 at 18s gives `since_warning = 15s`, warns under `>=`).

- VERIFIED non-gaps - `OVERLOADED_THRESHOLD_NANOS` 1s, the sprint decrement in
  `end_sprint_tick_work`, and the nanosecond `ServerTickEndEvent` payload are
  kept as recorded (fork API / equivalent behaviour).

No BLOCKED-DATA rows: no missing generated `pumpkin-data` field or missing
subsystem is involved. No new unit test: the pure seams (`EmptyTickClock`,
`apply_overload_skip`, `advances_tick_count`) are already unit-tested; the two
behaviour rows need runtime scenarios. No compile/test run (unit rules);
rustfmt is run by the orchestrator.
