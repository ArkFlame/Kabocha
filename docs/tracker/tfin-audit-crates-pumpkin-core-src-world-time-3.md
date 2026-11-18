# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-world-time.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: set_time.rs time.rs

All vanilla paths below are under `/run/media/linsaftw/SDD/GitHub/ArkFlame/Kabocha/.vendor/vanilla-decompiled/net/minecraft/`.

| 7 | VERIFIED | `default_clock_id` (`world/time.rs:24-38`) resolves the network clock id from the `minecraft:world_clock` dynamic registry via `get_dynamic_registry_raw_id` (used at `player.rs:4718` and `world/time.rs:281`); `world_clock` is loaded by `data/datapack/dynamic_registry_loader.rs:49`, so the old "absent from pumpkin-data" blocker is gone. Matches `world/clock/WorldClocks.java` `bootstrap` (OVERWORLD registered before THE_END = 1). `java/client/play/set_time.rs` `CUpdateTime` codec matches `network/protocol/game/ClientboundSetTimePacket.java`, `world/clock/ClockNetworkState.java` (VAR_LONG totalTicks, FLOAT partialTick, FLOAT rate) and `WorldClock.java` (`holderRegistry` = VarInt clock id). |
| 5 | VERIFIED | Clock registry and per-clock `ServerClockManager` live in `world/time.rs` (`server_clock_manager_keeps_clocks_independent`); `/time set <timemarker>` and `of <clock>` live in `command/commands/time.rs`; `SleepWakeMarker::resolve` matches `world/clock/ClockTimeMarker.java` `resolveTimeToMoveTo`. |
| 5 | VERIFIED | Environment-attribute timeline tracks in `world/environment.rs` match `world/timeline/Timelines.java`, `util/KeyframeTrackSampler.java`, `world/attribute/EnvironmentAttributeSystem.java` and `world/attribute/WeatherAttributes.java`; no missing generated field or subsystem remains. |

## Result
All three rows are closed as VERIFIED against decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/`. No source edits were needed this
pass: `default_clock_id` already replaced the hardcoded clock id with a
`minecraft:world_clock` dynamic-registry lookup, the clock registry / marker
resolver / environment tracks are all present and wired, and the `set_time.rs`
codec matches `ClientboundSetTimePacket`. No `STILL OPEN` rows remain, so
rustfmt was not needed.
