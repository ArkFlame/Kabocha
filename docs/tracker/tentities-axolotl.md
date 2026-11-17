> Superseded for gap tracking by `docs/tracker/tentities-axolotl-4ff0f4.md` (re-review).

# entities / Axolotl

Row: `entities	Axolotl	partial	RE-REVIEW` in `docs/PARITY.tsv`.
Source re-reviewed at `386e27d03` (`feat(entity): vanilla parity for axolotl`, #4502).
Reference: decompiled vanilla `net.minecraft.world.entity.animal.axolotl.Axolotl` (no `.vendor/` in this worktree, method signatures checked against the 1.21 / 1.21.6 mappings index).

## Fixed in this pass

- `travel_in_water` now applies vanilla `Axolotl.travel`'s no-target sink (`deltaMovement.y -= 0.005` after the `* 0.9` drag). `crates/pumpkin-core/src/entity/passive/axolotl.rs:541`.

## Remaining gaps (unverified, need the decompiled body)

1. **No effective-AI guard in travel** — `crates/pumpkin-core/src/entity/passive/axolotl.rs:541`
   Vanilla `Axolotl.travel` only uses the axolotl water movement `if (isEffectiveAi() && isInWater())`, otherwise `super.travel`. Here `travel_in_water` (called through `mob_travel_in_fluid`, see `crates/pumpkin-core/src/entity/living.rs:2351`) has no `is_effective_ai()` check, and the unused `gravity`/`falling` parameters are exactly what a `LivingEntity::travel_in_water` fallback needs.
   Fix: `if !self.is_effective_ai() { self.mob_entity.living_entity.travel_in_water(caller, gravity, falling); return; }`. Size S.

2. **Play-dead regeneration constants** — `crates/pumpkin-core/src/entity/passive/axolotl/ai.rs:39-40,353-364`
   Port grants `REGENERATION` for 200 ticks at amplifier 0. Vanilla `Axolotl$PlayDead.start` is believed to use `new MobEffectInstance(MobEffects.REGENERATION, 100, 1)` (100 ticks, Regeneration II). Confirm against 1.21.x source; if so change duration/amplifier. Size S.

3. **No-AI air path** — `crates/pumpkin-core/src/entity/passive/axolotl.rs:520-529`
   `base_tick` sends no-AI axolotls to `LivingEntity::tick_air_supply` and AI axolotls to `handle_air_supply`, on the claim that vanilla wraps `handleAirSupply` in `!isNoAi()`. Verify `Axolotl.baseTick` / `handleAirSupply(ServerLevel, int)` (1.21.6 signature) and the resulting land refill `. Size S.

4. **Hurt play-dead condition** — `crates/pumpkin-core/src/entity/passive/axolotl.rs:158-166`
   Uses `(roll/3 < damage || health/max < 0.5) && damage < health && in_water && has_attacker`. Confirm the OR between the damage roll and the half-health check, and the `damage < health` ordering, against `Axolotl.hurtServer`. Size S.

5. **Extra bucket NBT keys** — `crates/pumpkin-core/src/entity/passive/axolotl.rs:363-401`
   `save_to_bucket_tag` writes `Age`, `AgeLocked` and `HuntingCooldown`; vanilla `saveToBucketTag` is `super.saveToBucketTag` plus the `axolotl/variant` component. Check whether these keys are really in `Bucketable.saveDefaultDataToBucketTag` (1.21.x) or were added speculatively; if speculative, drop keys 2-3. Size S.

6. **Duplicated doc comment** — `crates/pumpkin-core/src/entity/passive/axolotl.rs:183-184`
   `apply_supporting_effects` doc line is repeated (second copy lacks the final period). Cosmetic. Size S.

## Checked and matching

Variant pools and rare 1/1200 roll, group-data spawning/baby rule, air supply 6000/1800 and dry-out damage 2, bucket round trip, breeding/food tag, play-dead/attack/fight brain activities, hunting cooldown memory, `onStopAttacking` killer-in-20-blocks regen, supporting-effects duration math (100 per axolotl, cap 2400), head rotation limits, hurt/death/ambient/swim sounds (from `entity_type` data plus overrides), leash (axolotls are leashable), `removeWhenFarAway`, `requiresCustomPersistence` (call site ORs `persistence_required`), `canBeSeenAsEnemy`.
