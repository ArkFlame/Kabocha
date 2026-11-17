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
