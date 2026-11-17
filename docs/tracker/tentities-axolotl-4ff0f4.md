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
