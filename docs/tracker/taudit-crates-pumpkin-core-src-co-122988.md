# Tracker: command / time (DEEP AUDIT)

Audited `crates/pumpkin-core/src/command/commands/time.rs` against the vanilla 26.3
command messages in `assets/en_us_java.json` and the Bedrock messages in
`assets/en_us_bedrock.lang`, plus `crates/pumpkin-core/src/world/time.rs`.

No `TODO`/`FIXME` markers exist in the file; the two markers the tracker counted are
the explanatory `vanilla` comments at `time.rs:62` and `time.rs:386`.

## Fixed in this pass

| Gap | Rust | Vanilla / Bedrock | Fix | Size |
|---|---|---|---|---|
| `/time set` fed the clock name into a numeric placeholder for Bedrock (`Set the time to %1$d`) | `time.rs:203-209` | Bedrock `commands.time.clock.set` = `Set the time of '%1$s' to %2$d` | uses `COMMANDS_TIME_CLOCK_SET` with `(clock, ticks)` | S |
| `/time add` same mismatch (`Added %1$d to the time`, order is `added, clock, now`) | `time.rs:222-244` | Bedrock `commands.time.clock.added` = `Added %1$d time to '%2$s'. Time is now %3$d` | per-edition component: Java `set.absolute`, Bedrock `clock.added` with the added amount, clock, new total | S |
| `/time pause` printed `Time <clock>` (Bedrock `commands.time.stop`) | `time.rs:257-266` | Bedrock `commands.time.clock.paused` = `Paused '%1$s' at %2$d` | `COMMANDS_TIME_CLOCK_PAUSED` with `(clock, ticks)` | S |
| `/time resume` printed `Set the time to <clock>` | `time.rs:278-287` | Bedrock `commands.time.clock.resumed` = `Resumed '%1$s' at %2$d` | `COMMANDS_TIME_CLOCK_RESUMED` with `(clock, ticks)` | S |
| `/time query time` printed `Daytime is <clock>` | `time.rs:132-144` | Bedrock `commands.time.clock.query.time` = `Time of '%1$s' is %2$d` | `COMMANDS_TIME_CLOCK_QUERY_TIME` | S |
| `/time rate` printed `Set the time to <clock>` | `time.rs:289-316` | Bedrock has no rate message | Bedrock now prints the rate (`commands.time.set`), Java keeps `commands.time.rate` | S (see gap 6) |

## Remaining gaps (not changed)

| # | Gap | Rust | Vanilla | Suggested fix | Size |
|---|---|---|---|---|---|
| 1 | `/time set <timemarker>` does not exist; `commands.time.set.time_marker`, `set.already_at_time_marker` and `no_time_marker_found` are never emitted | `time.rs:323-345` (set node) | `TimeCommand` accepts a `timemarker` argument resolved against the clock's timeline | add the argument type and emit the marker keys; needs the timeline data first (gap 5) | M |
| 2 | `/time of <clock> ...` accepts any id and always mutates this world's single `LevelTime`; a bogus or foreign clock silently edits the current dimension | `time.rs:105-117`, `183-190`, `405-411`; `argument_types/registry_id.rs:98-100` parses any `Identifier` without a registry check | `ResourceArgument` resolves in the `world_clock` registry and errors with "Unknown clock" / Bedrock `commands.time.clock.failed` | validate against a clock registry (gap 5) and send the matching error key | M |
| 3 | `commands.time.no_default_clock` unused: a dimension without a default clock should fail instead of falling back to `minecraft:overworld` | `time.rs:19`, `105-110`, `183-188` | `TimeCommand` fails with `There is no default clock in dimension %s` | read the dimension type's `default_clock` and emit the key | M |
| 4 | `pause.already_paused`, `pause.already_running`, `rate.already_same`, `set.already_at_time` unused: re-pausing/re-rating still broadcasts `CUpdateTime` and returns success | `time.rs:188-245` (set/add), `247-316` (pause/resume/rate) | vanilla reports the "already" message and leaves the clock untouched | early return with the right key before `send_time` (`send_failure` vs `send_success` not yet verified) | M |
| 5 | Clocks are not modelled: one `LevelTime` per world, `ClockInstance` only used by unit tests | `world/time.rs:5-75` (`ClockInstance`), `77-204` (`LevelTime`) | `ServerClockManager` holds an instance per `world_clock` with its own rate/pause and a timeline registry | store clocks per world keyed by registry id, route `/time of` and the tick through it | L |
| 6 | Bedrock has no `commands.time.rate` key, so `/time rate` from a Bedrock client uses `commands.time.set` ("Set the time to 0.5") | `time.rs:300-304` | Bedrock has no rate command | leave the fallback or add a Pumpkin-specific key | S - STILL OPEN: needs a Pumpkin-specific key decision for Bedrock fallback |
| 7 | `query time` returns `time_of_day`, and `query day`/`daytime` hardcode `minecraft:day` even for `/time of <other clock>` | `time.rs:19-20`, `123-161` | queries report the requested clock's own timeline | use the clock resolved from the `of` branch | S (after gap 5) |
| 8 | Rate and tick counts are printed with Rust `f32`/`i64` `to_string`: `/time rate 1` prints `1`, Java prints `1.0` (`Float.toString`) | `time.rs:230-232`, `300-313` | `Component.translatable(..., rate)` renders `1.0` | format the rate as Java does (`{rate:?}`) | S - FIXED 358b5f3e4 |
| 9 | `/time pause`, `/time resume`, `/time rate` return `1` | `time.rs:266`, `287`, `315` | `TimeCommand` returns the clock's tick count (used by command blocks/`execute if`) | return the clock total ticks | S |
| 10 | `/time add` uses `TimeArgumentType::new(i32::MIN)` while `world/time.rs:161-163` clamps at 0; vanilla's `TimeArgument.time(0)` may reject negative amounts | `time.rs:347-352` | `TimeArgument.time(0)` in Java, but the clamp in `add_time` suggests vanilla accepts and clamps | confirm against decompiled `TimeCommand.addTime` before changing the bound | S (needs vanilla source) |

## Gap status
- `/time set` Bedrock clock message (`time.rs:203-209`, uses `COMMANDS_TIME_CLOCK_SET` at `:206`): FIXED.
- `/time add` Bedrock message (`time.rs:222-244`, `COMMANDS_TIME_CLOCK_ADDED` at `:228`): FIXED.
- `/time pause` Bedrock message (`time.rs:257-266`, `COMMANDS_TIME_CLOCK_PAUSED` at `:260`): FIXED.
- `/time resume` Bedrock message (`time.rs:278-287`, `COMMANDS_TIME_CLOCK_RESUMED` at `:281`): FIXED.
- `/time query time` Bedrock message (`time.rs:132-144`, `COMMANDS_TIME_CLOCK_QUERY_TIME` at `:132`): FIXED.
- `/time rate` Bedrock message (`time.rs:289-316`): STILL OPEN (not re-read this pass).
- 1. `/time set <timemarker>` markers not implemented: STILL OPEN (M; no `time_marker` symbol in `time.rs`).
- 2. Foreign clock id mutates single `LevelTime`: STILL OPEN (M).
- 3. `commands.time.no_default_clock` unused: STILL OPEN (S; key not referenced in `time.rs`).
- 4. `already_paused`/`already_running`/`rate.already_same`/`set.already_at_time` unused: FIXED (UNVERIFIED; time.rs now returns the vanilla errors).
- 5. One `LevelTime` per world, clocks not modelled: STILL OPEN (L).
- 6. Bedrock rate key: STILL OPEN (S).
- 7. Query day/daytime hardcoded `minecraft:day`: STILL OPEN (S).
- 8. f32/i64 `to_string` formatting: STILL OPEN (S).
- 9. Pause/resume/rate return value 1: STILL OPEN (S).
- 10. `/time add` negative amount bounds: STILL OPEN (S; vanilla bound unverified).

## Result

Verified STILL OPEN rows against `.vendor/vanilla-decompiled/net/minecraft/server/commands/TimeCommand.java`
(26.3, Paper/Canvas patched) and `TimeArgument.java`. Smallest rows first.

- `/time rate` Bedrock message (gap-status first row): VERIFIED, no change. `Action::Rate`
  emits `COMMANDS_TIME_SET` with the rate for Bedrock clients (time.rs:346-350).
- 8. f32/i64 `to_string` formatting: VERIFIED, no change. Rate uses `format!("{rate:?}")`
  at time.rs:339/349/356, matching Java `Float.toString` for ordinary values; tick counts
  are integers where Rust `to_string()` already matches Java.
- 10. `/time add` negative amount bounds: VERIFIED, no change. Vanilla `addTime` uses
  `TimeArgument.time(Integer.MIN_VALUE)` (TimeCommand.java:104), matching
  `TimeArgumentType::new(i32::MIN)` at time.rs:394. The doc's "time(0)" concern is about
  `/time set`, not `/time add`, so it does not apply.
- 9. pause/resume/rate return value: FIXED (3 lines). The doc premise was inverted:
  vanilla `setPaused`/`setRate` return `Command.SINGLE_SUCCESS` (1), NOT the clock tick
  count (TimeCommand.java:281,295). Only set/add/query return `wrapTime(ticks)`. Changed
  time.rs:300/327/361 from `Ok(wrap_time(level_time.time_of_day))` to `Ok(1)`.
  Risk: this reverts a prior wrap_time change; it is correct for vanilla parity.
- 3. `commands.time.no_default_clock`: LEFT OPEN. Needs the dimension type `default_clock`
  value from generated data (outside this unit).
- 6. Bedrock rate key: LEFT OPEN. Needs a new translation key, but keys are generated
  (crates/pumpkin-data/src/generated, cannot edit); current `COMMANDS_TIME_SET` fallback kept.
- 7. query day/daytime hardcoded `minecraft:day`: LEFT OPEN. Requires clock/timeline
  resolution from gap 5 (outside this unit).
- 1. `/time set <timemarker>`: LEFT OPEN. Needs ClockTimeMarkers timeline registry data and
  world/time.rs (outside this unit).
- 2. foreign clock id mutates single LevelTime: LEFT OPEN. Needs world_clock registry
  validation and per-clock instances (gap 5, world/time.rs, outside this unit); the generic
  `RegistryIdArgumentType::parse` cannot resolve registry contents.
- 5. one LevelTime per world / clocks not modelled: LEFT OPEN. L-sized, requires the
  ServerClockManager model in world/time.rs (outside this unit).

Test: no pure seam exists for a command executor's return literal, so no new unit test
was added; the existing `wrap_time` test is unchanged.
Files touched: crates/pumpkin-core/src/command/commands/time.rs.


## Result (pass 2, current HEAD 1cddd3e37)

Re-verified every STILL OPEN row against `.vendor/vanilla-decompiled/net/minecraft/server/commands/TimeCommand.java`,
`world/clock/ClockTimeMarker.java`, `world/clock/ServerClockManager.java` and the
en_us translation strings. The source has advanced since the prior Result: rows 1, 2, 3,
5, 9 and 10 are now implemented in `crates/pumpkin-core/src/command/commands/time.rs`
(clean against HEAD). Smallest rows first:

- `/time rate` Bedrock message: verified, no change. Bedrock has no rate key; both branches
  fall back to `COMMANDS_TIME_SET` with the rate. LEFT OPEN (needs a generated Pumpkin key).
- 8. f32/i64 `to_string` formatting: verified. `format!("{rate:?}")` matches Java
  `Float.toString` for ordinary values; only the scientific-notation edge differs.
  LEFT OPEN (minor; a faithful Java formatter is out of scope).
- 10. `/time add` negative bound: verified. `TimeArgumentType::new(i32::MIN)` matches
  vanilla `TimeArgument.time(Integer.MIN_VALUE)`. CLOSED.
- 9. pause/resume/rate return value: verified. `ActionExecutor` returns `Ok(1)` for those
  actions (`Command.SINGLE_SUCCESS`); queries still return `wrap_time` as in vanilla.
  CLOSED.
- 6. Bedrock rate key: LEFT OPEN (same reason as first row; translation keys are generated).
- 1. `/time set <timemarker>`: verified CLOSED. `SetTimeMarkerExecutor`, `builtin_time_marker`
  and the 4 suggestions match `ClockTimeMarker.occursAt`/`resolveTimeToMoveTo` and `day.json`
  (`show_in_commands` only for day/noon/night/midnight). Extended the pure test to cover
  all six builtin markers and their vanilla ticks.
- 2. foreign clock id: verified CLOSED. `resolve_clock_name` checks
  `has_dynamic_registry_entry("minecraft:world_clock")` and errors with `ERROR_UNKNOWN_CLOCK`;
  non-default clocks use `ServerClockManager` instances.
- 3. `no_default_clock`: verified CLOSED. `default_clock_name` returns `None` for the Nether,
  producing `ERROR_NO_DEFAULT_CLOCK`.
- 5. one LevelTime per world / clocks: verified CLOSED. Per-clock `ClockInstance`s live in
  `ServerClockManager` in `world/time.rs`.
- 7. query day/daytime hardcoded `minecraft:day`: verified partial. `ensure_day_timeline_clock`
  reproduces vanilla `wrong_timeline_for_clock` for other clocks, but arbitrary timelines
  (`moon`, `villager_schedule`, datapacks) are not queryable. LEFT OPEN (needs timeline
  registry support, outside this unit).

Test: extended `builtin_markers_resolve_against_the_day_timeline` in `time.rs` (pure, no new
seam needed elsewhere). `rustfmt --edition 2024` run on the touched file. Compile/tests were
NOT run (orchestrator compiles after all workers finish).
Files touched: crates/pumpkin-core/src/command/commands/time.rs (test only); no code change.

## Result (pass 3, current HEAD)

Re-verified the three remaining STILL OPEN rows against
`.vendor/vanilla-decompiled/net/minecraft/server/commands/TimeCommand.java` (26.3, Paper/Canvas
patched) and the generated Bedrock translation table. Every remaining row needs a resource
outside this unit (registry_id.rs / time.rs), so no source change was made.

- 6. Bedrock `/time rate` message: LEFT OPEN. Vanilla has no rate command; Pumpkin falls back
  to `COMMANDS_TIME_SET` ("Set the time to <rate>"). The Bedrock table has no rate key (only
  CLOCK_SET/ADDED/PAUSED/RESUMED/QUERY_TIME/FAILED/NUM_TOOSMALL/TIMEMARKER_*), and adding one
  means editing generated translation data (outside this unit).
- 7. query day/daytime for arbitrary timelines: LEFT OPEN. Vanilla resolves a `timeline`
  resource against the TIMELINE registry in `queryTimelineTicks`/`queryTimelineRepetitions`;
  Pumpkin only models `minecraft:day`. Needs the timeline registry and world/time.rs (outside
  this unit). `ensure_day_timeline_clock` still reproduces vanilla `wrong_timeline_for_clock`
  for non-default clocks.
- 8. rate float formatting: LEFT OPEN. Java `Float.toString` differs from Rust `{rate:?}` only
  for rates in [1e-5, 1e-3) (e.g. "1.0E-5" vs "1e-5"); `{rate:?}` matches Java for every
  ordinary rate in the valid [1e-5, 1000] range. The tested Java formatter
  (`java_float_to_string`) is private in data.rs (outside this unit); duplicating it here
  would risk drift.

Also re-checked the already-CLOSED rows against vanilla; all remain correct: `setPaused` and
`setRate` return `Command.SINGLE_SUCCESS` (1), `addTime` uses `TimeArgument.time(Integer.MIN_VALUE)`,
per-clock instances and the dimension's `default_clock` are modelled, and the `<timemarker>` and
registry-id validation paths match vanilla (TimeCommand.java:104,281,295 and the clock helpers).

No pure seam for a new test; existing tests unchanged. No compile run (orchestrator builds all).
Files touched: docs/tracker/taudit-crates-pumpkin-core-src-co-122988.md (this Result only).
