# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-server-tic.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- 1 FIXED: tick-count freeze semantics, M, many call sites.
- 2 FIXED: overload-warning gate needs decompiled 1.20.5+ server, S.
- 3 VERIFIED: plugin-visible event numbering is an API semantic change, S.
- 4 VERIFIED: 1m/5m/15m rings, M.

## Result

All four rows are closed in the current tree; no source edit was needed this pass.

- 1 (tick-count freeze semantics): FIXED. `crates/pumpkin-core/src/server/ticker.rs:140-142` increments `tick_count` only when `super::advances_tick_count(runs_normally, is_sprinting)` is true; `server/mod.rs:2332-2334` returns `runs_normally || is_sprinting`. `update_tick_times` (`server/mod.rs:2079-2116`) keeps the ring writes unconditional and no longer increments the count, so frozen ticks reuse the last slot while MSPT/TPS still record. `sync_game_time` (`:2067-2068`), scheduled-function indexing (`:2004-2007`) and autosave (`:2036-2038`) read the gated count. Folia comments out `this.tickCount++` at `.vendor/vanilla-decompiled/net/minecraft/server/MinecraftServer.java:1705` and Paper's note at `:343-357` confirms tickCount advances during the server tick; the `runsNormally()` gate (`.vendor/vanilla-decompiled/net/minecraft/world/TickRateManager.java:32-34,56-61`) is the prescribed vanilla design. Test: `tick_count_advances_only_when_game_runs` (`server/mod.rs:2470-2478`).

- 2 (overload-warning gate): FIXED. `ticker.rs:232-276` `apply_overload_skip` takes `runs_normally`, `run()` passes `manager.runs_normally()` (`:117-122`), and the `this_tick_nanos <= 0` early return (`:244-250`) is the vanilla `i > 0` equivalent for sprint ticks, so frozen/zero-length ticks never warn. Folia's tick loop (`.vendor/vanilla-decompiled/net/minecraft/server/MinecraftServer.java:1357-1385`) uses catchup scheduling and removed the "Can't keep up" log, so the exact vanilla sub-condition is not observable here; the implemented gate matches the audit's prescribed design. Tests: `frozen_overload_does_not_skip_or_warn`, `sustained_overload_warns_again_after_wall_clock_interval` (`ticker.rs:373-402`).

- 3 (plugin-visible tick numbers): VERIFIED (1-based). `tick_count` is incremented at the top of the loop before events fire, so both events carry the current 1-based tick (`ticker.rs:186-192,214-220`); plugin docs state 1-based in `server_tick_start.rs:8-9` and `server_tick_end.rs:8`. Paper/Folia passes `getCurrentTick()` to both events at `.vendor/vanilla-decompiled/net/minecraft/server/MinecraftServer.java:1657` and `:1753`, confirming 1-based start and end. (The `tick_event_number` helper/test named in older Result notes is not present in this tree; the increment placement is the implementation.)

- 4 (1m/5m/15m TPS rings): VERIFIED. `server/mod.rs` keeps `tick_times_1m/5m/15m_nanos` rings (`:171-173`, `:428-432`), sizes 1200/6000/18000 (`:92-94`), fed from `update_tick_times` (`:2096-2116`), with `get_recent_tps`, `get_tps_1m/5m/15m` (`:2185-2214`) and `ring_tps` (`:2295-2309`) returning the ideal rate until samples fill. Paper declares `tickTimes1m/5m/15m` at `.vendor/vanilla-decompiled/net/minecraft/server/MinecraftServer.java:333-335` and `computeTPS()` returns their TPS in that order at `:409-416`. Test: `ring_tps_uses_filled_samples_and_caps_at_target` (`server/mod.rs:2368-2383`).

No cargo/rustfmt run (forbidden for this worker); the orchestrator's batch check covers the tree.

## Final pass verification

Re-verified all four rows against the current tree and decompiled vanilla; no source edit was required because each row was already implemented correctly. No cargo/rustfmt run (forbidden for this worker); the orchestrator's batch check covers the tree.

- 1: ticker.rs:140-142 gates the tick_count increment on advances_tick_count(runs_normally, is_sprinting), with server/mod.rs:2332-2334 returning runs_normally || is_sprinting. Vanilla gate confirmed at .vendor/vanilla-decompiled/net/minecraft/world/TickRateManager.java:32-34,56-61 (runsNormally() returns runGameElements, updated by tick() from isFrozen || frozenTicksToRun > 0) and the commented-out "this.tickCount++;" at .vendor/vanilla-decompiled/net/minecraft/server/MinecraftServer.java:1705.
- 2: ticker.rs:117-122 passes manager.runs_normally() into apply_overload_skip (:232-276), which returns early when this_tick_nanos <= 0 for sprint/frozen ticks. Folia's catch-up scheduling without the "Can't keep up" log is at .vendor/vanilla-decompiled/net/minecraft/server/MinecraftServer.java:1357-1385.
- 3: tick_count is incremented at the top of the loop (ticker.rs:140-142) before both events fire (start :214-220, end :186-192), so both carry the 1-based tick. Paper passes getCurrentTick() to ServerTickStartEvent at .vendor/vanilla-decompiled/net/minecraft/server/MinecraftServer.java:1657 and to ServerTickEndEvent at :1753.
- 4: rings tick_times_1m/5m/15m_nanos (server/mod.rs:171-173, fed at :2096-2116) with get_recent_tps/get_tps_1m/5m/15m (:2185-2214) and ring_tps (:2295-2309). Paper declares the rings at .vendor/vanilla-decompiled/net/minecraft/server/MinecraftServer.java:333-335 and returns them in 1m/5m/15m order from computeTPS() at :409-416.
