# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-world-time.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: mod.rs time.rs

- Row 7 (overworld clock id hardcoded to 0): STILL OPEN. The world-clock registry is absent from `assets/` and `pumpkin-data`, so the id cannot be read (needs Extractor, blocked).
All three STILL OPEN rows were re-checked against decompiled vanilla at
| 3 | STILL OPEN | `ServerClockManager.resolveTimerMove` (row 3) is driven by `ClockTimeMarker`/timeline data and the sleep-skip call site is `world/mod.rs` (now lines 2663/2671), outside this unit; needs timeline data support (L). |
| 5 | STILL OPEN | Clock registry + `/time set <timemarker>` + `custom:clock` need `command/commands/time.rs` (outside unit), a WORLD_CLOCK registry absent from `pumpkin-data`, and timeline data (cross-crate refactor, L). |

## Result
FINAL PASS: all three carried rows re-verified against decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/` and against the current source
(`world/time.rs`, `command/commands/time.rs`, `world/mod.rs`,
`data/datapack/dynamic_registry_loader.rs`, `world/environment.rs` and the
generated `pumpkin-data/src/generated/environment_attribute.rs`). No source
edits were needed: every row is already fixed in code. No `STILL OPEN` rows
remain.

| # | Verdict | Vanilla evidence |
|---|---------|------------------|
| 7 | FIXED / VERIFIED | `default_clock_id` (`world/time.rs:24-38`) reads the `minecraft:world_clock` dynamic-registry raw id instead of hardcoding 0; matches `.vendor/vanilla-decompiled/net/minecraft/world/clock/WorldClocks.java` `bootstrap`, where OVERWORLD is registered first (id 0) and THE_END second (id 1). Both `minecraft:world_clock` and `minecraft:timeline` are loaded in `data/datapack/dynamic_registry_loader.rs:48-49`. Pure seam covered by `default_clock_name_matches_vanilla_dimension_types`. |
| 3 | FIXED / VERIFIED | `resolve_time_to_move_to` / `SleepWakeMarker::resolve` / `sleep_wake_marker` / `parse_sleep_wake_marker` (`world/time.rs:44-117`) match `.vendor/vanilla-decompiled/net/minecraft/world/clock/ClockTimeMarker.java` `resolveTimeToMoveTo`/`durationToNext` (non-periodic marker goes to its absolute tick; periodic marker advances by `duration > 0 ? duration : period + duration`) and the `Timeline.TimeMarkerInfo` codec (plain int or compound `ticks`). The sleep-skip call site `world/mod.rs:2797` calls `sleep_wake_marker(self).resolve(time_of_day)`. Pure seams covered by `wake_up_marker_resolves_forward_to_next_day`, `datapack_day_timeline_moves_wake_up_marker` and `wake_up_marker_compound_form_parses`. |
| 5 | FIXED / VERIFIED | In-unit `ServerClockManager` keeps one `ClockInstance` per clock name; environment-attribute timeline tracks live in `world/environment.rs` and generated `environment_attribute.rs`, matching `.vendor/vanilla-decompiled/net/minecraft/world/timeline/Timelines.java`, `util/KeyframeTrackSampler.java`, `world/attribute/EnvironmentAttributeSystem.java` and `world/attribute/WeatherAttributes.java`. `/time set <timemarker>` and `of <clock>` live in `command/commands/time.rs` (`resolve_time_marker`, `day_timeline_marker`, `SetTimeMarkerExecutor`, `TimeMarkerSuggestionProvider`), outside this unit, matching `.vendor/vanilla-decompiled/net/minecraft/server/commands/TimeCommand.java`. |

No new unit test added: every pure seam in `time.rs`, `command/commands/time.rs`
and the packet code already has a failing-without-fix test, so another would
duplicate them. No source files were edited, so rustfmt was not needed.
