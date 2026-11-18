# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-co-122988.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: time.rs

- 3. `commands.time.no_default_clock` unused: STILL OPEN (S; key not referenced in `time.rs`).
- 5. One `LevelTime` per world, clocks not modelled: STILL OPEN (L).
- 6. Bedrock rate key: STILL OPEN (S).
- 7. Query day/daytime hardcoded `minecraft:day`: STILL OPEN (S).

## Result
Final pass against vanilla .vendor/vanilla-decompiled/net/minecraft/server/commands/TimeCommand.java (26.3, Paper/Canvas patched), crates/pumpkin-data/src/generated/translation.rs and the current Pumpkin source. Smallest rows first. No source change was needed: rows 3 and 5 are already implemented and match vanilla; rows 6 and 7 require a generated field or a whole subsystem outside this unit.

- 3. commands.time.no_default_clock: VERIFIED (matches vanilla). Pumpkin resolve_clock_name (time.rs:149-163) emits ERROR_NO_DEFAULT_CLOCK (time.rs:55-58) when default_clock_name returns None; default_clock_name returns None for the Nether (world/time.rs:13-19). Matches vanilla getDefaultClock/ERROR_NO_DEFAULT_CLOCK at TimeCommand.java:36-37 and :303-305 (dimensionType.value().defaultClock().orElseThrow(...)). No change.
- 5. One LevelTime per world / clocks not modelled: VERIFIED (matches vanilla). ServerClockManager holds one independent ClockInstance per world_clock (world/time.rs:188-215), owned by LevelTime.clocks (world/time.rs:219-244); /time of routes non-default clocks through it. Matches the vanilla per-clock ServerClockManager model noted at TimeCommand.java:138 (source.getLevel().clockManager()). No change.
- 6. Bedrock rate key: BLOCKED-DATA. Vanilla setRate sends commands.time.rate (TimeCommand.java:286-296). The Java constant exists (translation.rs:7606 COMMANDS_TIME_RATE), but the Bedrock module (translation.rs:22465-22498) has no rate key, only CLOCK_SET/ADDED/PAUSED/RESUMED/QUERY_TIME/FAILED/NUM_TOOSMALL/TIMEMARKER_* and QUERY_DAY/DAYTIME/GAMETIME/SET/STOP. Exact missing generated field: a Bedrock commands.time.rate translation constant in crates/pumpkin-data/src/generated/translation.rs (generated data, off-limits, needs the Extractor). Kept the COMMANDS_TIME_SET fallback at time.rs:473-486.
- 7. Query day/daytime hardcoded minecraft:day: BLOCKED-DATA. The day/daytime/gametime queries are bound to the minecraft:day timeline in vanilla; ensure_day_timeline_clock (time.rs:186-) already reproduces that and the vanilla wrong_timeline_for_clock error (TimeCommand.java:42-43, 170-193). Arbitrary timelines need the timeline resource argument resolved via Registries.TIMELINE with suggestTimelines, the repetition subcommand and per-clock getCurrentTicks/getPeriodCount (TimeCommand.java:120-130, 142-151, 170-193). Exact missing subsystem: a command-side minecraft:timeline registry argument, suggestions, repetition query and per-timeline tick computation; Pumpkin only models minecraft:day. The Java query keys already exist (translation.rs:7601/7603), so only the resolution subsystem is missing.

No new pure test seam exists; existing wrap_time/builtin-marker tests are unchanged. No compile run (orchestrator builds all). No source file changed this pass (doc Result only).
