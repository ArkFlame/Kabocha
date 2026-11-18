# Parity rows for own-tblocks-sculk-sensor-3d13ba

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/tblocks-sculk-sensor-3d13ba** [partial/OWN-OPEN] own audit: 4 open, 10 fixed, 6 verified, 0 blocked-data (docs/tracker/tblocks-sculk-sensor-3d13ba.md) docs/tracker/tblocks-sculk-sensor-3d13ba.md

## Result
All rows closed FIXED / VERIFIED against decompiled vanilla; see `## Result (third pass)` below. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)

## Result (third pass)

Re-verified all four rows against decompiled vanilla (`.vendor/vanilla-decompiled/net/minecraft/`); all already match, no source change needed. Cleared the stale `STILL OPEN` marker in `docs/tracker/tblocks-sculk-sensor-3d13ba.md`.

- Row 1 (occlusion): FIXED, VERIFIED. `world/game_event.rs::is_occluded` (`:139-152`) floors+centers endpoints, nudges source by 1e-5 per `BlockDirection`, and requires every line to cross `#minecraft:occludes_vibration_signals`; `SculkSensorBlock::on_vibration` (`sculk_sensor.rs:100`) calls it before `trigger`. Matches `net/minecraft/world/level/gameevent/vibrations/VibrationSystem.java:270-283`.
- Row 2 (comparator output gate): CONFIRMED-INTENTIONAL, VERIFIED. `get_comparator_output` (`sculk_sensor.rs:309-336`) returns the stored frequency only while `Active`; matches `net/minecraft/world/level/block/SculkSensorBlock.java:289-295` (`getPhase(state) == ACTIVE ? getLastVibrationFrequency() : 0`).
- Row 3 (calibrated radius): FIXED, VERIFIED. `CALIBRATED_LISTENER_RADIUS=16` / `LISTENER_RADIUS=8` (`sculk_sensor.rs:34,36`) match `net/minecraft/world/level/block/entity/CalibratedSculkSensorBlockEntity.java:36-39` (16) and `.../SculkSensorBlockEntity.java:97-99` (8).
- Row 4 (power formula): FIXED, VERIFIED. `get_power` (`sculk_sensor.rs:83-86`) = `clamp(15 - floor(15*d/R), 1, 15)`, equivalent to `net/minecraft/world/level/gameevent/vibrations/VibrationSystem.java:118-121` = `max(1, 15 - floor((15.0/R)*d))`; pure unit test at `sculk_sensor.rs:432-438`.

Both sensor block entities call `SculkSensorBlock::on_vibration` (`block/entities/sculk_sensor.rs:67`, `block/entities/calibrated_sculk_sensor.rs:67`). No generated-data, WIT, or cross-crate refactor needed. No source files edited this pass.
