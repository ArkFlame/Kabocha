# Parity rows for crates/pumpkin-core/src/entity/mob/spawn.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-en-a97307** [partial/OWN-OPEN] own audit: 7 open, 3 fixed, 12 verified, 1 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-en-a97307.md) docs/tracker/taudit-crates-pumpkin-core-src-en-a97307.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)


## Result (third pass, final)

All rows closed against the authoritative decompiled dump at .vendor/vanilla-decompiled/net/minecraft/.

- Row 1 (baby spawn chance ignores difficulty): VERIFIED. AgeableMob.java:341-343 getBabySpawnChance() takes no LocalDifficulty and returns plain babySpawnChance; finalizeSpawn (AgeableMob.java:47-62) applies it with no difficulty multiplier. Pumpkin ageable.rs matches.
- Row 2 (first group mob never rolls baby): VERIFIED. AgeableMob.java:55-57 keeps the getGroupSize() > 0 guard, so the first mob skips the baby roll. Pumpkin group_size > 0 guard matches.
- Row 3 (AgeableMobGroupData lacks can_spawn_another): VERIFIED. AgeableMob.java:311-344 exposes only groupSize / increaseGroupSizeByOne / getBabySpawnChance / isShouldSpawnBaby, no cap method. Pumpkin group_size plus species caps matches.
- Row 4 (triangle draw precision): FIXED (fix already present, re-confirmed). spawn.rs:97 uses rng.random::<f64>() - rng.random::<f64>(), matching vanilla Mob.finalizeSpawn f64 RandomSource.triangle(0.0, 0.11485) (Mob.java:1263, RandomSource.java:59-61). Modifier.amount is f64, so it type-checks.
- Row 5 (equipment roll ignores spawn reason): VERIFIED NO ACTION (prior BLOCKED-L status was incorrect). The registry path (spawn.rs:109-123) applies only to skeleton / stray / bogged / wither_skeleton / piglin / pillager / vindicator, and every one of these rolls equipment unconditionally with respect to spawn reason in vanilla: AbstractSkeleton.java:134-137, WitherSkeleton.java:86 (inherits AbstractSkeleton), Pillager.java:161-162, Vindicator.java:132-133, Piglin.java:201-202 (only the baby/weapon branch is STRUCTURE-gated, never equipment). The sole vanilla class that gates equipment on CONVERSION is Zombie.java:514-517; zombies are not in EQUIPMENT_REGISTRY and use their own finalize_spawn, whose finalize_zombie_spawn in zombie/mod.rs:285-288 already gates equipment on reason != Conversion. No observable difference and no cross-crate refactor required.
- Row 6 (Halloween constants): VERIFIED. Zombie.java:520-521 and AbstractSkeleton.java:140-141 confirm the 0.25 head roll and 0.1 jack-o-lantern share; spawn.rs:20-22 constants match.

No .rs files edited this pass: row 4 fix was already in place and row 5 is VERIFIED as matching vanilla. No pure test seam (finalize_spawn_base needs a live World/attribute set; every registry mob reason handling is vanilla-correct), so no formula-mirroring test was added. Checked against the Worker compile pitfalls table: no import/macro/struct changes, no unwrap/expect/panic, no em dashes.
