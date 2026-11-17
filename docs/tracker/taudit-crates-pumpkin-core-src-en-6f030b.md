# Audit: crates/pumpkin-core/src/entity/mob/mod.rs

Coverage AUDIT. Fixed in this branch, then the gaps left open.

## Fixed

| Gap | Vanilla | Fix |
|---|---|---|
| `is_bright_enough_to_spawn` (was line 1121) used `get_max_local_raw_brightness`, i.e. `getRawBrightness(pos, skyDarken)` | `Mob.isBrightEnoughToSpawn` = `level.getRawBrightness(pos, 0) > 8` | `get_raw_brightness(pos, 0) > 8`; animals now spawn at night/rain like vanilla |
| `check_despawn` (line 1290) never handled Peaceful | `Mob.checkDespawn` discards when `difficulty == PEACEFUL && shouldDespawnInPeaceful()` (`Enemy` returns true) | peaceful branch before persistence/distance checks, using `is_monster_class()` |
| `get_experience_reward` (line ~3406) ignored the killer weapon | `Mob.getExperienceReward` runs the attacker's weapon through the mob experience effects (Looting) | `EnchantmentHelper::modify_mob_experience` on the killer's held item, clamped at 0 |
| `// TODO` at old line 1050 in `is_dark_enough_to_spawn` | vanilla takes the spawner's `RandomSource` | stale: a fresh `Xoroshiro` seed per call has the same distribution; comment now says so |

## Open gaps

| File:line | Vanilla class | Gap | Suggested fix | Size |
|---|---|---|---|---|
| `entity/mob/mod.rs:3433` `PathAwareEntity::can_spawn` | `PathfinderMob.checkSpawnRules(LevelAccessor, MobSpawnType)` -> `getWalkTargetValue(pos) >= 0` | no caller and no override anywhere, so the walk-target spawn rule never runs; the `MobSpawnType` argument is missing (the TODO) | add a `reason: EntitySpawnReason` parameter, have `get_pathfinding_favor` use `Mob::get_walk_target_value`, and call it from `natural_spawner` / `EntityType` spawn paths | M |
| `entity/mob/mod.rs:3444` `PathAwareEntity::is_panicking` | none (`Mob` has no such hook) | stale TODO: the default `false` is correct, `Mob::is_panicking` already unions `EscapeDangerGoal` and the `IS_PANICKING` brain memory, and species such as Allay override it | delete the marker when touching the trait | XS |
| `entity/mob/mod.rs:2017` `check_spawn_obstruction` | `Mob.checkSpawnObstruction` = `!level.containsAnyLiquid(box) && level.isUnobstructed(this)` | `isUnobstructed` tests block collisions, Pumpkin tests occupying entities, so a mob can pass the rule standing inside blocks | use the world block-collision query over the mob box | S |
| `entity/mob/mod.rs:2949` `is_monster_class` | `instanceof Monster` | excludes ageable mobs, so `Hoglin` (AgeableMob + MONSTER) is not treated as a monster: it keeps its loot/xp on Peaceful and is not despawned there | add a `Mob::is_monster` override per species (Hoglin at least) instead of deriving from the category | S |
| `entity/living.rs:3002` sculk catalyst | `SculkCatalystBlockEntity` calls `getExperienceReward(level, null)` | passes `killer`, so the soul charge would pick up Looting now that the reward applies it | pass `None` for the soul charge | XS |
| `entity/mob/mod.rs:1139` `try_attack` | `Mob.doHurtTarget` | no attack-sound hook (`Mob.playAttackSound`, overridden by e.g. Ravager) and no mob weapon durability loss | add a `play_attack_sound` trait hook called on a successful hit | S |

## Checked and matching

`check_mob_spawn_rules`, `check_monster_spawn_rules`, `check_any_light_monster_spawn_rules`,
`check_surface_monsters_spawn_rules`, `check_surface_water_animal_spawn_rules`, `is_dark_enough_to_spawn`
ordering and comparisons, `look_at_entity`, `attack_bounding_box` / `is_within_melee_attack_range`,
`finalize_spawn_base` (follow range triangle, 5% left handed), equipment population and enchantment
chances, `should_drop_loot` / `should_drop_experience`, `server_ai_step` ordering.
