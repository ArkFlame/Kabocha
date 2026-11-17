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
| Play-dead roll accepted any direct entity | `axolotl.rs:591` (`has_causing_entity`) | `Axolotl.hurtServer` requires `DamageSource.getEntity() != null`, i.e. the causing entity (owner/thrower), not `getDirectEntity()` | the check is now `cause.is_some()`; the local damage path (`living.rs:4200`) documents `cause` as `DamageSource#getEntity()`; the existing test now asserts an unowned direct entity leaves the memory alone | S |
| Play-dead roll used the health before the hit | `axolotl.rs:560` (`mob_hurt_server`) | `Axolotl.hurtServer` calls `super.hurtServer` first, then checks `getHealth()/getMaxHealth() < 0.5` and `amount < getHealth()` | the pre-damage hook only records the raw `amount`; the roll runs in `on_damage_with_context`, after the damage is applied | S |
| Duplicated doc line over `apply_supporting_effects` | `axolotl.rs:183-184` | n/a | removed the repeated line | XS |

## Remaining gaps (not certain or not small)

| Gap | Rust | Vanilla | Suggested fix | Size |
|---|---|---|---|---|
| Support regen uses an expanded AABB instead of a distance | `axolotl.rs:321-334` (`on_stop_attacking`) | likely `source.getEntity() instanceof Player player && distanceToSqr(axolotl) <= 400.0` | keep the killer-player lookup, add `pos.squared_distance_to_vec(&player.pos) <= 400.0` (the cube corners reach ~34 blocks today; the 20-block sphere is a subset of the cube, so the filter only tightens) | S |
| Play-dead roll is skipped when a hit deals no damage | `on_damage_with_context` runs only when `damaged` (`mob/mod.rs:3209`) | vanilla evaluates the roll regardless of `super.hurtServer`'s return, so a blocked/invulnerable hit can still start play-dead | evaluate on `post_damage_with_context` for the not-damaged branch, or pass the raw amount through the post hook | M |
| Play-dead regeneration duration | `ai.rs:39,353-364` grants 200 ticks at amplifier 0 | wiki states Regeneration I while playing dead; duration is not documented in code we can read here | keep amplifier 0 (wiki: Regeneration I); only shorten to 100 ticks if 1.21.x source confirms | S |

## Checked this pass (no change)

- Regeneration renewal: wiki (`minecraft.wiki/Axolotl`) states 100 ticks per
  axolotl in the fight, capped at 2400, so adding 100 to the remaining duration
  (`supporting_regeneration_duration`) is correct; a flat 100 would not stack and
  contradicts the wiki. Amplifier > 0 is still left alone.
- No-AI travel: `travel_in_water` is only reached through
  `LivingEntity::tick` -> `travel`, already gated on `is_effective_ai()`
  (`living.rs:2089`), so the missing guard in the override is not observable.
- Bucket tag: `Bucketable::save_default_data_to_bucket_tag` (`bucketable.rs:23`)
  writes no `Age`/`AgeLocked`, so the axolotl-specific keys are needed;
  `HuntingCooldown` set/erase matches `saveToBucketTag`/`loadFromBucketTag`.
- Damage-source mapping: melee paths (`player.rs:1806`, `mob/mod.rs:1151`) pass
  the attacker as both `source` and `cause`, so the causing-entity check keeps
  melee hits working.

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
