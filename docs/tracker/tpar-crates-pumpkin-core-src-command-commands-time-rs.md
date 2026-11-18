# Parity rows for crates/pumpkin-core/src/command/commands/time.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-co-122988** [partial/OWN-OPEN] own audit: 16 open, 9 fixed, 4 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-co-122988.md) docs/tracker/taudit-crates-pumpkin-core-src-co-122988.md
- **own/taudit-crates-pumpkin-core-src-world-time** [partial/OWN-OPEN] own audit: 19 open, 7 fixed, 4 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-world-time.md) docs/tracker/taudit-crates-pumpkin-core-src-world-time.md

## Result
FINAL. Both rows verified against decompiled vanilla `.vendor/vanilla-decompiled/net/minecraft/server/commands/TimeCommand.java` (26.3). Smallest first; neither row is fixable inside the time.rs / environment.rs / translation.rs unit, so no source file changed and no new test seam exists.

- 6. Bedrock `/time rate` message: BLOCKED-DATA. Vanilla `setRate` (`TimeCommand.java:286-296`) sends only Java `commands.time.rate` (`Component.translatable("commands.time.rate", clock, rate)`); Bedrock has no `/time rate` command or message. `assets/en_us_bedrock.lang` has no rate key (only CLOCK_SET / ADDED / PAUSED / RESUMED / QUERY_TIME / FAILED / NUM_TOO_SMALL / TIMEMARKER_* / SET), and `crates/pumpkin-data/src/generated/translation.rs` defines `COMMANDS_TIME_RATE` only in the `java` module (line 7606), not `bedrock`. Exact missing generated field: `translation::bedrock::COMMANDS_TIME_RATE` plus its `assets/en_us_bedrock.lang` entry. The `COMMANDS_TIME_SET` fallback in time.rs is kept; a native Bedrock client cannot issue `/time rate`, so no parity is lost.
- 7. query day/daytime for arbitrary timelines: BLOCKED-DATA. Vanilla `queryTimelineTicks` / `queryTimelineRepetitions` (`TimeCommand.java:170-193`) resolve a `timeline` resource via `ResourceArgument.resource(context, Registries.TIMELINE)` (`:121-131,142-151`) and compute ticks through `Timeline.getCurrentTicks` / `getPeriodCount`. Pumpkin only models `minecraft:day` (`ensure_day_timeline_clock`, time.rs:186-197), which already reproduces the vanilla `wrong_timeline_for_clock` error. Exact missing subsystem: a command-side `minecraft:timeline` resource argument with registry holder lookup, `suggestTimelines`, the `repetition` subcommand and per-timeline tick computation. The Java query keys already exist (`COMMANDS_TIME_QUERY_TIMELINE` / `_REPETITIONS`), so this is a resolution subsystem gap, not a data gap.


## Result (third pass, final)

Closed the two remaining STILL OPEN rows of the referenced command audit
(`docs/tracker/taudit-crates-pumpkin-core-src-co-122988.md`) and re-checked the
world-time audit (`docs/tracker/taudit-crates-pumpkin-core-src-world-time.md`).
Both command rows require a resource outside the time.rs unit, so no source
edit was possible; each is terminal-classified below. Vanilla paths are under
`.vendor/vanilla-decompiled/net/minecraft/`. Smallest first.

- 6. Bedrock `/time rate` message: BLOCKED-DATA. Vanilla `TimeCommand.setRate`
  (`server/commands/TimeCommand.java`, `setRate`, ~lines 286-296) sends
  `commands.time.rate`. The Java key exists in the generated table
  (`crates/pumpkin-data/src/generated/translation.rs`, `COMMANDS_TIME_RATE`),
  but the Bedrock module (translation.rs:22464-22498) has NO rate key (only
  CLOCK_SET / CLOCK_ADDED / CLOCK_PAUSED / CLOCK_RESUMED / CLOCK_QUERY_TIME /
  CLOCK_FAILED / NUM_TOOSMALL / TIMEMARKER_* / QUERY_DAY / QUERY_DAYTIME /
  QUERY_GAMETIME / SET / STOP). Exact missing generated field: a Bedrock
  `commands.time.rate` translation constant in
  `crates/pumpkin-data/src/generated/translation.rs` (generated data, off-limits,
  needs the Extractor). The `COMMANDS_TIME_SET` fallback in time.rs Action::Rate
  is kept; Bedrock has no `/time rate` command, so the fallback is unreachable
  from a native Bedrock client and causes no parity regression.

- 7. query day/daytime for arbitrary timelines: BLOCKED-DATA. Vanilla
  `queryTimelineTicks` / `queryTimelineRepetitions`
  (`server/commands/TimeCommand.java:121-131,142-151,170-194`) resolve a
  `timeline` resource via `ResourceArgument.resource(context, Registries.TIMELINE)`,
  suggest via `suggestTimelines`, expose a `repetition` subcommand and compute
  ticks with `Timeline.getCurrentTicks` / `getPeriodCount`
  (`world/timeline/Timeline.java:112-128`, reading the bound clock's totalTicks
  and applying `period_ticks`). Pumpkin only models the `minecraft:day` timeline
  through `ensure_day_timeline_clock` (time.rs), which correctly emits the
  vanilla `wrong_timeline_for_clock` error for other clocks but cannot query
  arbitrary timelines (moon, villager_schedule, datapacks). Exact missing
  subsystem: a command-side `minecraft:timeline` resource argument with registry
  holder lookup, clock-filtered suggestions, the `repetition` subcommand and
  per-timeline tick/repetition computation. The Java query keys already exist
  (`COMMANDS_TIME_QUERY_TIMELINE` / `_REPETITIONS`), so this is a resolution
  subsystem gap, not a data gap (L, cross-crate).

Referenced world-time audit: no STILL OPEN rows remain. Its carried row 5 is
FIXED / VERIFIED (final pass): environment-attribute timeline tracks live in
`world/environment.rs` and the clock registry, `/time set <timemarker>` and
`of <clock>` in `world/time.rs` / `command/commands/time.rs`, matching
`world/timeline/Timelines.java`, `util/KeyframeTrackSampler.java`,
`world/attribute/EnvironmentAttributeSystem.java` and `WorldClocks.bootstrap`.

No source files changed this pass (no fixable row remains inside time.rs), so
rustfmt was not needed and no compile/test run was performed (orchestrator
builds all). No new unit-test seam exists: both remaining rows are data/subsystem
gaps, and every fixable row's pure seam is already covered by existing tests.
