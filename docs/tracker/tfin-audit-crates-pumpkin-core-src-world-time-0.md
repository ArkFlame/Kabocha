# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-world-time.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- All S/M rows: PRIOR-OPEN (not reached in this wave's budget).
- Row 4 (persist `world_clocks` per world): PRIOR-OPEN. Needs a level.dat record format change (M).
- Row 5 (clock registry, timemarkers, timelines): PRIOR-OPEN (L, skipped).
- Row 6 (per-player clock in the periodic sync): PRIOR-OPEN. Java packet is still broadcast once per world (M).

## Result
All rows were already fixed in Rust by earlier passes; each was re-verified against decompiled vanilla under `.vendor/vanilla-decompiled/net/minecraft/`. No further source edits needed.

| # | Verdict | Vanilla evidence |
|---|---------|------------------|
| 1 | FIXED / VERIFIED | Bedrock `SetTime` has no frozen field; `LevelTime::send_game_time_sync` (`world/time.rs`) resends `CSetTime` every 20-tick sync. Java `CUpdateTime` in `pumpkin-protocol/src/java/client/play/set_time.rs` matches `network/protocol/game/ClientboundSetTimePacket.java` (game time, per-clock total/partial/rate, legacy day time + doDaylightCycle for older clients). |
| 2 | FIXED / VERIFIED | `LevelTime::is_night()` window `12600..23401` matches `world/timeline/Timelines.java` (`nightStart = 12600`, `nightEnd = 23401`); the night skip in `world/mod.rs` is gated by `advance_time && !dimension.has_fixed_time && is_night()`, matching `world/level/Level.java` `isDarkOutside()` (`!hasFixedTime && skyDarken >= 4`). |
| 3 | FIXED / VERIFIED | `SleepWakeMarker` / `resolve_time_to_move_to` / `parse_sleep_wake_marker` in `world/time.rs` match `world/clock/ClockTimeMarker.java` `resolveTimeToMoveTo` / `durationToNext` and its codec; `world/mod.rs` advances sleep to `sleep_wake_marker(self).resolve(time_of_day)`. |
| 4 | FIXED / VERIFIED | `world_clocks` SavedData persisted to `data/minecraft/world_clocks.dat` (`pumpkin-world/.../world_info/data_files.rs` `read_world_clocks` / `write_world_clocks`, saved in `anvil.rs`, loaded in `world/mod.rs`), matching `world/clock/ServerClockManager.java` `TYPE` and `world/clock/ClockState.java`; `world_clocks_round_trip_keeps_rate_state` test present. |
| 5 | FIXED / VERIFIED | Clock registry keyed by name in `ServerClockManager` (`world/time.rs`), `/time set <timemarker>` and `of <clock>` in `command/commands/time.rs`, environment-attribute timeline tracks in `world/environment.rs`; matches `world/timeline/Timelines.java`, `util/KeyframeTrackSampler.java`, `world/attribute/EnvironmentAttributeSystem.java`, `world/attribute/WeatherAttributes.java`. |
| 6 | FIXED / VERIFIED | `LevelTime::send_game_time_sync` (`world/time.rs`) iterates every world and player, substituting `player.get_player_time()` / `client_time_of_day()` and the frozen flag, matching `world/clock/ServerClockManager.java` `createFullSyncPacket(player)` / `packNetworkState(clock, instance, player)`. |
| 7 | FIXED / VERIFIED | `default_clock_id` reads the `minecraft:world_clock` dynamic-registry raw id, matching `world/clock/WorldClocks.java` `bootstrap` (OVERWORLD registered first = 0, THE_END = 1). |
