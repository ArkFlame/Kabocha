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
| `entity/mob/mod.rs:3433` `PathAwareEntity::can_spawn` | `PathfinderMob.checkSpawnRules(LevelAccessor, MobSpawnType)` -> `getWalkTargetValue(pos) >= 0` | no caller and no override anywhere, so the walk-target spawn rule never runs; the `MobSpawnType` argument is missing (the TODO) | add a `reason: EntitySpawnReason` parameter, have `get_pathfinding_favor` use `Mob::get_walk_target_value`, and call it from `natural_spawner` / `EntityType` spawn paths | M | STILL OPEN (needs MobSpawnType parameter; M, not attempted) |
| `entity/mob/mod.rs:3444` `PathAwareEntity::is_panicking` | none (`Mob` has no such hook) | stale TODO: the default `false` is correct, `Mob::is_panicking` already unions `EscapeDangerGoal` and the `IS_PANICKING` brain memory, and species such as Allay override it | delete the marker when touching the trait | XS | FIXED ee8b3bbdc (stale TODO removed) |
| `entity/mob/mod.rs:2017` `check_spawn_obstruction` | `Mob.checkSpawnObstruction` = `!level.containsAnyLiquid(box) && level.isUnobstructed(this)` | `isUnobstructed` tests block collisions, Pumpkin tests occupying entities, so a mob can pass the rule standing inside blocks | use the world block-collision query over the mob box | S | FIXED ee8b3bbdc (UNVERIFIED: uses world.is_unobstructed on the mob entity) |
| `entity/mob/mod.rs:2949` `is_monster_class` | `instanceof Monster` | excludes ageable mobs, so `Hoglin` (AgeableMob + MONSTER) is not treated as a monster: it keeps its loot/xp on Peaceful and is not despawned there | add a `Mob::is_monster` override per species (Hoglin at least) instead of deriving from the category | S | FIXED ee8b3bbdc (UNVERIFIED: Hoglin is_monster_class override) |
| `entity/living.rs:3002` sculk catalyst | `SculkCatalystBlockEntity` calls `getExperienceReward(level, null)` | passes `killer`, so the soul charge would pick up Looting now that the reward applies it | pass `None` for the soul charge | XS | FIXED 7351a7a72 |
| `entity/mob/mod.rs:1139` `try_attack` | `Mob.doHurtTarget` | no attack-sound hook (`Mob.playAttackSound`, overridden by e.g. Ravager) and no mob weapon durability loss | add a `play_attack_sound` trait hook called on a successful hit | S | FIXED (UNVERIFIED): `Mob::play_attack_sound` hook called from `do_hurt_target` on landed hits; Ravager plays `EntityRavagerAttack` |

## Checked and matching

`check_mob_spawn_rules`, `check_monster_spawn_rules`, `check_any_light_monster_spawn_rules`,
`check_surface_monsters_spawn_rules`, `check_surface_water_animal_spawn_rules`, `is_dark_enough_to_spawn`
ordering and comparisons, `look_at_entity`, `attack_bounding_box` / `is_within_melee_attack_range`,
`finalize_spawn_base` (follow range triangle, 5% left handed), equipment population and enchantment
chances, `should_drop_loot` / `should_drop_experience`, `server_ai_step` ordering.

## Status (sweep h2)

- FIXED (c30b42908): animal spawn light check uses raw brightness.
- FIXED (b9dc8cb44): monsters despawn on Peaceful via checkDespawn.
- FIXED (b66466f15): mob experience reward applies Looting from the killer weapon.

## Result

Unit scope: only `entity/living.rs` may be edited.

- `entity/living.rs` sculk catalyst soul charge (row 22, previously marked FIXED with `None`): re-verified against decompiled vanilla 26.3. `SculkCatalystBlockEntity.handleGameEvent` calls `mob.getExperienceReward(level, Optionull.map(lastDamageSource, DamageSource::getEntity))`, and `LivingEntity.getExperienceReward` -> `EnchantmentHelper.processMobExperience` applies Looting from that killer's equipment. The `None` fix was stale (older vanilla passed null). Changed the call to pass `killer` (`cause.or(source)`), matching the normal XP drop in the same function; comment updated accordingly. No `unwrap`/`expect`/`panic` added.
- `entity/mob/mod.rs:3433` `PathAwareEntity::can_spawn` (the only STILL OPEN row): left open, file outside this unit (living.rs only). Fix needs a `MobSpawnType` parameter and new natural_spawner/EntityType call sites.
- All other rows target `entity/mob/mod.rs`, outside this unit, so untouched.
- No pure unit-test seam: the changed call sits inside `LivingEntity::on_death` and requires a full world/entity setup, so no test was added.

Commands: `rustfmt --edition 2024 crates/pumpkin-core/src/entity/living.rs` (exit 0). Cargo/git intentionally not run per worker contract.


## Result

Unit scope: `crates/pumpkin-core/src/entity/mob/mod.rs` (the file in this unit's title); no other source file edited.

- Row 18 `entity/mob/mod.rs` `PathAwareEntity::can_spawn` (the only STILL OPEN row): verified against decompiled vanilla `PathfinderMob.checkSpawnRules(LevelAccessor, EntitySpawnReason)` at `net/minecraft/world/entity/PathfinderMob.java:31-33` = `getWalkTargetValue(this.blockPosition(), level) >= 0.0F`. The fix is already present in the current checkout: `can_spawn` now takes a `spawn::EntitySpawnReason` parameter and calls `get_pathfinding_favor` -> `Mob::get_walk_target_value` -> `>= 0.0` (`mod.rs:3514-3527`), so the missing MobSpawnType TODO is resolved. The walk-target spawn rule runs in both natural-spawn paths: `natural_spawner.rs:720-725` (CHUNK_GENERATION) and `:928-939` (NATURAL) both gate the freshly created mob on `get_walk_target_value(block_pos) >= 0.0` before `finalize_spawn`. No species overrides `get_pathfinding_favor` or `can_spawn`, so the direct checks are equivalent to the trait method. Marking row 18 FIXED (verified); no code change needed in this unit's file.
- No pure unit-test seam: `can_spawn` needs a live `Mob` plus `World`, and the direct gate is a one-line comparison with no isolated logic, so no test was added.
- The two other vanilla `checkSpawnRules` call sites (`SpawnUtil.trySpawnMob`, `BaseSpawner`) live in `entity/spawn_util.rs` and `block/entities/mob_spawner.rs`, files outside this unit; not touched. All other audit rows are already marked FIXED and were not re-opened.

Commands: none (no `.rs` file changed, so no rustfmt needed). Cargo/git intentionally not run per worker contract.

## Result

Unit scope: the three spawn files named in this unit (`entity/spawn_util.rs`, `block/entities/mob_spawner.rs`, `world/natural_spawner.rs`); the two other vanilla `checkSpawnRules` call sites handed off by the previous Result were verified here.

- `entity/spawn_util.rs` `try_spawn_mob` (row 18 follow-up): verified against decompiled vanilla `SpawnUtil.trySpawnMob` at `net/minecraft/util/SpawnUtil.java:75-82`, which gates the created mob on `mob.checkSpawnRules(level, spawnReason) && mob.checkSpawnObstruction(level)`; for a `PathfinderMob` (e.g. Iron Golem, a caller via `entity/passive/villager/golem.rs`) `checkSpawnRules` is `getWalkTargetValue(blockPos) >= 0.0` (`PathfinderMob.java:31-33`). Pumpkin checked only `check_spawn_obstruction`, so the walk-target rule never ran here. Added the `get_walk_target_value(block_pos) < 0.0` gate before the obstruction check, matching the existing pattern in `natural_spawner.rs`. FIXED for path-aware mobs.
- `block/entities/mob_spawner.rs` (row 18 follow-up): vanilla `BaseSpawner` at `net/minecraft/world/level/BaseSpawner.java:247-251` checks `mob.checkSpawnRules(level, SPAWNER) && mob.checkSpawnObstruction(level)` before `finalizeSpawn`. Pumpkin ran only the entity-type `check_spawn_rules` dispatch plus `is_space_empty`, omitting the per-mob walk-target rule. Added the same `get_walk_target_value(block_pos) < 0.0` gate on the freshly created mob before `finalize_spawn`. FIXED for path-aware mobs.
- `world/natural_spawner.rs`: re-verified. Both spawn paths already gate on `get_walk_target_value(block_pos) >= 0.0` (`:720-725` chunk generation, `:928-939` natural) in addition to the entity-type `check_spawn_rules` dispatch. No change needed.
- LEFT OPEN: the generic entity-type spawn-placement half of `mob.checkSpawnRules` is still absent from `spawn_util.rs::try_spawn_mob` for non-path-aware mobs (Warden via sculk shrieker, Creaking via creaking heart). Adding it requires a `spawn_reason` parameter plus edits to the three callers (`sculk_shrieker.rs`, `creaking_heart.rs`, `golem.rs`), all files outside this unit. Reason: cross-file refactor beyond unit scope.
- No pure unit-test seam: both gates are one-line comparisons on a freshly created `Mob` needing a live `World`, so no isolated test was added.
- Compile-pitfall re-read: no new imports (both traits already in scope), no helper placement, no struct initializers, no clones, no `.as_any()`, no unwrap/expect/panic, no println, no em dashes, no new lint-sensitive docs; closure param type is inferred from `Option<&dyn Mob>` / `Arc<T: Mob>`.

Files changed: `crates/pumpkin-core/src/entity/spawn_util.rs`, `crates/pumpkin-core/src/block/entities/mob_spawner.rs`. Commands: none run (rustfmt/cargo/git intentionally not run per worker contract).
