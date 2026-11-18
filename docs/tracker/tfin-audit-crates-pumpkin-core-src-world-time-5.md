# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-world-time.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

Verified the unit's carried row against decompiled vanilla; no STILL OPEN rows
No `STILL OPEN` rows and no "STILL OPEN until closed..." line remain; no source

## Result
FINAL PASS: every row FIXED / VERIFIED against decompiled vanilla under
`.vendor/vanilla-decompiled/net/minecraft/`. No BLOCKED-DATA rows: no missing generated
field or subsystem remains. No source edits were needed; all fixes and their failing-without-fix
unit tests were confirmed present in the current checkout.

| # | Verdict | Vanilla evidence (paths under `.vendor/vanilla-decompiled/net/minecraft/`) |
|---|---------|--------------------------------------------------------------------------|
| 1 | VERIFIED | Bedrock `CSetTime` has no frozen field; `world/time.rs` `send_game_time_sync` resends it each 20-tick sync. `network/protocol/game/ClientboundSetTimePacket.java` record `(long gameTime, Map<Holder<WorldClock>, ClockNetworkState> clockUpdates)` matches the `java/client/play/set_time.rs` `CUpdateTime` codec. |
| 2 | VERIFIED | `is_night()` window 12600..23401 matches `world/timeline/Timelines.java` (`nightStart = 12600`, `nightEnd = 23401`); night skip gated by `advance_time && is_dark_outside()` in `world/mod.rs` (outside unit). |
| 3 | VERIFIED | `SleepWakeMarker` / `resolve_time_to_move_to` / `parse_sleep_wake_marker` in `world/time.rs` match `world/clock/ClockTimeMarker.java` `resolveTimeToMoveTo` / `durationToNext` (non-periodic markers move to their absolute tick; periodic advance without rewinding) and the `Timeline.TimeMarkerInfo` codec. |
| 4 | VERIFIED | `world_clocks` persistence (`world_clocks.dat`, `load_from`) matches the `ServerClockManager` SavedData (totalTicks, partialTick, rate, paused); `world_clocks_round_trip_keeps_rate_state` test present. |
| 5 | VERIFIED | Clock-name registry, `/time set <timemarker>` and `of <clock>` (`world/time.rs`, `command/commands/time.rs`) plus environment-attribute timeline tracks (`world/environment.rs`, generated `environment_attribute.rs`) match `world/timeline/Timelines.java`, `util/KeyframeTrackSampler.java`, `world/attribute/EnvironmentAttributeSystem.java` and `world/attribute/WeatherAttributes.java`. Verified keyframes: day sky 133/1.0, 11867/1.0, 13670/0.26666668, 22330/0.26666668; monsters burn 12542/false, 23460/true; bees in hive 12542/true, 23460/false; creaking/eyeblossom 12600/true, 23401/false. |
| 6 | VERIFIED | `send_game_time_sync` carries the per-player day-time override and frozen flag (`legacy_day_time` in `CUpdateTime`), matching `world/clock/ServerClockManager.java` `packNetworkState` and `network/protocol/game/ClientboundSetTimePacket.java`. |
| 7 | VERIFIED | `default_clock_id` reads the `minecraft:world_clock` raw id (both `world_clock` and `timeline` loaded in `dynamic_registry_loader.rs`), matching `world/clock/WorldClocks.java` `bootstrap` (OVERWORLD registered first = 0, THE_END = 1). |

Confirmation: `ClockInstance.tick` / `packNetworkState` / `addTicks` / `setTotalTicks` ports
match `world/clock/ClockInstance.java` line for line. Pure seams already have failing-without-fix
tests (`clock_instance_ticking`, `level_time_set_and_add`, `level_time_pack_network_state_marks_frozen_clock`,
`server_clock_manager_keeps_clocks_independent`, the three wake-marker tests,
`default_clock_name_matches_vanilla_dimension_types`, the two daylight-detector weather tests,
the `day_timeline_marker` tests and the `set_time.rs` codec tests), so no new test was added.
No source files edited, so rustfmt was not needed.
