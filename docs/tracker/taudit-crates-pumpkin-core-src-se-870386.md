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
FIXED: OVERLOADED_THRESHOLD_NANOS is 1s (server/ticker.rs:17-18). STILL OPEN: game-test freeze gate ordering and sprint redundancy not re-read.


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
