# Parity rows for crates/pumpkin-core/src/world/time.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-wo-12a966** [partial/OWN-OPEN] own audit: 6 open, 0 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-wo-12a966.md) docs/tracker/taudit-crates-pumpkin-core-src-wo-12a966.md

## Result
All 9 rows of the referenced audit are terminal (FIXED or VERIFIED); no row is left open and no BLOCKED-DATA applies. Verified directly against decompiled vanilla at .vendor/vanilla-decompiled/net/minecraft/ and the current crates/pumpkin-core/src/world/time.rs:

- Row 1 (Bedrock frozen clock) - VERIFIED. ServerClockInstance.packNetworkState forces rate 0 when paused or advance_time is off (world/clock/ServerClockManager.java:284-287). LevelTime::send_time (time.rs:270-289) re-sends the constant query_daytime() day time via CSetTime on the every-20-tick sync (server/mod.rs:2064-2075, no paused guard), re-pinning a frozen Bedrock sky. Bedrock CSetTime has no rate field.
- Row 2 (night window) - VERIFIED. LevelTime::is_night (time.rs:373-380) uses nightStart = 12600, nightEnd = 23401, matching world/timeline/Timelines.java:41-42. The skip-night gate in world/mod.rs requires advance_time && !has_fixed_time.
- Row 3 (sleep-wake marker) - FIXED. resolve_time_to_move_to, SleepWakeMarker::resolve and parse_sleep_wake_marker (time.rs:40-118) mirror ClockTimeMarker.resolveTimeToMoveTo / durationToNext (world/clock/ClockTimeMarker.java:20-36). Marker 0 and period 24000 match Timelines.java:47-52 and generated DayTimeline.
- Row 4 (per-world persistence) - FIXED. ClockInstance::load_from and ServerClockManager (time.rs:120-216) round-trip total/partial/rate/paused; world/mod.rs:620-638 loads them from world_clocks.dat, matching vanilla PackedClockStates / ClockState / world_clocks SavedData (world/clock/PackedClockStates.java, ServerClockManager.java:24-59).
- Row 5 (independent clocks) - FIXED. One ClockInstance per world_clock id (ServerClockManager.clocks, time.rs:193-216), matching ServerClockManager.java:34,48-51.
- Row 6 (per-player override) - FIXED. send_game_time_sync (time.rs:304-332) substitutes each player's get_player_time() / client_time_of_day(), matching the getPlayerTime branch of packNetworkState (ServerClockManager.java:203-213).
- Row 7 (clock registry id) - VERIFIED. default_clock_id (time.rs:21-38) resolves the id from the minecraft:world_clock dynamic registry (Registries.WORLD_CLOCK, ServerClockManager.java:48-51).
- Row 8 (Bedrock SetTime field) - FIXED. All three broadcast paths send day time via query_daytime(); CSetTime.time is in-game day time (time.rs:287,328; pumpkin-protocol bedrock/client/set_time.rs:11-12).
- Row 9 (add_time clamp) - VERIFIED (no code change). LevelTime::add_time (time.rs:334-336) uses .max(0), matching addTicks Math.max(instance.totalTicks + ticks, 0L) (ServerClockManager.java:146-148).

No generated pumpkin-data field, WIT change, or cross-crate refactor is needed. No production source was modified this pass (all fixes are already present in time.rs); the existing unit tests cover the clamp, night window, marker resolution, per-clock independence and frozen-clock packing. rustfmt: no source lines touched this pass.
