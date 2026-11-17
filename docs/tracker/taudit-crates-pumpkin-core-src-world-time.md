# Audit: crates/pumpkin-core/src/world/time.rs

Reviewed: `crates/pumpkin-core/src/world/time.rs` (`ClockInstance`, `LevelTime`), plus its
callers `world/mod.rs` (tick / skip-night / sync), `server/mod.rs` (`sync_game_time`),
`command/commands/time.rs` and `entity/player.rs` (join + per-player time).
Vanilla reference: decompiled Canvas/Paper tree at
`/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft` (`.vendor/` absent in this worktree):
`world/clock/ServerClockManager.java`, `world/clock/ClockInstance.java`, `world/timeline/Timeline.java`,
`server/level/ServerLevel.java`, `server/MinecraftServer.java`, `server/commands/TimeCommand.java`.

## Fixed in this pass

| Where | Vanilla | Gap | Fix |
|---|---|---|---|
| `world/time.rs:147-165` + `pumpkin-protocol/.../set_time.rs` | pre-26.1 `ClientboundSetTimePacket(gameTime, dayTime, doDaylightCycle)` | periodic sync sent an empty clock map, so clients < 26.1 read day time 0 and `doDaylightCycle = true` (sky snapped to day 0 twice a second, frozen worlds kept advancing). TODO at the old line 145 | `CUpdateTime.legacy_day_time` carries each world's day time and frozen flag; the writer uses it for < 26.1 and still falls back to the first clock entry |

## Remaining gaps

| # | file:line | Vanilla class / member | Gap | Suggested fix | Size |
|---|---|---|---|---|---|
| 1 | `world/time.rs:131` | Bedrock `SetTime` (time only) | TODO: a paused or `advance_time=false` world still sends only the current day time, so a Bedrock client keeps advancing its own sky. Java clients are frozen by `rate = 0` in `CUpdateTime`, Bedrock has no equivalent field | while `paused || !advance_time`, resend `CSetTime` (same value) on the 20-tick sync; verify the client holds the sky | S, needs a Bedrock client to verify |
| 2 | `world/time.rs:206-209`, caller `world/mod.rs:2426,2450` | `Level.isBrightOutside` / `isDarkOutside` (1.21.9+), `ServerLevel.tickSleep` | `is_night()` is raw `time_of_day % 24000 >= 12000`: it ignores `dimension.has_fixed_time` (a Nether/End level at 18000 reports night) and ignores `advance_time`. `World::is_dark_outside` already exists (`world/mod.rs:6629`) | in `tick_environment`, use `self.is_dark_outside()` and require the `advance_time` gamerule before skipping; then `LevelTime::is_night` has no caller and can go | S |
| 3 | `world/mod.rs:2456-2457` (uses `LevelTime::set_time`) | `ServerClockManager.resolveTimerMove(clock, WAKE_UP_FROM_SLEEP)` | sleep-skip hardcodes next morning (`time + 24000 - time % 24000`); datapack timelines that move `wake_up_from_sleep` to another tick are ignored | resolve the marker from the timeline data and `set_time` to its target | L (needs timeline data support) |
| 4 | `world/mod.rs:541-543`, `server/mod.rs:641-663` | `ServerClockManager` (`world_clocks` SavedData: totalTicks, partialTick, rate, paused) | only `day_time`/`game_time` reach `level.dat`; `/time pause`, `/time rate` and the partial tick reset to 1.0/unpaused/0 on restart, per world | persist a `world_clocks` record per world like vanilla's SavedData and `load_from` it | M |
| 5 | `world/time.rs:6-75`, `command/commands/time.rs` | `ClockManager` interface, `ClockTimeMarker`, `Timeline`, `ServerClockManager.moveToTimeMarker` | one implicit clock per level; no `/time set <timemarker>`, no `custom:clock` argument, no timeline attribute tracks. `ClockInstance` is otherwise a faithful port (tick, `packNetworkState`, `addTicks` clamp, `setTotalTicks` partial reset all match) | add a clock registry keyed by `world_clock` id and a marker resolver | L |
| 6 | `world/time.rs:147-165` | `ServerClockManager.packNetworkState(clock, instance, player)` | periodic sync builds one packet per world for every player; vanilla substitutes `player.getPlayerTime()` when the player's dimension default clock matches and honours `relativeTime`. Only the join path (`entity/player.rs:4547`) is per-player, so a player's custom time is overwritten on the next sync | use `player.client_time_of_day`/per-player override in the periodic loop too | M |
| 7 | `world/time.rs:130`, `entity/player.rs:4558,4568` | `Holder<WorldClock>` registry id | `CUpdateTime::new_clock(..., 0, ...)` hardcodes clock id 0 for `minecraft:overworld`; correct only while the generated world-clock registry keeps overworld first | read the id from `pumpkin-data` world-clock registry | S (verify) |
