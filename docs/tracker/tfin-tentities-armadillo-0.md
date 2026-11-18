# Final rows from docs/tracker/tentities-armadillo.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- 2 dead `can_stay_rolled_up` branch: resolved.
- 4 redundant baby flag: resolved.

## Result
- Row 1 (scare with no caller): FIXED. `custom_server_ai_step` now runs the 5-tick scan
  (`SCARE_SCAN_INTERVAL`) while `can_stay_rolled_up()`, including while already rolled
  (vanilla `MobSensor` at `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/sensing/SensorType.java:26-27`,
  `ai/sensing/MobSensor.java:33-38`). A hit sets `danger_detected_recently_ticks` to 80 before `roll_up`.
  Source: `crates/pumpkin-core/src/entity/passive/armadillo.rs`.
- Row 2 (hurt counter): VERIFIED. `on_damage` stores 80 once for a living attacker and rolls up only if
  `can_stay_rolled_up()`, matching `Armadillo.actuallyHurt` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/armadillo/Armadillo.java:290-305`).
- Row 3 (shell damage reduction): VERIFIED. `hurtServer` (`Armadillo.java:280-285`) halves damage whenever `isScared()`, not only while hidden; Pumpkin matches.
- Row 4 (redundant baby flag): VERIFIED. `mob_init_data_tracker` only syncs the state; baby flag comes from `AgeableMob`.
- Row 5 (brush drop save semantics): VERIFIED. Brush and shed drops both use `world.spawn_entity` (vanilla `spawnAtLocation`, `Armadillo.java:144,324`).
- Row 6 (state ids): VERIFIED. Existing unit test `state_ids_and_names_match_vanilla_order` covers ids.
- BLOCKED-DATA: none. Not compiled (worker restriction); rustfmt not run (orchestrator).
