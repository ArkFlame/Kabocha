# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-se-1269c5.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: mod.rs

All rows closed in FINAL PASS: close_admission holds player_join_lock; spawn_world fires WorldInitEvent + WorldLoadEvent; get_tps caps at target rate. All re-verified in the Result section below; no source edit was needed.

## Result
FINAL PASS: all rows in this unit are closed. The fixes were already present in code
from earlier passes; each was re-read and re-verified against decompiled vanilla / Pumpkin's
own startup path. No source edit was needed this pass.

| Row | Final status | Evidence |
|---|---|---|
| `close_admission` holds `player_join_lock` | FIXED (present) | `crates/pumpkin-core/src/server/mod.rs:1466-1477` takes `player_join_lock` at lines 1469-1472 before setting `admission_closed`; the join path takes the same lock at `server/mod.rs:1259-1262`. A join between the admission check and world insertion therefore cannot slip past the shutdown kick pass. No vanilla equivalent (Pumpkin concurrency fix); vanilla shutdown is single-threaded in `net/minecraft/server/MinecraftServer.java`. |
| `spawn_world` fires `WorldInitEvent` + `WorldLoadEvent` | VERIFIED (FIXED, present) | `server/mod.rs:945-951` fires `WorldInitEvent` then `WorldLoadEvent` for a runtime-created world, matching the startup path at `server/mod.rs:550-564` which fires both in the same order. These are Paper/Bukkit API events, not vanilla classes (no match under `.vendor/vanilla-decompiled`); the vanilla level create/load path is `net/minecraft/server/MinecraftServer.java` `createLevels()`. |
| `get_tps` caps at the target rate | VERIFIED (FIXED, present) | `server/mod.rs:2175-2182` computes `1000/mspt` then caps with `.min(1e9 / nanoseconds_per_tick())`. Target-rate definition matches `.vendor/vanilla-decompiled/net/minecraft/world/TickRateManager.java:10,17,24-29` (`nanosecondsPerTick = NANOSECONDS_PER_SECOND / tickrate`, default 20 TPS) and `.vendor/vanilla-decompiled/net/minecraft/server/ServerTickRateManager.java:120-125` (`setTickRate` delegates to the parent). The ticker never runs above target (`server/ticker.rs:99`), so the cap stops a healthy 2 ms tick reporting 500 TPS. `ring_tps` (`server/mod.rs:2295-2309`) applies the same ideal-rate cap to the rolling 1/5/15-minute windows.
