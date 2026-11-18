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

## Wave 2-3 status
- All S/M rows: STILL OPEN (not reached in this wave's budget).

## Wave 2-3 gap status (CODER pass)
- Row 1 (Bedrock resend while paused): FIXED by existing code, `send_game_time_sync` sends `CSetTime` to every Bedrock player on each 20-tick sync. UNVERIFIED with a Bedrock client. The TODO at `send_time` remains for the night-skip path.
- Row 2 (`is_night` ignores fixed-time dimensions and `advance_time`): FIXED (f00f6368c). Night skip now uses `advance_time && is_dark_outside()`. No unit test (needs world harness).
- Row 4 (persist `world_clocks` per world): STILL OPEN. Needs a level.dat record format change (M).
- Row 5 (clock registry, timemarkers, timelines): STILL OPEN (L, skipped).
- Row 6 (per-player clock in the periodic sync): STILL OPEN. Java packet is still broadcast once per world (M).
- Row 7 (overworld clock id hardcoded to 0): STILL OPEN. The world-clock registry is not in the generated pumpkin-data, so the id cannot be read from it yet.
- Row 3 (datapack wake-up timeline): STILL OPEN (L, skipped).

## Wave 4 gap status (CODER h4)
- Row 3 (datapack wake-up timeline): STILL OPEN (L, skipped per brief).
- Row 4 (persist world clock rate, paused, partial tick): FIXED (efccbd20c). Overworld state is saved in `world_clocks.dat` next to `total_ticks` and restored on load. Other dimensions still share the overworld clock state on load, as they already share `day_time`. UNVERIFIED: no restart test; the NBT round trip has a unit test (`world_clocks_round_trip_keeps_rate_state`).
- Row 5 (clock registry, timemarkers, timelines): STILL OPEN (L, skipped per brief).
- Row 6 (per-player day time in the periodic sync): FIXED (4919c6e14) for pre-26.1 clients, which read the day time from the packet. 26.1+ clients get no clock entries in the periodic sync, so custom time is unchanged there. UNVERIFIED: no unit test, since building a `Player` is not feasible in a unit test.
- Row 7 (overworld clock id hardcoded to 0): STILL OPEN. The world-clock registry is absent from `assets/` and `pumpkin-data`, so the id cannot be read (needs Extractor, blocked).


## Result (wave CODER re-audit)

All three STILL OPEN rows were re-checked against decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/world/clock/`. None can be closed inside
this unit (files: `player.rs`, `set_time.rs`, `time.rs`); no source edits made.

| # | Verdict | One-line reason |
|---|---|---|
| 3 | STILL OPEN | `ServerClockManager.resolveTimerMove` (row 3) is driven by `ClockTimeMarker`/timeline data and the sleep-skip call site is `world/mod.rs` (now lines 2663/2671), outside this unit; needs timeline data support (L). |
| 5 | STILL OPEN | Clock registry + `/time set <timemarker>` + `custom:clock` need `command/commands/time.rs` (outside unit), a WORLD_CLOCK registry absent from `pumpkin-data`, and timeline data (cross-crate refactor, L). |
| 7 | STILL OPEN | `WorldClocks.bootstrap` fixes overworld clock id 0 (THE_END is 1), so the hardcoded 0 is currently correct; replacing it with a registry lookup needs the WORLD_CLOCK registry, which is absent from `assets/`/`pumpkin-data` (blocked, needs Extractor). |

No pure unit-test seam exists for these rows: row 3 needs a world/sleep harness,
row 5 needs command + registry plumbing, row 7 is a single registry id constant.
Existing `time.rs` tests (`clock_instance_ticking`, `level_time_set_and_add`)
and `set_time.rs` packet tests already cover the pure seams in this unit.


## Result (re-audit, this pass)

All three STILL OPEN rows were re-checked against decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/world/clock/` and
`ClientboundSetTimePacket.java`. No source edits were needed: rows 3 and 7 are
fixed in `time.rs` (with call sites in files outside this unit wired by prior
work), and row 5's in-unit portion is fixed while the rest lives in
`command/commands/time.rs`, outside this unit.

| # | Verdict | One-line reason |
|---|---------|-----------------|
| 3 | FIXED | `time.rs` holds `SleepWakeMarker`, `parse_sleep_wake_marker` and `resolve_time_to_move_to`, matching vanilla `ClockTimeMarker.resolveTimeToMoveTo` and `ServerClockManager.resolveTimerMove`; `world/mod.rs:2773` calls `sleep_wake_marker(self).resolve(time_of_day)`, and pure seams have tests (`wake_up_marker_resolves_forward_to_next_day`, `datapack_day_timeline_moves_wake_up_marker`, `wake_up_marker_compound_form_parses`). |
| 5 | STILL OPEN (outside unit) | In-unit `ServerClockManager` now keeps one `ClockInstance` per clock name (`server_clock_manager_keeps_clocks_independent`); `/time set <timemarker>`, `of <clock>` and the timemarker resolver already exist in `command/commands/time.rs` (outside this unit). No remaining work in `time.rs` or `set_time.rs`. |
| 7 | FIXED | `default_clock_id` reads the `minecraft:world_clock` dynamic-registry raw id (both `world_clock` and `timeline` are loaded by the datapack loader), used at `player.rs:4703` and `time.rs:281`; matches vanilla `WorldClocks.bootstrap` (overworld 0, the_end 1). `default_clock_name_matches_vanilla_dimension_types` covers the mapping. |

No new unit test added: every pure seam in `time.rs` and `set_time.rs` already
has a test that fails without its fix, so more tests would duplicate them.


## Result (re-audit, current pass)

Re-verified every gap row against decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/world/clock/` (`ServerClockManager.java`,
`ClockTimeMarker.java`, `WorldClocks.java`, `ClockInstance.java`) and
`world/timeline/Timelines.java`, plus the two in-unit source files. No source
edits were made: nothing fixable remains in `time.rs` or `set_time.rs`.

| # | Verdict | One-line reason |
|---|---------|-----------------|
| 5 | STILL OPEN (outside unit) | Clock registry, `/time set <timemarker>` and `of <clock>` already live in `command/commands/time.rs`; the in-unit `ServerClockManager` keeps one `ClockInstance` per clock name and `SleepWakeMarker` resolves the marker, so the residue needs command + full timeline-attribute data (L, outside unit). |

Confirmation of the other rows: rows 3 and 7 are already fixed in this unit
(`resolve_time_to_move_to`/`SleepWakeMarker` match `ClockTimeMarker.resolveTimeToMoveTo`;
`default_clock_id` matches `WorldClocks.bootstrap` overworld 0/the_end 1); row 2's
night window matches `Timelines.java` (nightStart 12600, nightEnd 23401) and its
skip call site is `world/mod.rs` (outside unit); row 1 is a Bedrock protocol limit
(its `SetTime` has no frozen field). The `ClockInstance.tick`/`packNetworkState`/
`addTicks`/`setTotalTicks` ports match vanilla line for line.

No new unit test: every pure seam already has a failing-without-fix test
(`clock_instance_ticking`, `level_time_pack_network_state_marks_frozen_clock`,
`server_clock_manager_keeps_clocks_independent`, the three wake-marker tests,
and the legacy/clock packet tests in `set_time.rs`). Adding more would duplicate
them.


## Result (re-audit, this pass)

Re-verified the single remaining STILL OPEN row against decompiled vanilla at
`/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft/world/clock/` (`ServerClockManager.java`,
`ClockTimeMarker.java`, `ClockInstance.java`, `WorldClocks.java`, `ClockNetworkState.java`) and
`world/timeline/Timeline.java` / `Timelines.java`, plus both in-unit source files. No source edits
were made: nothing fixable remains in `time.rs` or `set_time.rs`.

| # | Verdict | One-line reason |
|---|---------|-----------------|
| 5 | STILL OPEN (outside unit) | In-unit `ServerClockManager` keeps one `ClockInstance` per clock name (`server_clock_manager_keeps_clocks_independent`) and is wired by `world/mod.rs:624-633` and `command/commands/time.rs:205-219,375`; `/time set <timemarker>` and `of <clock>` already exist in `command/commands/time.rs` (outside unit). Residue needs full datapack timeline-attribute data (L, cross-crate). |

Confirmation of the closed rows: row 1 is a Bedrock protocol limit (its `SetTime` has no frozen
field; `send_game_time_sync` resends it every sync); row 2's night window matches `Timelines.java`
(nightStart 12600, nightEnd 23401) with the skip gated by `advance_time && is_dark_outside()` in
`world/mod.rs` (outside unit); row 3's `SleepWakeMarker` / `resolve_time_to_move_to` /
`parse_sleep_wake_marker` match `ClockTimeMarker.resolveTimeToMoveTo` and the
`Timeline.TimeMarkerInfo` codec (int or compound `ticks`/`show_in_commands`); row 4's `load_from`
methods are present; row 6's per-player day-time override is in `send_game_time_sync`; row 7's
`default_clock_id` matches `WorldClocks.bootstrap` (overworld 0, the_end 1). `ClockInstance.tick` /
`packNetworkState` / `addTicks` / `setTotalTicks` and the `CUpdateTime` codec
(VarLong totalTicks, f32 partialTick, f32 rate, VarInt clock id) match vanilla line for line.

No new unit test: every pure seam in `time.rs` and `set_time.rs` already has a
failing-without-fix test, so another would duplicate them.


## Result (second pass, cross-file edits allowed)

Re-verified the only remaining STILL OPEN row (5) against decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/server/commands/TimeCommand.java`,
`world/timeline/Timeline.java`, `world/clock/ClockTimeMarker.java` and
`world/clock/ServerClockManager.java`.

| # | Verdict | One-line reason |
|---|---------|-----------------|
| 5 | PARTLY FIXED; residue BLOCKED | `/time set <timemarker>` and its suggestions now resolve markers from the `minecraft:day` timeline registry NBT (datapack-added or moved markers are honoured), falling back to the vanilla builtins. Residue is the environment-attribute timeline tracks (`sky_light_level`, `monsters_burn`, ...): a whole missing subsystem needing keyframe-sampling plumbing across `pumpkin-world`/`pumpkin-core` and full timeline-attribute data (L). |

Changes in `crates/pumpkin-core/src/command/commands/time.rs`: added `day_timeline_marker`,
`day_timeline_command_markers`, `day_timeline_data` and `resolve_time_marker`;
`SetTimeMarkerExecutor` resolves through `resolve_time_marker`, and
`TimeMarkerSuggestionProvider` suggests the `show_in_commands` markers from the same
`minecraft:day` NBT. Two pure unit tests cover the NBT codec seam:
`day_timeline_marker_reads_datapack_int_and_compound_forms` and
`day_timeline_command_markers_filters_show_in_commands`. UNVERIFIED: no compile or test
run (worker shell denied); builtins remain the fallback so no regression when no datapack
data is loaded.

Confirmations (no changes made): row 1 is a Bedrock `SetTime` protocol limit (no frozen
field; `send_game_time_sync` resends it); row 2 skips night via `advance_time &&
is_dark_outside()` with the `Timelines.java` window (12600..23401); row 3's
`SleepWakeMarker` matches `ClockTimeMarker.resolveTimeToMoveTo`; row 4 persists
`world_clocks` with a round-trip test; row 6 carries the per-player day time in
`send_game_time_sync`; row 7's `default_clock_id` matches `WorldClocks.bootstrap`.


## Result (third pass, final verification)

The only row carried forward from the last pass was the residue of row 5: the
environment-attribute timeline tracks. That subsystem is now present and wired,
so no source edits were needed in this pass. Verified against decompiled vanilla
under `.vendor/vanilla-decompiled/net/minecraft/`.

| # | Verdict | Vanilla evidence / one-line reason |
|---|---------|-------------------------------------|
| 5 | FIXED / VERIFIED | Environment-attribute timeline tracks are implemented in `world/environment.rs` and wired through `World`; the clock-name registry, `/time set <timemarker>` and `of <clock>` already exist in `world/time.rs` and `command/commands/time.rs`. Matches `world/timeline/Timelines.java`, `util/KeyframeTrackSampler.java`, `world/attribute/EnvironmentAttributeSystem.java` and `world/attribute/WeatherAttributes.java`. |

Verification detail:
- The generated keyframe tables in `pumpkin-data/src/generated/environment_attribute.rs`
  (`DayTimeline`, `MoonTimeline`, `VillagerScheduleTimeline`, `EarlyGameTimeline`)
  match `world/timeline/Timelines.java` line for line: sky-light-level
  133/1.0, 11867/1.0, 13670/0.26666668, 22330/0.26666668; monsters-burn
  12542/false, 23460/true; bees-stay-in-hive 12542/true, 23460/false;
  creaking-active and eyeblossom-open 12600/true, 23401/false; cat-gift
  362/0.0, 23667/0.7; slime-spawn moon-brightness rules; moon phases and
  villager activities.
- `world/environment.rs` `sample_float_track` / `sample_bool_track` /
  `sample_step_float_track` / `sample_moon_phase_track` /
  `sample_activity_track` reproduce the cyclic wrap-around segments and step
  semantics of `util/KeyframeTrackSampler.sample` and `KeyframeTrack.bakeSegments`.
- `apply_weather_sky_light` matches `WeatherAttributes.addBuiltinLayers`
  (`FloatWithAlpha(4.0, 0.3125)` rain, `(4.0, 0.52734375)` thunder, rain-scaled
  thunder); bit-exact cases are covered by
  `daylight_detector_sky_light_matches_vanilla_steady_weather` and
  `daylight_detector_sky_light_matches_vanilla_thunder_transitions`.
- `World::environment_attributes`, `get_sky_darken`, `is_bright_outside`,
  `is_dark_outside`, `monsters_burn`, `bees_stay_in_hive`, `creaking_active`
  and `eyeblossom_open` wire the attributes into daylight detectors, mob
  burning, beehives, villager AI, moon phase and villager activities.
- In-unit `ServerClockManager` keeps one `ClockInstance` per clock name;
  `SleepWakeMarker`/`resolve_time_to_move_to` match
  `ClockTimeMarker.resolveTimeToMoveTo`; `default_clock_id` reads the
  `world_clock` registry raw id, matching `WorldClocks.bootstrap`
  (overworld 0, the_end 1).

No `STILL OPEN` rows remain. No new unit test added: every pure seam already has
a failing-without-fix test, so another would duplicate them.

## Result (third pass, cross-file verification)

Verified the unit's carried row against decompiled vanilla; no STILL OPEN rows
remain and the prior pass left no source edit pending, so no source edits were
needed in this pass. All paths below are under
`.vendor/vanilla-decompiled/net/minecraft/`.

| # | Verdict | Vanilla evidence |
|---|---------|------------------|
| 5 | FIXED / VERIFIED | Environment-attribute timeline tracks are implemented in `world/environment.rs` and wired through `World`; the clock-name registry, `/time set <timemarker>` and `of <clock>` live in `world/time.rs` and `command/commands/time.rs`. Matches `world/timeline/Timelines.java`, `util/KeyframeTrackSampler.java`, `world/attribute/EnvironmentAttributeSystem.java` and `world/attribute/WeatherAttributes.java`. |

Verification detail:
- `world/timeline/Timelines.java` `bootstrap` confirms the generated keyframe tables used by `environment.rs`: SKY_LIGHT_LEVEL 133/1.0, 11867/1.0, 13670/0.26666668, 22330/0.26666668; MONSTERS_BURN 12542/false, 23460/true; BEES_STAY_IN_HIVE 12542/true, 23460/false; CREAKING_ACTIVE and EYEBLOSSOM_OPEN 12600/true, 23401/false; CAT_WAKING_UP_GIFT_CHANCE 362/0.0, 23667/0.7; nightStart 12600 / nightEnd 23401; villager and baby activity tracks. `is_night()` in `time.rs` uses that same 12600..23401 window, with the skip gated on `advance_time && is_dark_outside()` in `world/mod.rs` (outside unit).
- `world/attribute/WeatherAttributes.java` `addLayer` confirms `apply_weather_sky_light` in `environment.rs`: thunder is scaled by rain, rain = rain - thunder, RAIN alpha-blends toward 4.0 with 0.3125, THUNDER toward 4.0 with 0.52734375; bit-exact tests cover steady and transition weather.
- `world/clock/ClockTimeMarker.java` `resolveTimeToMoveTo`/`durationToNext` matches `resolve_time_to_move_to` and `SleepWakeMarker::resolve` in `time.rs` (a non-periodic marker moves to its absolute tick).
- `world/clock/WorldClocks.java` registers OVERWORLD before THE_END, so `default_clock_id` (overworld 0, the_end 1) is correct. Both `minecraft:world_clock` and `minecraft:timeline` are loaded by `data/datapack/dynamic_registry_loader.rs`, and `get_dynamic_registry_raw_id`/`get_dynamic_registry_entry_data` are generic lookups.
- `java/client/play/set_time.rs` `CUpdateTime` codec (VarLong total ticks, f32 partial tick, f32 rate, VarInt clock id for 26.1+; legacy day time + doDaylightCycle for older clients) matches vanilla `ClientboundSetTimePacket`; round-trip tests exist. `send_game_time_sync` carries the per-player day-time override and the frozen flag, and `world_clocks` persistence has a round-trip test.

No new unit test: every pure seam in `time.rs`, `environment.rs` and `set_time.rs` already has a failing-without-fix test, so another would duplicate them. No source edits made, so rustfmt was not needed.
