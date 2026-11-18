# Final rows from docs/tracker/tentities-ocelot.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: ocelot.rs

## Result
- 1 distance gate (`ocelot.rs:162`): VERIFIED. `Ocelot.mobInteract` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/feline/Ocelot.java` :167-186) gates on `temptGoal` running, `!isTrusting()`, `isFood`, and `distanceToSqr < 9.0`; `ocelot.rs` `mob_interact` uses the same checks (`squared_distance_to_vec < 9.0`, `is_running::<TemptGoal>()`).
- 2 breeding trust inheritance: VERIFIED. Vanilla `getBreedOffspring` (:230-232) creates a fresh `EntityTypes.OCELOT` with no trust copy; Pumpkin uses the default `Animal::get_breed_offspring`, which also builds a new entity from the type.
- 3 sound table entries: VERIFIED. Ambient interval 900 (:152-154) matches `OCELOT_AMBIENT_SOUND_INTERVAL`. Hurt/death/ambient sounds (:147-164) match generated `entity_type.rs` OCELOT `hurt_sound`/`death_sound`/`ambient_sound`.
- 4 stepping-carefully override: VERIFIED. Vanilla `isCrouching() || super.isSteppingCarefully()` (:279-281) and `Entity.isSteppingCarefully` = `isShiftKeyDown()` (Entity.java:3936). Pumpkin `mob_is_stepping_carefully` (`pose == Crouching || is_sneaking()`) is the same.

Status: fixed=0 open=0. No compile run.
