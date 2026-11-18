# Audit: crates/pumpkin-core/src/entity/mob/spawn.rs

Row: audit / crates/pumpkin-core/src/entity/mob/spawn.rs (AUDIT).
Note: the "3 TODO/FIXME/vanilla markers" are false positives - the file has no TODO,
FIXME, todo! or unimplemented!. The grep hits are doc comments naming vanilla classes
(lines 31, 103, 215).

## Verified as matching vanilla (no action)

- SpawnGroupData variants mirror vanilla implementors: AgeableMob.AgeableMobGroupData,
  SchoolSpawnGroupData, TropicalFishGroupData, Spider.SpiderGroupData, llama/horse/rabbit/fox
  group data, ZombieGroupData, AxolotlGroupData (spawn.rs:32-72).
- EntitySpawnReason has all 19 vanilla values, isSpawner() = SPAWNER|TRIAL_SPAWNER,
  ignoresLightRequirements() = TRIAL_SPAWNER only (spawn.rs:156-188; Monster.checkSpawnRules).
- finalize_spawn_base order: follow-range minecraft:random_spawn_bonus (ADD_MULTIPLIED_BASE),
  5% left-handed, then per-type equipment (spawn.rs:76-100; Mob.finalizeSpawn).
- EQUIPMENT_REGISTRY keys are bare resource_name (EntityType::resource_name is unprefixed),
  so the lookup at spawn.rs:107 does hit (generated entity_type.rs:277+).
- Zombie already applies baby_speed_boost in its own set_baby (zombie/mod.rs:199-215), so
  set_baby_flag/set_baby_by_age need no speed modifier.
- Halloween 25% head roll split into 0.1 jack o'lantern matches the 2.5% / 22.5% wiki numbers
  and the upstream #4267 fix; not re-verified against decompiled Zombie/AbstractSkeleton.

## Remaining gaps (larger than a single in-file edit)

1. ageable.rs:156-161 - baby spawn chance ignores difficulty. Vanilla
   AgeableMob.AgeableMobGroupData.getBabySpawnChance(LocalDifficulty) returns
   shouldSpawnBaby ? babySpawnChance * difficulty.getSpecialMultiplier() : 0. Pumpkin rolls
   random() <= baby_spawn_chance with no difficulty factor, and finalize_ageable_spawn has no
   RegionalDifficulty argument (ageable.rs:137; caller mod.rs:2570). Fix: thread
   RegionalDifficulty through default_finalize_spawn/finalize_spawn_super/finalize_ageable_spawn
   and multiply. Size: M (signature reaches every finalize_spawn override).
2. ageable.rs:156-161 - first mob of a group never rolls baby. Vanilla creates
   new AgeableMobGroupData(true) for the first mob and rolls its baby chance too; the
   group_size > 0 guard skips that roll. Fix: drop the guard once (1) is in place so the size
   is incremented after the roll. Size: S, needs a vanilla re-read to confirm ordering.
3. spawn.rs:33-37 - AgeableMobGroupData lacks can_spawn_another. Vanilla caps a group via
   canSpawnAnother/isMaxGroupSizeReached (AgeableMobGroupData.increaseGroupSize), used by
   spawners; Pumpkin stores only group_size and each species reimplements the cap. Size: M
   (touch every group-data consumer: llama/horse/rabbit/fox/axolotl spawners).
4. spawn.rs:93 - triangle draw uses f32. Vanilla RandomSource.triangle(0.0, 0.11485000171139836)
   is center + (nextDouble() - nextDouble()) * spread; Pumpkin uses 0.114_85 * (f32 - f32).
   Distribution matches, only quantisation/precision differs. Size: XS; only worth changing if a
   differential RNG test against vanilla is added.
5. spawn.rs:105-118 - base equipment roll runs for every spawn reason. Vanilla gates some
   per-class rolls on the reason (zombies skip equipment and loot pickup on CONVERSION). The
   registry path has no reason argument, so registry mobs (skeleton/stray/bogged/piglin/pillager/
   vindicator) get equipment on CONVERSION/LOAD; no registry mob is converted in-tree today, so
   no observable difference. Size: S - pass EntitySpawnReason into populate_spawn_equipment and
   gate on Conversion|Load. Confirm in decompiled AbstractSkeleton.finalizeSpawn first.
6. spawn.rs:20-22 - Halloween constants unverified against decompiled source. Confirmed against
   wiki numbers only; re-check Zombie.finalizeSpawn/AbstractSkeleton.finalizeSpawn when a
   decompiled dump is available.

## Gap status
- 1 baby spawn chance ignores difficulty (ageable.rs:156-161): STILL OPEN, M, signature change across finalize overrides.
- 2 first group mob never rolls baby: STILL OPEN, S, depends on 1 and vanilla re-read.
- 3 AgeableMobGroupData lacks can_spawn_another: STILL OPEN, M.
- 4 triangle draw precision (spawn.rs:93): STILL OPEN, XS, no differential RNG test.
- 5 equipment roll ignores spawn reason (spawn.rs:105-118): STILL OPEN, S, needs decompiled AbstractSkeleton.
- 6 Halloween constants (spawn.rs:20-22): BLOCKED-DATA, no decompiled Zombie/AbstractSkeleton tree.


## Result

Verified against decompiled vanilla at .vendor/vanilla-decompiled/net/minecraft (authoritative; the doc's stated vanilla signatures did not match this dump).

- Row 1 (baby spawn chance ignores difficulty): VERIFIED NO ACTION. Decompiled AgeableMob.java:341-343 `getBabySpawnChance()` takes no LocalDifficulty and returns plain `babySpawnChance`; `finalizeSpawn` (lines 55-57) applies it with no difficulty multiplier. Pumpkin ageable.rs:156-161 matches.
- Row 2 (first group mob never rolls baby): VERIFIED NO ACTION. Decompiled AgeableMob.java:56 keeps the `getGroupSize() > 0` guard, so the first mob also skips the baby roll. Pumpkin `group_size > 0` guard matches.
- Row 3 (AgeableMobGroupData lacks can_spawn_another): VERIFIED NO ACTION. Decompiled AgeableMob.java:311-344 has no cap method; `increaseGroupSizeByOne()` only increments, and no `getMaxGroupSizeReached`/`canSpawnAnother` exists in this dump. Pumpkin's group_size plus species caps matches.
- Row 4 (triangle draw precision): FIXED. spawn.rs:94 now uses `rng.random::<f64>() - rng.random::<f64>()`, matching vanilla `Mob.finalizeSpawn`'s `RandomSource.triangle(0.0, 0.11485)` f64 overload (`mean + spread * (nextDouble() - nextDouble())`, Mob.java:1263, RandomSource.java:59-61).
- Row 5 (equipment roll ignores spawn reason): left open. Fix needs EntitySpawnReason threaded through finalize_spawn_base/populate_spawn_equipment and touches every finalize_spawn caller outside this unit; vanilla AbstractSkeleton.finalizeSpawn (lines 134-143) rolls equipment for all reasons and no registry mob is converted in-tree, so no observable difference today.
- Row 6 (Halloween constants): VERIFIED NO ACTION. Decompiled Zombie.java:520-521 and AbstractSkeleton.java:140-141 confirm 0.25 head roll and 0.1 jack-o'-lantern share; spawn.rs:20-22 constants match.

Files edited: crates/pumpkin-core/src/entity/mob/spawn.rs (rustfmt --edition 2024 passed, exit 0). No tests added: the change is a one-line RNG type swap with no pure seam (finalize_spawn_base needs a live World/attribute set).

## Result (re-verification, this pass)

Independently re-checked every STILL OPEN row against .vendor/vanilla-decompiled/net/minecraft (authoritative dump).

- Row 1 (baby spawn chance ignores difficulty): VERIFIED NO ACTION. AgeableMob.java:341-343 `getBabySpawnChance()` takes no LocalDifficulty and returns plain `babySpawnChance`; finalizeSpawn (AgeableMob.java:47-62) applies it with no difficulty multiplier. Pumpkin ageable.rs:156-161 matches.
- Row 2 (first group mob never rolls baby): VERIFIED NO ACTION. AgeableMob.java:55-57 keeps the `getGroupSize() > 0` guard, so the first mob also skips the baby roll. Pumpkin `group_size > 0` guard matches.
- Row 3 (AgeableMobGroupData lacks can_spawn_another): VERIFIED NO ACTION. AgeableMob.java:311-344 has only groupSize / increaseGroupSizeByOne / getBabySpawnChance / isShouldSpawnBaby, no cap method. Pumpkin group_size plus species caps matches.
- Row 4 (triangle draw precision): FIXED (fix already present from prior pass, re-confirmed here). spawn.rs:97 uses `rng.random::<f64>() - rng.random::<f64>()`; vanilla Mob.java:1263 `triangle(0.0, 0.11485)` and RandomSource.java:59-61 f64 overload match. `Modifier.amount` is f64 (attributes.rs:51), so the expression type-checks.
- Row 5 (equipment roll ignores spawn reason): LEFT OPEN (one-line reason). Needs EntitySpawnReason threaded through finalize_spawn_base / populate_spawn_equipment and every finalize_spawn caller outside this unit, a cross-crate refactor. No observable difference today: no registry mob is converted in-tree per prior AbstractSkeleton.finalizeSpawn check.
- Row 6 (Halloween constants): VERIFIED NO ACTION (prior Zombie.java / AbstractSkeleton.java verification stands).

No .rs files edited this pass: the row 4 fix was already in place. rustfmt --edition 2024 re-run on spawn.rs, exit 0. No pure test seam for the RNG type swap (finalize_spawn_base needs a live World and attribute set); did not add a formula-mirroring test.

## Result (independent re-verification, this pass)

Re-checked every STILL OPEN row against .vendor/vanilla-decompiled/net/minecraft (authoritative) and the current source files.

- Row 1 (baby spawn chance ignores difficulty): VERIFIED NO ACTION. AgeableMob.java:341-343 getBabySpawnChance() takes no LocalDifficulty and returns plain babySpawnChance; finalizeSpawn (AgeableMob.java:47-62) applies it with no difficulty multiplier. Pumpkin ageable.rs matches.
- Row 2 (first group mob never rolls baby): VERIFIED NO ACTION. AgeableMob.java:55-57 keeps the getGroupSize() > 0 guard, so the first mob skips the baby roll. Pumpkin group_size > 0 guard matches.
- Row 3 (AgeableMobGroupData lacks can_spawn_another): VERIFIED NO ACTION. AgeableMob.java:311-344 exposes only groupSize / increaseGroupSizeByOne / getBabySpawnChance / isShouldSpawnBaby, no cap method. Pumpkin group_size plus species caps matches.
- Row 4 (triangle draw precision): FIXED, fix present and confirmed. spawn.rs:97 uses rng.random::<f64>() - rng.random::<f64>(); vanilla Mob.java:1263 random.triangle(0.0, 0.11485) and RandomSource.java:59-61 f64 overload mean + spread * (nextDouble() - nextDouble()) match. Modifier.amount is f64 (attributes.rs:51), so the expression type-checks.
- Row 5 (equipment roll ignores spawn reason): LEFT OPEN, one-line reason. Needs EntitySpawnReason threaded through finalize_spawn_base / populate_spawn_equipment and every finalize_spawn caller outside this unit (cross-crate refactor). AbstractSkeleton.java:134-137 rolls equipment for all reasons; only Zombie.java:479,514 gates equipment/loot on CONVERSION, and zombies are handled by their own finalize_spawn rather than this registry path, so no observable difference today.
- Row 6 (Halloween constants): VERIFIED NO ACTION. Zombie.java:520-521 and AbstractSkeleton.java:140-141 confirm 0.25 head roll and 0.1 jack-o'-lantern share; spawn.rs:20-22 constants match.

No .rs files edited this pass (all actionable rows resolved earlier or verified NO ACTION). No pure test seam for the RNG swap, and no formula-mirroring test added. Checked row 4 against the Worker compile pitfalls table: f64 literal 0.114_85 and f64 RNG draw match Modifier.amount type, no unwrap/expect/panic, no import or macro-path changes.
