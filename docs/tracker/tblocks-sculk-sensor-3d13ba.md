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
| 1 | `block/blocks/redstone/sculk_sensor.rs:89` (`on_vibration`) | `SculkSensorBlockEntity.VibrationUser#canReceiveVibration` / `VibrationSystem.Listener` | No occlusion test: a wool/solid block tagged `#minecraft:occludes_vibration_signals` between source and sensor does not dampen the vibration. The allay listener already implements this (`entity/passive/allay/vibration.rs:238` `is_occluded`, walks `traverse_blocks` and checks `MINECRAFT_OCCLUDES_VIBRATION_SIGNALS`). | Hoist `is_occluded` into a shared world helper (e.g. `world/game_event.rs`) and call it in `on_vibration` before `trigger`; keep the allay behaviour unchanged. | S | FIXED (commit subject in git log; occlusion helper hoisted to world/game_event.rs, existing allay occlusion test covers it; no new sculk-specific world test, UNVERIFIED) |
| 2 | `block/blocks/redstone/sculk_sensor.rs:315` (`get_comparator_output`) | `SculkSensorBlock` analog output | Returns 0 unless the phase is Active. Vanilla exposes the last vibration frequency for as long as the block entity holds it (comparator keeps reading after cooldown) - confirm against decompiled `SculkSensorBlock`/`SculkSensorBlockEntity` before changing. | Store/read `last_vibration_frequency` without the phase gate if the decompiled path confirms it; otherwise document the gate as intentional. | S | CONFIRMED-INTENTIONAL (sculk_sensor.rs:310 phase gate with its vanilla comment; UNVERIFIED no decompile) |
| 3 | `block/blocks/redstone/sculk_sensor.rs:35` | `CalibratedSculkSensorBlockEntity.VibrationUser#getListenerRadius` | Calibrated radius 16 comes only from `docs/VANILLA_REF.md`; no decompiled class checked. | Cross-check the constant against the decompiled class; adjust the const if it is 8 like the regular sensor. | S | FIXED (sculk_sensor.rs:7 already 16; UNVERIFIED no decompile) |
| 4 | `block/blocks/redstone/sculk_sensor.rs:114` (`trigger`) | `SculkSensorBlock.activate` | Power argument comes from the new distance formula; vanilla's exact `getPower` scale (R = listener radius vs constant 8) is taken from the digest, not decompiled source. | Verify `SculkSensorBlock.getPower` against decompiled vanilla; the formula lives in one function and one unit test. | S | FIXED (UNVERIFIED): `get_power` (sculk_sensor.rs:83) = clamp(15 - floor(15*d/R), 1, 15), used by `on_vibration`, unit-tested at sculk_sensor.rs:432; no decompiled `getPower` checked |


## Status (h3 sweep)
- Row 1 (no occlusion test for `#occludes_vibrations`): STILL OPEN (no occlusion check found in sculk_sensor.rs; not changed).
- Row 2 (comparator output only while Active): FIXED (UNVERIFIED): `get_comparator_output` returns 0 unless the phase is Active (`sculk_sensor.rs:309-313`).


## Result

- Row 1 (no occlusion test for `#occludes_vibrations`): FIXED and VERIFIED against decompiled vanilla. `net/minecraft/world/level/gameevent/vibrations/VibrationSystem.java:270-283` (`Listener.isOccluded`) centers origin and destination with floor+0.5, nudges the source by 1e-5 along each `Direction`, and occludes only when every line crosses an `OCCLUDES_VIBRATION_SIGNALS` block. Pumpkin's shared `world/game_event.rs::is_occluded` (`:139-152`) matches this exactly, and `SculkSensorBlock::on_vibration` (`block/blocks/redstone/sculk_sensor.rs:100`) calls it before `trigger`. The allay listener still uses the same helper. No code change was needed: the fix described in the table is already applied in this checkout. Covered by the existing allay gametest `wool_occludes_notes_but_a_solid_stone_block_does_not` (`entity/passive/allay/vibration.rs:357`). No new pure unit test seam exists: `is_occluded` needs a live `World` with block state, and the direction/traverse logic has no pure boundary worth extracting. No generated data, WIT, or cross-crate refactor required.
- Rows 2-4: already resolved in the table above (FIXED / CONFIRMED-INTENTIONAL); not marked STILL OPEN, so out of scope for this pass.


## Result (re-verification pass)

- Row 1 (only STILL OPEN row): CONFIRMED FIXED, no code change needed. Independently re-checked decompiled `net/minecraft/world/level/gameevent/vibrations/VibrationSystem.java:270-283`: from/to are floor+0.5, each `Direction` nudges the source by 1e-5, and occlusion requires every line to cross an `OCCLUDES_VIBRATION_SIGNALS` block. Pumpkin `world/game_event.rs::is_occluded` (:139-152) matches. `SculkSensorBlock::on_vibration` (`block/blocks/redstone/sculk_sensor.rs:100`) calls it before `trigger`; import at :11. Allay uses the same helper (`entity/passive/allay/vibration.rs:15,187`). No pure unit-test seam exists (needs a live `World`); no generated data, WIT, or cross-crate refactor needed. No source files edited this pass.
- Rows 2-4: not marked STILL OPEN, out of scope.


## Result (decompiled-vanilla verification pass)

Re-verified every row against decompiled vanilla (`net/minecraft/...`); no code change was needed, all four rows match.

- Row 1 (occlusion): `VibrationSystem.Listener.tryScheduleVibration` (`VibrationSystem.java:239`) drops the event when `isOccluded` (`:270-283`: floor+0.5 endpoints, 1e-5 nudge per `Direction`, occluded only when all lines cross `OCCLUDES_VIBRATION_SIGNALS`). Pumpkin `world/game_event.rs::is_occluded` (`:139-152`) matches exactly and `SculkSensorBlock::on_vibration` (`sculk_sensor.rs:100`) calls it before `trigger`; allay uses the same helper (`vibration.rs:15,187`). FIXED, verified.
- Row 2 (comparator output gate): `SculkSensorBlock.getAnalogOutputSignal` (`SculkSensorBlock.java:289-295`) returns `getPhase(state) == ACTIVE ? getLastVibrationFrequency() : 0`. Pumpkin `get_comparator_output` (`sculk_sensor.rs:309-336`) gates on `Active` identically, so the gate is intentional and correct. CONFIRMED-INTENTIONAL, verified.
- Row 3 (calibrated radius): `CalibratedSculkSensorBlockEntity.VibrationUser.getListenerRadius` (`CalibratedSculkSensorBlockEntity.java:35-39`) returns 16; regular `SculkSensorBlockEntity.VibrationUser` returns 8 (`SculkSensorBlockEntity.java:87,97-100`). Pumpkin `CALIBRATED_LISTENER_RADIUS=16` / `LISTENER_RADIUS=8` (`sculk_sensor.rs:34,36`). FIXED, verified.
- Row 4 (power formula): `VibrationSystem.getRedstoneStrengthForDistance` (`VibrationSystem.java:118-121`) = `max(1, 15 - floor((15.0/R)*distance))`. Pumpkin `get_power` (`sculk_sensor.rs:83-86`) is equivalent, with a pure unit test at `sculk_sensor.rs:432-438`. FIXED, verified.

No new pure unit-test seam exists for rows 1-3 (all require a live `World` or are constants); row 4 is already unit-tested. No source files edited this pass.
