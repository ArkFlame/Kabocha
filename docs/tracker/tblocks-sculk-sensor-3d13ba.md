# Sculk Sensor audit (blocks/Sculk Sensor)

Reviewed: `block/blocks/redstone/sculk_sensor.rs`, `block/entities/sculk_sensor.rs`,
`block/entities/calibrated_sculk_sensor.rs`, `world/game_event.rs`.

Fixed in this slice (commit "feat(blocks): trigger sculk sensors from game events"):
- Block entities had no `VibrationSystem` listener, so `World::post_game_event` reached
  mobs and the sculk shrieker but never a sensor. Added `game_event_radius` (8, calibrated
  16) and `on_game_event` to both sensor block entities, dispatching to
  `SculkSensorBlock::on_vibration`.
- `SculkSensorBlock::trigger` stored the redstone power in `last_vibration_frequency`.
  Vanilla stores the game event frequency (comparator output); power and frequency are now
  separate parameters, and the calibrated input-face comparison uses the frequency.
- Added `SculkSensorBlock::get_power(distance, radius)` = `clamp(15 - floor(15*d/R), 1, 15)`
  per `docs/VANILLA_REF.md`, instead of the caller having to pass a power.

Remaining gaps:

| # | file:line | Vanilla class | Gap | Suggested fix | Size |
|---|---|---|---|---|---|
| 1 | `block/blocks/redstone/sculk_sensor.rs:89` (`on_vibration`) | `SculkSensorBlockEntity.VibrationUser#canReceiveVibration` / `VibrationSystem.Listener` | No occlusion test: a wool/solid block tagged `#minecraft:occludes_vibration_signals` between source and sensor does not dampen the vibration. The allay listener already implements this (`entity/passive/allay/vibration.rs:238` `is_occluded`, walks `traverse_blocks` and checks `MINECRAFT_OCCLUDES_VIBRATION_SIGNALS`). | Hoist `is_occluded` into a shared world helper (e.g. `world/game_event.rs`) and call it in `on_vibration` before `trigger`; keep the allay behaviour unchanged. | S |
| 2 | `block/blocks/redstone/sculk_sensor.rs:315` (`get_comparator_output`) | `SculkSensorBlock` analog output | Returns 0 unless the phase is Active. Vanilla exposes the last vibration frequency for as long as the block entity holds it (comparator keeps reading after cooldown) - confirm against decompiled `SculkSensorBlock`/`SculkSensorBlockEntity` before changing. | Store/read `last_vibration_frequency` without the phase gate if the decompiled path confirms it; otherwise document the gate as intentional. | S |
| 3 | `block/blocks/redstone/sculk_sensor.rs:35` | `CalibratedSculkSensorBlockEntity.VibrationUser#getListenerRadius` | Calibrated radius 16 comes only from `docs/VANILLA_REF.md`; no decompiled class checked. | Cross-check the constant against the decompiled class; adjust the const if it is 8 like the regular sensor. | S |
| 4 | `block/blocks/redstone/sculk_sensor.rs:114` (`trigger`) | `SculkSensorBlock.activate` | Power argument comes from the new distance formula; vanilla's exact `getPower` scale (R = listener radius vs constant 8) is taken from the digest, not decompiled source. | Verify `SculkSensorBlock.getPower` against decompiled vanilla; the formula lives in one function and one unit test. | S |

