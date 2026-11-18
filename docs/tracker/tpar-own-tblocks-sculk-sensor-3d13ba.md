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


## Result (final pass)

All four rows of the base audit (`docs/tracker/tblocks-sculk-sensor-3d13ba.md`) were
re-verified directly against decompiled vanilla in
`.vendor/vanilla-decompiled/net/minecraft/` and against the current Pumpkin source.
Every gap described in the table is already fixed and matches vanilla, so no source
file was edited this pass. No row remains STILL OPEN and no row needs generated data.

- Row 1 (occlusion): FIXED, VERIFIED. `net/minecraft/world/level/gameevent/vibrations/VibrationSystem.java:270-283` (`Listener.isOccluded`) floors+centers both endpoints (`Mth.floor(x)+0.5`), nudges the source by `1.0E-5` along each `Direction`, and returns true only if every one of the six lines hits an `OCCLUDES_VIBRATION_SIGNALS` block (early false otherwise). Pumpkin `world/game_event.rs::is_occluded` (:139-152) is line-for-line equivalent; `SculkSensorBlock::on_vibration` (`sculk_sensor.rs:100`) calls it before `trigger`, and the allay listener shares the helper.
- Row 2 (comparator output gate): FIXED, VERIFIED. `net/minecraft/world/level/block/SculkSensorBlock.java:289-295` (`getAnalogOutputSignal`) returns `getPhase(state) == ACTIVE ? sculk.getLastVibrationFrequency() : 0`. Pumpkin `get_comparator_output` (`sculk_sensor.rs:309-336`) gates on `SculkSensorPhase::Active` identically. The phase gate is intentional and correct, not a bug.
- Row 3 (listener radii): FIXED, VERIFIED. `net/minecraft/world/level/block/entity/CalibratedSculkSensorBlockEntity.java:36-39` returns 16 and `net/minecraft/world/level/block/entity/SculkSensorBlockEntity.java:97-99` returns 8. Pumpkin declares `CALIBRATED_LISTENER_RADIUS=16` / `LISTENER_RADIUS=8` (`sculk_sensor.rs:34,36`) and `on_vibration` selects by block id (:103-107).
- Row 4 (power formula): FIXED, VERIFIED. `net/minecraft/world/level/gameevent/vibrations/VibrationSystem.java:118-121` (`getRedstoneStrengthForDistance`) computes `max(1, 15 - floor((15.0/R)*distance))`. Pumpkin `get_power` (`sculk_sensor.rs:83-86`) is equivalent (`clamp(15 - floor(15*d/R), 1, 15)`), used by `on_vibration` (:114) and covered by the pure unit test at :432-438.

No generated `pumpkin-data` fields, WIT changes, or cross-crate refactors are required. No `.rs` file was edited, so no rustfmt was needed.
