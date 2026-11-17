# Audit: crates/pumpkin-core/src/entity/mob/spawn.rs

Category: audit. Vanilla class: `net.minecraft.world.entity.Mob#finalizeSpawn`,
`MobSpawnType`, `SpawnGroupData`, `AgeableMob.AgeableMobGroupData`.

## Markers
No TODO/FIXME in the file. The 3 tracker markers are the word "vanilla" in the doc
comments at `spawn.rs:19`, `spawn.rs:91`, `spawn.rs:200`; all three describe vanilla
behaviour correctly.

## Verified against vanilla (no change needed)
- `finalize_spawn_base` (`spawn.rs:64`): follow-range modifier `minecraft:random_spawn_bonus`,
  amount `triangle(0, 0.11485)` with `ADD_MULTIPLIED_BASE`, then left-handedness at 5%,
  then equipment; RNG draw order matches `Mob.finalizeSpawn`.
- `EntitySpawnReason` (`spawn.rs:141`): all 19 `MobSpawnType` constants in vanilla order;
  `is_spawner` (`SPAWNER | TRIAL_SPAWNER`) and `ignores_light_requirements`
  (`TRIAL_SPAWNER`) match.
- `SpawnGroupData::Ageable` handling in `ageable.rs:137`: `group_size > 0` and
  `random <= baby_spawn_chance` match `AgeableMobGroupData`.
- `set_baby_by_age` / `set_baby_flag`: -24000 age, synced baby key, dimension refresh.

## Fixed in this pass
- `52acc5146` pickup loot was rolled twice for skeleton/stray/bogged (once in
  `Mob::default_finalize_spawn`, once in `MobEntity::populate_spawn_equipment`); now a
  single roll in `spawn.rs` using `MobEntity::MAX_PICKUP_LOOT_CHANCE`.
- `e88d651f3` spider/skeleton jockey was finalized with the spider's spawn reason;
  vanilla `Spider.finalizeSpawn` passes `MobSpawnType.JOCKEY`.

## Remaining gaps
1. `spawn.rs:72-84` skips the follow-range modifier when one already exists; vanilla
   `addPermanentModifier` removes and re-adds it (reroll). Only observable if
   `finalizeSpawn` runs twice on one mob, which no current caller does. Size S, low.
2. Skeleton Halloween head is missing. Vanilla `AbstractSkeleton.finalizeSpawn`, after
   `setCanPickUpLoot` and before equipment: if the head slot is empty and the date is
   Oct 31, 25% chance of a carved pumpkin (10% jack o'lantern), head drop chance 0.
   Pumpkin only does this for zombies (`zombie/mod.rs:283-307`, `is_halloween` private at
   `zombie/mod.rs:665`). Fix: move `is_halloween` to a shared helper and run it in
   `MobEntity::populate_spawn_equipment` for the skeleton family before `equip_mob_on_spawn`.
   Size S/M.
3. Dead generic equipment path: `SkeletonEntityBase::populate_default_equipment_slots`
   (`skeleton/mod.rs:126`) and `populate_default_equipment_enchantments`
   (`skeleton/mod.rs:169`) are never called; skeleton gear comes from
   `EQUIPMENT_REGISTRY` via `spawn.rs`. Either delete them or route
   `default_finalize_spawn` through them for parity with the zombie family. Size S.
