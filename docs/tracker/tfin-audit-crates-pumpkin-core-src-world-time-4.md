# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-world-time.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: mod.rs time.rs

Re-verified the single remaining PRIOR-OPEN row against decompiled vanilla at
| 5 | PRIOR-OPEN (outside unit) | In-unit `ServerClockManager` keeps one `ClockInstance` per clock name (`server_clock_manager_keeps_clocks_independent`) and is wired by `world/mod.rs:624-633` and `command/commands/time.rs:205-219,375`; `/time set <timemarker>` and `of <clock>` already exist in `command/commands/time.rs` (outside unit). Residue needs full datapack timeline-attribute data (L, cross-crate). |
Re-verified the only remaining PRIOR-OPEN row (5) against decompiled vanilla at
No `PRIOR-OPEN` rows remain. No new unit test added: every pure seam already has

## Result
All gap rows were re-verified against decompiled vanilla under
`.vendor/vanilla-decompiled/net/minecraft/`. No source edits were needed: row 5's
residue (environment-attribute timeline tracks) is implemented and wired, and
every pure seam already has a failing-without-fix test.

| # | Verdict | Vanilla evidence |
|---|---------|------------------|
| 5 | FIXED / VERIFIED | Clock registry + `/time set <timemarker>` + `of <clock>` and the environment-attribute timeline tracks are all present. `world/time.rs` `ServerClockManager` keeps one `ClockInstance` per clock name (`server_clock_manager_keeps_clocks_independent`); `SleepWakeMarker`/`resolve_time_to_move_to` match `world/clock/ClockTimeMarker.java` `resolveTimeToMoveTo`. The keyframe tables in `crates/pumpkin-data/src/generated/environment_attribute.rs` match `world/timeline/Timelines.java` exactly (sky-light 133/1.0, 11867/1.0, 13670/0.26666668, 22330/0.26666668; monsters-burn 12542/false 23460/true; bees-stay-in-hive 12542/true 23460/false; creaking-active/eyeblossom-open 12600/true 23401/false; cat-gift 362/0.0 23667/0.7; day markers 1000/6000/13000/18000, wake-up 0). `world/environment.rs` `sample_float_track`/`sample_bool_track`/`sample_step_float_track` and `apply_weather_sky_light` reproduce `util/KeyframeTrackSampler.java` cyclic sampling and `world/attribute/WeatherAttributes.java` rain/thunder layering. `command/commands/time.rs` resolves markers from the `minecraft:day` timeline registry NBT. |

Confirmation of the closed rows (all VERIFIED against
`.vendor/vanilla-decompiled/net/minecraft/`): row 1 is a Bedrock `SetTime`
protocol limit (no frozen field; `send_game_time_sync` resends every 20-tick
sync); row 2's night window (12600..23401) matches `world/timeline/Timelines.java`
with the skip gated on `advance_time && is_dark_outside()`; row 3 matches
`world/clock/ClockTimeMarker.java` `resolveTimeToMoveTo`; row 4 persists
`world_clocks` with a round-trip test; row 6 carries the per-player day time in
`send_game_time_sync` matching `ServerClockManager.packNetworkState`; row 7's
`default_clock_id` reads the `minecraft:world_clock` raw id matching
`world/clock/WorldClocks.java` `bootstrap` (overworld 0, the_end 1).

No new unit test added: every pure seam already has a failing-without-fix test
(`clock_instance_ticking`, `level_time_set_and_add`,
`level_time_pack_network_state_marks_frozen_clock`,
`server_clock_manager_keeps_clocks_independent`, the three wake-marker tests,
`default_clock_name_matches_vanilla_dimension_types`, the two daylight-detector
weather tests, and the `set_time.rs` codec tests). Another would duplicate them.
