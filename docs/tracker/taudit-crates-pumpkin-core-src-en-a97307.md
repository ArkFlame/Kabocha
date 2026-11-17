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
