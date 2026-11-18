# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-co-122988.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: data.rs

Re-verified every STILL OPEN row against `.vendor/vanilla-decompiled/net/minecraft/server/commands/TimeCommand.java`,
Re-verified the three remaining STILL OPEN rows against
Re-checked the three STILL OPEN rows. This unit now also covers `data.rs`, so gap 8
Re-verified the two STILL OPEN rows against vanilla

## Result (FINAL PASS)

Final classification. Re-verified every row against
`.vendor/vanilla-decompiled/net/minecraft/server/commands/TimeCommand.java` (26.3, Paper/Canvas
patched) and the generated tables in `crates/pumpkin-data/src/generated/translation.rs`. Worked
smallest first. No source file was edited this pass: neither remaining row can be closed without
a generated field or a whole missing subsystem, and the one previously-fixed row is intact.

- 8. f32 `to_string` rate formatting: VERIFIED (no change). Vanilla `setRate` passes a raw float to
  `Component.translatable(...)` (`TimeCommand.java:286-296`), which renders `Float.toString` style
  (`1.0`, `1.0E-5`). `data.rs:588` exposes `java_float_to_string` (Java-format scientific notation)
  with the pure unit test `java_float_to_string_matches_java_format` at `data.rs:1263`. `time.rs`
  uses it at all six rate/error call sites (469, 476, 483, 645, 655, 662). Fix is intact.
- 6. Bedrock `/time rate` message: BLOCKED-DATA. Vanilla `setRate` sends `commands.time.rate`
  (`TimeCommand.java:286-296`). The Java key exists (`translation.rs:7606 COMMANDS_TIME_RATE`), but
  the Bedrock module has NO rate key: its complete `commands.time.*` set is ADDED, CLOCK_ADDED,
  CLOCK_FAILED, CLOCK_NUM_TOOSMALL, CLOCK_PAUSED, CLOCK_QUERY_TIME, CLOCK_RESUMED, CLOCK_SET,
  CLOCK_TIMEMARKER_FAILED, CLOCK_TIMEMARKER_PREVIOUS_FAILED, DESCRIPTION, DISABLED, QUERY_DAY,
  QUERY_DAYTIME, QUERY_GAMETIME, SET, STOP (`translation.rs:22464-22498`). Exact missing generated
  field: a Bedrock `commands.time.rate` translation constant in
  `crates/pumpkin-data/src/generated/translation.rs` (generated data, off-limits, needs the
  Extractor). The `translation::bedrock::COMMANDS_TIME_SET` fallback in `time.rs:475/654` is kept.
- 7. query day/daytime for arbitrary timelines: BLOCKED-DATA. Vanilla `queryTimelineTicks` /
  `queryTimelineRepetitions` (`TimeCommand.java:170-194`) resolve a `timeline` resource via
  `ResourceArgument.resource(context, Registries.TIMELINE)` (`:121`), suggest via `suggestTimelines`
  (`:142-151`), reject a clock mismatch via `wrong_timeline_for_clock` (`:171-173`, `:184-186`),
  and read `getCurrentTicks` / `getPeriodCount` per clock. `TimeCommand.java:124-129` also has the
  `repetition` subcommand. Exact missing subsystem: a command-side `minecraft:timeline` resource
  argument with registry holder lookup, suggestions, the `repetition` subcommand and per-timeline
  tick computation. Pumpkin only models `minecraft:day` (`time.rs` `ensure_day_timeline_clock`,
  `QueryMode::DayTime`/`Day` at `:248-275/:323-338`), so arbitrary timelines (moon,
  villager_schedule, datapacks) are not queryable. The Java keys already exist
  (`translation.rs:7601/7603` and `COMMANDS_TIME_WRONG_TIMELINE_FOR_CLOCK`), so the only gap is the
  resolution subsystem.

All other rows remain FIXED/CLOSED from earlier passes; no regressions found. No new pure test
seam exists (both blocked rows depend on the missing resources above). No source file changed, so
no rustfmt needed. No compile run (orchestrator builds all).
