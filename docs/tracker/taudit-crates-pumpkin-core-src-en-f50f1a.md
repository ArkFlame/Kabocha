# Tracker: audit / entity/passive/axolotl.rs (AUDIT)

Audit of `crates/pumpkin-core/src/entity/passive/axolotl.rs`,
`.../axolotl/ai.rs` and `.../axolotl/controls.rs` against vanilla `Axolotl`
(`hurtServer`, `baseTick`, `travel`, `finalizeSpawn`, `updateActivity`,
`applySupportingEffects`, `onStopAttacking`, bucket tag) and the `PlayDead`
behavior. No `TODO`/`FIXME` left in these files: the tracker markers are the
`Vanilla ...` doc comments.

## Fixed in this pass

| Gap | Rust | Vanilla | Fix | Size |
|---|---|---|---|---|
| Play-dead roll used the health before the hit | `axolotl.rs:557` (`mob_hurt_server`) | `Axolotl.hurtServer` calls `super.hurtServer` first, then checks `getHealth()/getMaxHealth() < 0.5` and `amount < getHealth()` | the pre-damage hook now only records the raw `amount`; the roll runs in `on_damage_with_context`, after the damage is applied | S |
| Duplicated doc line over `apply_supporting_effects` | `axolotl.rs:183-184` | n/a | removed the repeated line | XS |

## Remaining gaps (not certain or not small)

| Gap | Rust | Vanilla | Suggested fix | Size |
|---|---|---|---|---|
| Regeneration renewal extends instead of adding a flat 100 ticks | `axolotl.rs:174-181` (`supporting_regeneration_duration`) | `Axolotl.applySupportingEffects` adds `MobEffectInstance(REGENERATION, 100, 0)` when absent or `duration < 2400 && amplifier < 1`; `MobEffectInstance.update` keeps the longer of the two, so 50 ticks becomes 100, not 150, and 2399 stays 2399 | confirm against decompiled `Axolotl`; if confirmed, return `Some(100)` instead of `BIFF + left` and drop the 2400 cap (the `duration < 2400` gate stays) | S |
| Support regen uses an expanded AABB instead of the vanilla distance | `axolotl.rs:321-334` (`on_stop_attacking`) | `source.getEntity() instanceof Player player && this.distanceToSqr(player) <= 400.0` | keep the killer-player lookup, replace the box test with `pos.squared_distance_to(&player.pos) <= 400.0` (about one block tighter at the edge) | S |
| Attacker check accepts a direct entity with no causing entity | `axolotl.rs:591` (`has_attacker: source.is_some() || cause.is_some()`) | `source.getEntity() != null` (causing entity only) | check which of Pumpkin's `source`/`cause` maps to `DamageSource.getEntity()` and use only that one | S |
| Play-dead roll is skipped when a hit deals no damage | `on_damage_with_context` runs only when `damaged` | vanilla evaluates the roll regardless of `super.hurtServer`'s return, so a blocked/invulnerable hit can still start play-dead | evaluate on the pre-damage hook for the not-damaged branch as well, or pass the raw amount through the post hook | M |

## Verified matching vanilla (no change)

- `getSpawnVariant`/`getCommonSpawnVariant`/`getRareSpawnVariant` pools and the
  `SpawnGroupData` pair (two common variants, group `>= 2` babies).
- `getBreedOffspring` variant roll (1/1200 blue, otherwise either parent) and
  `setPersistenceRequired`.
- `baseTick`: `handleAirSupply` only with AI (dry-out 2.0 at air `-20`, refill to
  6000 in water or rain), plain `LivingEntity` air rules with `NoAI`.
- `rehydrate` (+1800 capped at 6000), `isPushedByFluid = false`,
  `canBreatheUnderwater` via `minecraft:can_breathe_under_water`,
  `getWalkTargetValue = 0`, `removeWhenFarAway` and `requiresCustomPersistence`.
- `travel`: input speed, `0.9` drag, `-0.005` sink without a target.
- Bucket tag: variant component, `Age`/`AgeLocked`, `HuntingCooldown`
  (`setMemoryWithExpiry` on load, erased when absent).
- Brain: five sensors, `Core` (LookAt/MoveTo/ValidatePlayDead/CountDown),
  `Idle` (look at players, make love, temptation/baby follow, start attacking,
  `TryFindWater`, ordered stroll/trigger gate), `Fight` (stop-attacking callback,
  walk target, melee, breeding erase), `PlayDead` (`PLAY_DEAD_TICKS` +
  `HURT_BY_ENTITY`, 200-tick duration, walk/look target erased).
- `updateActivity` hold while `PlayDead`, hunting cooldown `2400` on leaving
  `Fight`, breeding suppresses attack targets.
- Movement/look controls freeze while playing dead; constants `(85, 10, 0.1F,
  0.5F, false)` and yaw-from-centre `20`.
- Ambient/attack/hurt/death/swim sounds come from `pumpkin-data` (water vs air
  idle sound chosen in code); `canBeSeenAsEnemy` false while playing dead.
- `mobInteract`: bucket pickup first, then `Animal` feeding/breeding; tropical
  fish bucket returns the water bucket.
