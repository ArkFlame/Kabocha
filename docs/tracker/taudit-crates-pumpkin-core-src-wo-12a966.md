# Audit: crates/pumpkin-core/src/world/time.rs

Reviewed: `ClockInstance` and `LevelTime`, plus callers `world/mod.rs` (tick,
skip-night, `force_game_time_synchronization`), `server/mod.rs:641`
(`sync_level_time`), `command/commands/time.rs` and `entity/player.rs:4535`
(join / per-player time). `.vendor/` is absent in this worktree; vanilla
reference is decompiled 26.1/Paper trees and `ClockInstance`/`ServerClockManager`
behaviour.

TODO/FIXME markers found: 2 before this pass
(`:131` Bedrock frozen clock, `:145` multiversion legacy day time). The second is
fixed here and its marker removed; the first remains.

## Fixed in this pass

| Where | Vanilla | Gap | Fix |
|---|---|---|---|
| `world/time.rs:147-164`, `pumpkin-protocol/.../set_time.rs` | pre-26.1 `ClientboundSetTimePacket(gameTime, dayTime, doDaylightCycle)` | periodic sync sent an empty clock map; clients < 26.1 fell back to day 0 with `doDaylightCycle = true`, so the sky snapped to day 0 twice a second and frozen worlds kept advancing. TODO at old `:145` | `CUpdateTime.legacy_day_time: Option<(i64, bool)>`; the periodic sync fills each world's day time and frozen flag, the writer uses it for < 26.1, falling back to the first clock entry then day 0. Unit tests cover the pre-26.1 layout and the 26.1+ layout |

Note: branch `695f0b3d5` (tracker `taudit-crates-pumpkin-core-src-world-time.md`)
carries an equivalent fix; this pass reimplemented it on the current master
because master still has the empty-map TODO and the old writer.

## Remaining gaps

| # | file:line | Vanilla class / member | Gap | Suggested fix | Size |
|---|---|---|---|---|---|
| 1 | `world/time.rs:131` | Bedrock `SetTime` (time only) | TODO: a paused or `advance_time = false` world still sends only the day time, so a Bedrock client keeps advancing its own sky; Java is frozen by `rate = 0` in `CUpdateTime`, Bedrock has no equivalent field | while `paused \|\| !advance_time`, resend `CSetTime` (same value) on the 20-tick sync; needs a Bedrock client to verify | S |
| 2 | `world/time.rs:207-209`, caller `world/mod.rs:2482,2506` | `Level.isBrightOutside` / `isDarkOutside`, `ServerLevel.tickSleep` | `is_night()` is raw `time_of_day % 24000 >= 12000`: it ignores `dimension_type.has_fixed_time` (Nether/End at 18000 reports night) and ignores `advance_time`; `World::is_dark_outside` already exists | use `self.is_dark_outside()` for the skip-night check and require `advance_time`; then `LevelTime::is_night` loses its only caller | S |
| 3 | `world/mod.rs:2512-2513` via `LevelTime::set_time` | `ServerClockManager.resolveTimerMove(clock, WAKE_UP_FROM_SLEEP)` | sleep-skip hardcodes next morning (`time + 24000 - time % 24000`); datapack timelines that move `wake_up_from_sleep` are ignored | resolve the marker from timeline data and set the clock to its target | L (needs timeline data) |
| 4 | `world/mod.rs:549-551`, `server/mod.rs:641-663` | `ServerClockManager` `world_clocks` SavedData (`totalTicks`, `partialTick`, `rate`, `paused`) | only `day_time`/`game_time` are persisted; `/time pause`, `/time rate` and the partial tick reset on restart, per world | persist a per-world `world_clocks` record and `load_from` it | M |
| 5 | `world/time.rs:6-75`, `command/commands/time.rs` | `ClockManager`, `ClockTimeMarker`, `Timeline`, `moveToTimeMarker` | one implicit clock per level: no `/time set <timemarker>`, no `custom:clock` argument, no timeline attribute tracks. `ClockInstance` is otherwise a faithful port (tick, `packNetworkState`, `addTicks` clamp at 0, `setTotalTicks` partial reset all match) | add a clock registry keyed by world-clock id and a marker resolver | L |
| 6 | `world/time.rs:147-164` | `ServerClockManager.packNetworkState(clock, instance, player)` | periodic sync builds one packet per world for every player; vanilla substitutes `player.getPlayerTime()` when the dimension's default clock matches and honours `relativeTime`. Only the join path (`entity/player.rs:4546`) is per-player, so a custom player time is overwritten on the next sync | apply the per-player override in the periodic loop too | M |
| 7 | `world/time.rs:130,149`, `entity/player.rs:4555,4567` | `Holder<WorldClock>` registry id | `clock id 0` hardcoded for `minecraft:overworld`: correct only while the generated registry keeps overworld first | read the id from the `pumpkin-data` world-clock registry | S (verify) |
| 8 | `entity/player.rs:4568` vs `world/time.rs:131` | Bedrock `SetTime` field | join path sends `query_daytime()` (0..23999) while the broadcast path sends absolute `time_of_day`; one of them is wrong for day counts | decide against a Bedrock client and use the same value in both | S (needs client) |
| 9 | `world/time.rs:167-169` | `TimeCommand.addTime` | `add_time` clamps to 0; vanilla (`/time add -x` at day 0) leaves the day time negative | drop the clamp only if negative absolute day time is accepted downstream (`%`/`/` already match vanilla) | S (verify) |
