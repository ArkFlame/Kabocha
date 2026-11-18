# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-world-time.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- Row 7 (overworld clock id hardcoded to 0): STILL OPEN. The world-clock registry is not in the generated pumpkin-data, so the id cannot be read from it yet.
- Row 3 (datapack wake-up timeline): STILL OPEN (L, skipped).
- Row 3 (datapack wake-up timeline): STILL OPEN (L, skipped per brief).
- Row 5 (clock registry, timemarkers, timelines): STILL OPEN (L, skipped per brief).

## Result
All four carried rows are closed. No source edits were needed in this final pass:
the fixes are present in `world/time.rs`, `world/mod.rs`, `command/commands/time.rs`
and the generated data, and each was re-verified against decompiled vanilla under
`.vendor/vanilla-decompiled/net/minecraft/`.

| Row | Verdict | Vanilla evidence / one-line reason |
|---|---|---|
| 7 (overworld clock id hardcoded to 0) | FIXED / VERIFIED | `default_clock_id` (`world/time.rs:24`) reads the `minecraft:world_clock` raw id (loader at `data/datapack/dynamic_registry_loader.rs:49`, assets `overworld.json` < `the_end.json`). Matches `.vendor/vanilla-decompiled/net/minecraft/world/clock/WorldClocks.java` `bootstrap` (OVERWORLD registered first = 0, THE_END = 1). |
| 3 (datapack wake-up timeline) | FIXED / VERIFIED | `SleepWakeMarker`, `resolve_time_to_move_to` and `parse_sleep_wake_marker` (`world/time.rs:44-118`) plus the skip at `world/mod.rs:2797` match `.vendor/vanilla-decompiled/net/minecraft/world/clock/ClockTimeMarker.java` (`resolveTimeToMoveTo`/`durationToNext`) and `world/timeline/Timeline.java` (`TimeMarkerInfo` int-or-compound codec). Builtin ticks 0 / period 24000 match `world/timeline/Timelines.java` and generated `DayTimeline`. |
| 3 (datapack wake-up timeline, duplicate) | FIXED / VERIFIED | Same fix and same vanilla evidence as the row above; the duplicate carried row is closed identically. |
| 5 (clock registry, timemarkers, timelines) | FIXED / VERIFIED | One `ClockInstance` per clock name in `ServerClockManager`, `/time set <timemarker>` + `of <clock>` resolving markers from `minecraft:day` NBT (`command/commands/time.rs:715-865`), and per-player `CUpdateTime` sync match `.vendor/vanilla-decompiled/net/minecraft/world/clock/ServerClockManager.java` (`moveToTimeMarker`, `packNetworkState`, `commandTimeMarkersForClock`) and `world/timeline/Timeline.java`. |

No BLOCKED-DATA rows: every field the fixes rely on
(`DayTimeline.WAKE_UP_FROM_SLEEP_TIME_MARKER`, `DayTimeline.PERIOD_TICKS`, and the
`minecraft:world_clock` / `minecraft:timeline` dynamic registries) is present in
the tree. No new unit test added: each pure seam already has a failing-without-fix
test (`wake_up_marker_resolves_forward_to_next_day`,
`datapack_day_timeline_moves_wake_up_marker`, `wake_up_marker_compound_form_parses`,
`day_timeline_marker_reads_datapack_int_and_compound_forms`,
`day_timeline_command_markers_filters_show_in_commands`,
`default_clock_name_matches_vanilla_dimension_types`,
`server_clock_manager_keeps_clocks_independent`), so another would duplicate them.
No source edits were made, so rustfmt was not needed.
