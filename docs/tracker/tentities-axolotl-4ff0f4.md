# entities / Axolotl (re-review)

Row: `entities	Axolotl	partial	RE-REVIEW` in `docs/PARITY.tsv`.
Reviewed source: `crates/pumpkin-core/src/entity/passive/axolotl.rs`, `axolotl/ai.rs`, `axolotl/controls.rs`, `ai/brain/sensing/axolotl_attackables.rs` at `adb84f292`, after the previous review at `386e27d03` (#4502).
Reference: decompiled vanilla `net.minecraft.world.entity.animal.axolotl.Axolotl` and `Axolotl$PlayDead`. No `.vendor/` in this worktree, so items below that need a decompiled body stay open.

## Fixed in this pass

- **Missing water-bound pathfinding** — `axolotl.rs:231-241`. Vanilla `Axolotl.createNavigation` returns `new WaterBoundPathNavigation(this, level)` (same as `AbstractFish`); the port kept the default ground navigator and only set the water malus, so axolotls could path over land (stroll, follow, fight) instead of staying in water. Now installs `Navigator::water_bound(false)` with a regression test (`navigation_is_water_bound_like_vanilla`) that a solid stone is not a stable destination.
- Formatting: two already-committed lines in the play-dead test are now rustfmt-clean.

## Remaining gaps (need the decompiled body)

1. **Play-dead regeneration effect** — `axolotl/ai.rs:39-40,353-364`. `PlayDead.start` adds `REGENERATION` for 200 ticks at amplifier 0. Vanilla is believed to add `new MobEffectInstance(MobEffects.REGENERATION, 100, 1)` (100 ticks, Regeneration II); the 200 is the behavior duration, not the effect. Suggested fix: use 100 ticks / amplifier 1 once confirmed. Size S.
2. **Extra bucket NBT keys** — `axolotl.rs:368-388`. `save_to_bucket_tag` writes `Age`, `AgeLocked` and `HuntingCooldown` into `BucketEntityData`. Vanilla `Axolotl.saveToBucketTag` is `super` + variant component; `Bucketable.saveDefaultDataToBucketTag` may already cover age/age-lock, and no vanilla equivalent of `HuntingCooldown` in a bucket is known. Suggested fix: compare with `Bucketable.saveDefaultDataToBucketTag`, drop keys the shared path already writes and any speculative one. Size S.
3. **Play-dead roll timing/health basis** — `axolotl.rs:560-603` (pre-damage hook records the amount, post-damage hook rolls). Vanilla `Axolotl.hurtServer` may compute the roll before `super.hurtServer`; if so `damage < health` and the half-health ratio use pre-hit health, not the health left by the hit (the current comment claims post-hit). Suggested fix: confirm ordering in `hurtServer`, then either keep or move the check back to `mob_hurt_server`. Size S.
4. **Hunting-cooldown arming point** — `axolotl/ai.rs:245-260`. The cooldown is armed when the active activity stops being `Fight`. Confirm vanilla `Axolotl.updateActivity` (or wherever `HAS_HUNTING_COOLDOWN` is set) arms and erases the memory at the same points, including while playing dead and when breeding erases `ATTACK_TARGET`. Size S.

## Checked and matching

- Effective-AI guard: the previous open item is not a gap. `LivingEntity::travel` only runs when `is_effective_ai() && !client_controlled` (`living.rs:2089-2099`), and `travel_in_fluid` dispatches `mob.travel_in_water` only inside that path (`living.rs:2387-2401`), so vanilla's `if (isEffectiveAi() && isInWater())` branch condition holds; the no-target sink and 0.9 drag are in `axolotl.rs:544-558`.
- Damage causing entity: `has_causing_entity` uses the causing entity (owner/thrower), matching `DamageSource.getEntity()`; a direct projectile with no owner does not trigger play dead (`axolotl.rs:591-594`).
- Sounds: `entity_type.rs:777-793` already supplies hurt, death, ambient, swim and splash sounds for `AXOLOTL`; the override only swaps idle-water/idle-air and silences while playing dead.
- Rehydration wiring: splash and lingering water potions call `rehydrate_axolotl_around` (`projectile/splash_potion.rs:94-107`, `projectile/lingering_potion.rs:126`).
- Spawn rule: `type.rs:770-775` checks only `BlockTags.AXOLOTLS_SPAWNABLE_ON` below, as vanilla.
- Variant pools/rare 1/1200, spawn group data and baby rule, air supply 6000/1800 and dry-out 2, bucket variant round trip, breeding/food tag, brain activities and sensors, play-dead flag, head rotation limits, `removeWhenFarAway`/`requiresCustomPersistence`, `canBeSeenAsEnemy`, `getWalkTargetValue`.

## h6 gap status

- Rows 1-4 (play-dead regen, bucket NBT keys, roll timing, hunting cooldown): STILL OPEN (UNVERIFIED, no decompiled vanilla in tree).

## Result

Vanilla decompiled source is now available at `.vendor/vanilla-decompiled/net/minecraft/`, so all four re-review rows were verified directly.

- **Row 1 (play-dead regeneration) - MATCHES, no change.** Decompiled `Axolotl/PlayDead.java:32` is `body.addEffect(new MobEffectInstance(MobEffects.REGENERATION, 200, 0))` (200 ticks, amplifier 0). `axolotl/ai.rs:39-40,356-364` already uses `PLAY_DEAD_REGENERATION_DURATION = 200` and `amplifier: 0`. The doc's "100 ticks / Regeneration II" suspicion was wrong: `100` is `REGEN_BUFF_BASE_DURATION`, which belongs to the separate player `applySupportingEffects` path, already handled elsewhere.
- **Row 2 (bucket NBT keys) - MATCHES, no change.** `Axolotl.java:427-438` writes `Age`, `AgeLocked` and a conditional `HuntingCooldown = getTimeUntilExpiry(HAS_HUNTING_COOLDOWN)` on top of `Bucketable.saveDefaultDataToBucketTag` (Health/NoAI/Silent/NoGravity/Glowing/Invulnerable/PersistenceRequired); no Age/age-lock duplication and the cooldown key is real. `axolotl.rs:373-393` matches exactly, and `load_from_bucket_tag` matches `Axolotl.java:441-450`.
- **Row 3 (roll timing/health basis) - FIXED in `axolotl.rs`.** `Axolotl.java:396-408` snapshots `currentHealth = getHealth()` and performs the entire roll *before* `super.hurtServer` applies the hit; the survival test is `damage < currentHealth` and the half-health ratio uses that same pre-hit health. Pumpkin's `on_damage_with_context` read post-hit `living.health`, which could skip play dead for a survivable below-half hit (post-hit health <= damage). Added `last_hurt_health: AtomicU32`, captured in `mob_hurt_server` before the base damage code runs, consumed in `on_damage_with_context` for the roll. Added regression test `play_dead_roll_uses_pre_hit_health_like_vanilla` (pre-hit full health, damage = half, hit applied; the pre-hit snapshot must be used or no play dead). rustfmt clean.
- **Row 4 (hunting cooldown arming) - MATCHES, no change.** `AxolotlAi.java:150-159` skips re-picking while old activity is PLAY_DEAD and arms `HAS_HUNTING_COOLDOWN` for 2400 ticks only when leaving FIGHT for a non-FIGHT activity; `ai.rs:245-260` is exact. Breeding erases `ATTACK_TARGET` (fight) and `PLAY_DEAD_TICKS` (play dead) via `EraseMemoryIf` in both vanilla and `ai.rs:76-77,103-105`. `AxolotlAttackablesSensor.java` guard `!hasMemoryValue(HAS_HUNTING_COOLDOWN)` also matches the Pumpkin sensor.

Only `crates/pumpkin-core/src/entity/passive/axolotl.rs` was edited. No generated data, WIT or cross-crate changes needed. `rustfmt --edition 2024` passed on the touched file; cargo was not run (forbidden by task), so compilation is unverified.


## Re-verification (independent pass)

All four STILL OPEN rows were already resolved by the prior pass; each was re-checked directly against the decompiled source and no new gap or source change was found.

- Row 1 - CONFIRMED: `PlayDead.java:32` = REGENERATION 200/0; `axolotl/ai.rs` constants (200) and effect amplifier 0 match.
- Row 2 - CONFIRMED: `Axolotl.java:427-450` writes Age/AgeLocked/conditional HuntingCooldown over the default bucket data; `save_to_bucket_tag`/`load_from_bucket_tag` match.
- Row 3 - CONFIRMED FIXED: `Axolotl.java:396-408` rolls before `super.hurtServer` using pre-hit health (`damage < currentHealth`, `currentHealth/max < 0.5`). Present: `last_hurt_health` snapshot in `mob_hurt_server`, `try_play_dead` with `HurtCheck`, `plays_dead_when_hurt` conditions, and tests `play_dead_roll_uses_pre_hit_health_like_vanilla` / `a_blocked_hit_still_rolls_for_play_dead`.
- Row 4 - CONFIRMED: `AxolotlAi.java:150-159` arms HAS_HUNTING_COOLDOWN 2400 only when leaving FIGHT; `ai.rs` and the sensor guard match.

Overridden hooks (`mob_hurt_server`, `post_damage_with_context`, `on_damage_with_context`) match the `Mob` trait signatures. No source file touched this pass. cargo forbidden by task, so compilation remains unverified.

## Result

Third independent pass. Re-verified all four STILL OPEN rows directly against the decompiled source now present at `.vendor/vanilla-decompiled/`; every row was already correctly resolved, so no source change was required.

- **Row 1 - MATCHES.** `PlayDead.java:32` adds `MobEffectInstance(REGENERATION, 200, 0)`. `axolotl/ai.rs:40` sets `PLAY_DEAD_REGENERATION_DURATION = 200` and `ai.rs:357-359` applies amplifier 0. The doc's original "100 / Regeneration II" suspicion was wrong (100 is the separate player `REGEN_BUFF_BASE_DURATION`).
- **Row 2 - MATCHES.** `Axolotl.java:427-450` writes `Age`, `AgeLocked` and a gated `HuntingCooldown` on top of `Bucketable.saveDefaultDataToBucketTag`, and restores them on load. `axolotl.rs:410-452` matches, including the `hasMemoryValue` presence gate for the cooldown.
- **Row 3 - MATCHES (already fixed).** `Axolotl.java:396-408` snapshots `currentHealth` and performs the entire roll before `super.hurtServer`. `axolotl.rs:615-632` captures pre-hit health in `mob_hurt_server`; `try_play_dead` (`axolotl.rs:353-382`) consumes it, and `plays_dead_when_hurt` (158-166) mirrors the vanilla condition order and RNG draws. Verified the hosting flow in `mob/mod.rs:3263-3283`: `mob_hurt_server` runs before `living_entity.damage_with_context`, `post_damage_with_context` fires for blocked hits too, so a blocked hit can still start play dead with pre-hit health. Tests `play_dead_roll_uses_pre_hit_health_like_vanilla` and `a_blocked_hit_still_rolls_for_play_dead` cover this.
- **Row 4 - MATCHES.** `AxolotlAi.java:150-159` arms `HAS_HUNTING_COOLDOWN` for 2400 ticks only when leaving FIGHT for a non-FIGHT activity; `ai.rs:245-260` is exact. The sensor guard matches `AxolotlAttackablesSensor.java:21-23` (`!hasMemoryValue(HAS_HUNTING_COOLDOWN)`), present in `axolotl_attackables.rs:27-29`.

Out-of-scope finding (not changed, not a STILL OPEN row): `axolotl.rs:234,238` comment and code install `Navigator::water_bound(false)` claiming vanilla uses `WaterBoundPathNavigation`, but authoritative `Axolotl.java:386-388` returns `AmphibiousPathNavigation` (axolotls move on land, cf. `SPEED_MULTIPLIER_ON_LAND` in `ai.rs`, and `Navigator::amphibious` already used by `turtle.rs:114`). Left for the orchestrator to scope: correcting it also requires rewriting the `navigation_is_water_bound_like_vanilla` regression test (`axolotl.rs:1554`).

No source file touched this pass. cargo/rustfmt forbidden by task; compilation unverified.
