# Final rows from docs/tracker/tentities-fox-195403.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: mod.rs

- FINAL PASS: unit complete. Sole row mod.rs:877 (get_ambient_sound) VERIFIED against .vendor/vanilla-decompiled/net/minecraft/world/entity/animal/fox/Fox.java:688-702 (gate !isBrightOutside() && nextFloat()<0.1F; spectator-excluding nearby-player check; screech volume/pitch 2.0F/pitch) matching mod.rs:871-902 (!is_bright_outside() && random_range(0..10)==0; spectator-excluding get_nearest_player; 2.0/pitch). Proposed isDay() fix rests on an incorrect premise and is not applied. No STILL OPEN rows; no editable source files (basenames: none); no code, test or rustfmt changes.

## Result
- Sole row mod.rs:877 (get_ambient_sound): VERIFIED (matches vanilla). Decompiled .vendor/vanilla-decompiled/net/minecraft/world/entity/animal/fox/Fox.java:688-702 gates the night screech on !level().isBrightOutside() && random.nextFloat() < 0.1F, checks nearby players with EntitySelector.NO_SPECTATORS, and plays FOX_SCREECH at volume 2.0F / voice pitch (Fox.java:678-685). Rust mod.rs:871-903 matches: !world.is_bright_outside() && rand::rng().random_range(0..10) == 0 (10%), spectator-excluding get_nearest_player, and screech at 2.0 / get_pitch(). The proposed isDay() change rests on an incorrect premise and is not applied.
- All rows closed: zero FIXED code changes, zero BLOCKED-DATA, zero STILL OPEN. No editable source files in this unit (basenames: none), so no code/test edits and no rustfmt needed this pass.
