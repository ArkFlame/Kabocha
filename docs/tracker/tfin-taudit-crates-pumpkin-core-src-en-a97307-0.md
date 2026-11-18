# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-a97307.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files).

## Result (batch 09 pass, 2026-11-18)

Verified against .vendor/vanilla-decompiled/net/minecraft (authoritative).

- Row 1 (baby spawn chance ignores difficulty): VERIFIED NO ACTION. AgeableMob.java:341-343 and finalizeSpawn 55-57; ageable.rs:156-161 matches.
- Row 2 (first group mob never rolls baby): VERIFIED NO ACTION. AgeableMob.java:55-57 group size guard; ageable.rs:156-161 matches.
- Row 3 (AgeableMobGroupData lacks can_spawn_another): VERIFIED NO ACTION. AgeableMob.java:311-344 has no cap method.
- Row 4 (triangle draw precision): FIXED. spawn.rs:97 uses the f64 triangle form, matching Mob.java:1263 and RandomSource.java:59-61.
- Row 5 (equipment roll ignores spawn reason): VERIFIED NO ACTION. AbstractSkeleton.java:134-137, Pillager.java:157-163, Vindicator.java:126-134, Piglin.java:188-203 roll for every reason; only Zombie gates on CONVERSION, and zombie/mod.rs gates on reason.
- Row 6 (Halloween constants): VERIFIED NO ACTION. Zombie.java:520-521 and AbstractSkeleton.java:140-141 match spawn.rs:20-22.

Files edited earlier: crates/pumpkin-core/src/entity/mob/spawn.rs (pure EntitySpawnReason test). No BLOCKED-DATA rows.
