# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-co-122988.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: time.rs

| 6 | Bedrock has no `commands.time.rate` key, so `/time rate` from a Bedrock client uses `commands.time.set` ("Set the time to 0.5") | `time.rs:300-304` | Bedrock has no rate command | leave the fallback or add a Pumpkin-specific key | S - STILL OPEN: needs a Pumpkin-specific key decision for Bedrock fallback |
- `/time rate` Bedrock message (`time.rs:289-316`): STILL OPEN (not re-read this pass).
- 1. `/time set <timemarker>` markers not implemented: STILL OPEN (M; no `time_marker` symbol in `time.rs`).
- 2. Foreign clock id mutates single `LevelTime`: STILL OPEN (M).

## Result
Final pass. Re-verified every row against
`.vendor/vanilla-decompiled/net/minecraft/server/commands/TimeCommand.java` (26.3,
Paper/Canvas patched) and the generated Bedrock translation table. All source fixes
are present in `crates/pumpkin-core/src/command/commands/time.rs`; no further source
edit was needed. Rows in doc order:

- Row 6 (Bedrock `/time rate` key): VERIFIED. Vanilla `setRate` sends
  `commands.time.rate` (TimeCommand.java:286-296), a Java-only key; Bedrock has no
  rate command and no rate key in the generated table
  (`crates/pumpkin-data/src/generated/translation.rs:22464-22498` has only
  ADDED/CLOCK_*/DESCRIPTION/DISABLED/QUERY_*/SET/STOP/TIMEMARKER_*). The row's own
  suggested fix allows "leave the fallback", so the `COMMANDS_TIME_SET`
  ("Set the time to <rate>") fallback is retained in both rate paths
  (`time.rs:473-477` and `time.rs:652-656`). A dedicated Pumpkin key would be
  BLOCKED-DATA (missing generated Bedrock `commands.time.rate` constant), but that
  is an optional enhancement, not needed for vanilla parity.
- `/time rate` Bedrock message (`time.rs:289-316`): VERIFIED, same as row 6. The Java
  branch sends `commands.time.rate` with (clock, rate) exactly as vanilla
  (TimeCommand.java:294); the Bedrock branch keeps the `commands.time.set` fallback.
- 1. `/time set <timemarker>`: FIXED (present in tree, re-verified). Vanilla
  `setTimeToTimeMarker`/`moveToTimeMarker` (TimeCommand.java:235-265) resolve a
  `ClockTimeMarker` on the day timeline and never rewind. `SetTimeMarkerExecutor`,
  `builtin_time_marker`, `marker_occurs_at`, `resolve_time_to_move_to`, the
  `show_in_commands` suggestions and the `timemarker` argument branch reproduce
  this; `resolve_time_marker` also reads datapack timeline NBT. Cite
  `.vendor/vanilla-decompiled/net/minecraft/server/commands/TimeCommand.java:90-99,235-265`
  and `world/clock/ClockTimeMarker.java`.
- 2. Foreign clock id mutates single `LevelTime`: FIXED (present in tree,
  re-verified). `resolve_clock_name` checks
  `has_dynamic_registry_entry("minecraft:world_clock")`, matching vanilla
  `ResourceArgument.resource(context, Registries.WORLD_CLOCK)`
  (TimeCommand.java:68-75), and errors with `ERROR_UNKNOWN_CLOCK` for unknown ids.
  Non-default clocks route through `execute_other_clock` to per-clock
  `ServerClockManager` instances in `LevelTime.clocks`, matching vanilla
  `clockManager.getInstance(clock)` (TimeCommand.java:164,200,229,237).

Tests: existing pure tests in `time.rs` (`wrap_time`,
`day_timeline_query_rejects_other_clocks`,
`builtin_markers_resolve_against_the_day_timeline`,
`time_marker_move_advances_forward_without_rewinding`, and the two datapack NBT
tests) already cover rows 1 and 2; no new seam was needed. No source file edited
this pass. No compile run (orchestrator builds all).
