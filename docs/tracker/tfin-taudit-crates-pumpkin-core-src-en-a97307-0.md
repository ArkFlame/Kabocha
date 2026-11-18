# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-a97307.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

Independently re-checked every STILL OPEN row against .vendor/vanilla-decompiled/net/minecraft (authoritative dump).
Re-checked every STILL OPEN row against .vendor/vanilla-decompiled/net/minecraft (authoritative) and the current source files.
- Rows 1, 2, 3, 6: VERIFIED NO ACTION and row 4: FIXED, all as recorded in the Result sections above; nothing remains STILL OPEN.
Re-read every row against the authoritative decompiled sources at .vendor/vanilla-decompiled/net/minecraft and the current source files. All rows closed; none STILL OPEN.

## Result

Verified against decompiled vanilla at .vendor/vanilla-decompiled/net/minecraft (authoritative).

- Row 1 (baby spawn chance ignores difficulty): VERIFIED NO ACTION. AgeableMob.java:341-343 getBabySpawnChance() takes no LocalDifficulty and returns plain babySpawnChance; finalizeSpawn (AgeableMob.java:55-57) applies it with no difficulty multiplier. Pumpkin ageable.rs:156-161 matches.
- Row 2 (first group mob never rolls baby): VERIFIED NO ACTION. AgeableMob.java:55-57 keeps the getGroupSize() > 0 guard, so the first mob skips the baby roll. Pumpkin ageable.rs:156-161 group_size > 0 guard matches.
- Row 3 (AgeableMobGroupData lacks can_spawn_another): VERIFIED NO ACTION. AgeableMob.java:311-344 exposes only groupSize / increaseGroupSizeByOne / isShouldSpawnBaby / getBabySpawnChance, no cap method. Pumpkin group_size plus species caps matches.
- Row 4 (triangle draw precision): FIXED. spawn.rs:97 uses 0.114_85 * (rng.random::<f64>() - rng.random::<f64>()); vanilla Mob.java:1263 random.triangle(0.0, 0.11485) is the f64 overload (RandomSource.java:59-61: mean + spread * (nextDouble() - nextDouble())). Modifier.amount is f64, so the expression type-checks.
- Row 5 (equipment roll ignores spawn reason): VERIFIED NO ACTION. The EQUIPMENT_REGISTRY keys (skeleton, stray, bogged, wither_skeleton, piglin, pillager, vindicator) all roll equipment for every spawn reason in vanilla: AbstractSkeleton.java:134-137, WitherSkeleton.java:83-90 (via super), Pillager.java:157-163, Vindicator.java:126-134, Piglin.java:188-203. Only Zombie.java:514-517 gates equipment/loot on CONVERSION; zombies are not in the registry and Pumpkin finalize_zombie_spawn (zombie/mod.rs:256,285) already gates on reason != Conversion. No observable difference, no cross-crate refactor needed.
- Row 6 (Halloween constants): VERIFIED NO ACTION. Zombie.java:520-521 and AbstractSkeleton.java:140-141 confirm the 0.25 head roll and 0.1 jack-o'-lantern share; spawn.rs:20-22 constants (0.25, 0.1) match.

Files edited: crates/pumpkin-core/src/entity/mob/spawn.rs (added a pure unit test for EntitySpawnReason; rustfmt --edition 2024). No BLOCKED-DATA rows.
