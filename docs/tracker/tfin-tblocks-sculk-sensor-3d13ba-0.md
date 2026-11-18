# Final rows from docs/tracker/tblocks-sculk-sensor-3d13ba.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: game_event.rs sculk_sensor.rs vibration.rs

- Rows 2-4: already resolved in the table above (FIXED / CONFIRMED-INTENTIONAL); not marked STILL OPEN, so out of scope for this pass.
- Row 1 (only STILL OPEN row): CONFIRMED FIXED, no code change needed. Independently re-checked decompiled `net/minecraft/world/level/gameevent/vibrations/VibrationSystem.java:270-283`: from/to are floor+0.5, each `Direction` nudges the source by 1e-5, and occlusion requires every line to cross an `OCCLUDES_VIBRATION_SIGNALS` block. Pumpkin `world/game_event.rs::is_occluded` (:139-152) matches. `SculkSensorBlock::on_vibration` (`block/blocks/redstone/sculk_sensor.rs:100`) calls it before `trigger`; import at :11. Allay uses the same helper (`entity/passive/allay/vibration.rs:15,187`). No pure unit-test seam exists (needs a live `World`); no generated data, WIT, or cross-crate refactor needed. No source files edited this pass.
- Rows 2-4: not marked STILL OPEN, out of scope.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
