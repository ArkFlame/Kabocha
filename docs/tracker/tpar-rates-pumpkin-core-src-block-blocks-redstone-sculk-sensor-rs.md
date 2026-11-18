# Parity rows for crates/pumpkin-core/src/block/blocks/redstone/sculk_sensor.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **blocks/Sculk Sensor** [partial/AUDITED] Redstone signal output, calibrated directional filtering, and phase cooldown ticks are implemented, but World::post_game_event only notifies mob listeners and never triggers sculk sensors. 

## Result
- **blocks/Sculk Sensor** [partial/AUDITED]: VERIFIED (matches decompiled vanilla). No code change needed this pass: the dispatch fix described by this row is already present in the checkout (commit `feat(blocks): trigger sculk sensors from game events`). Verified the whole chain:
  - `world/game_event.rs:62-133` `World::post_game_event` now dispatches every posted game event to block entities exposing `BlockEntity::game_event_radius` (`block/entities/mod.rs:85-100`), in addition to mob listeners. Chunk scan bounds come from `GameEvent::notification_radius()` and each listener's own radius is re-checked, matching vanilla `net/minecraft/world/level/gameevent/GameEventDispatcher.java:22-75` (`post`) and `net/minecraft/world/level/gameevent/EuclideanGameEventListenerRegistry.java:115-124` (`getPostableListenerPosition`).
  - `block/entities/sculk_sensor.rs:55-68` and `block/entities/calibrated_sculk_sensor.rs:55-68` implement `game_event_radius` (8 / 16) and `on_game_event` -> `SculkSensorBlock::on_vibration`, matching vanilla `net/minecraft/world/level/block/entity/SculkSensorBlockEntity.java:87-139` (`VibrationUser.getListenerRadius` / `onReceiveVibration`) and `net/minecraft/world/level/block/entity/CalibratedSculkSensorBlockEntity.java:35-39`.
  - `block/blocks/redstone/sculk_sensor.rs:88-191` `on_vibration` + `trigger` activate the block, matching vanilla `net/minecraft/world/level/block/SculkSensorBlock.java:219-250` (`activate`). Occlusion (`world/game_event.rs:139-152`) matches `net/minecraft/world/level/gameevent/vibrations/VibrationSystem.java:270-283` (`Listener.isOccluded`).
- No generated data, WIT, or cross-crate refactor is required. No new pure unit test seam exists for the dispatch chain (it needs a live `World` with block entities); the existing pure tests in `sculk_sensor.rs:428-512` already cover power and directional-signal boundaries.

Status: PASS fixed=0 open=0 files=docs/tracker/tpar-rates-pumpkin-core-src-block-blocks-redstone-sculk-sensor-rs.md
