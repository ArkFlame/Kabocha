# entities / Axolotl (re-review)

Row: `entities	Axolotl	partial	RE-REVIEW` in `docs/PARITY.tsv`.
Re-reviewed on branch `kab/tentities-axolotl-4ff0f4` at `9be38b98d`.
Reference: decompiled vanilla `net.minecraft.world.entity.animal.axolotl.Axolotl` (no `.vendor/` in
this worktree; behaviour checked against the vanilla class bodies as known for 1.21.x).

Source now lives in `crates/pumpkin-core/src/entity/passive/axolotl.rs` and
`crates/pumpkin-core/src/entity/passive/axolotl/ai.rs` (the `axolotl/ai.rs` references in the old
`docs/tracker/tentities-axolotl.md` are current again; that file's line numbers were stale).

## Closed as non-gaps in this pass

1. **Effective-AI guard in `travel`** - `crates/pumpkin-core/src/entity/passive/axolotl.rs:541`
   Vanilla `Entity.isEffectiveAi()` is `!level.isClientSide`, which is always true on the server, so
   `Axolotl.travel`'s water branch runs for every server-side axolotl in water. `travel_in_water` is
   only reached from `LivingEntity::travel_in_fluid` with `water == true`
   (`crates/pumpkin-core/src/entity/living.rs:2362-2366`), which already mirrors `isInWater()`.
   No change needed; do not add a `is_effective_ai()` fallback (it would only create a dead branch).
2. **Play-dead regeneration constants** - `crates/pumpkin-core/src/entity/passive/axolotl/ai.rs:353-364`
   Vanilla `Axolotl$PlayDead.start` grants Regeneration for 200 ticks at amplifier 0 and removes it
   in `stop`; the port's 200/0 matches. The old 100/II note was wrong (see open item 2 for `stop`).
3. **No-AI air path** - `crates/pumpkin-core/src/entity/passive/axolotl.rs:523-529`
   Vanilla `Axolotl.baseTick` calls `super.baseTick()` (LivingEntity air rules, incl. refill on land
   for an air breather) and only runs `handleAirSupply` when `!isNoAi()`. The port's split matches.
4. **Hurt play-dead condition** - `crates/pumpkin-core/src/entity/passive/axolotl.rs:158-166`
   `plays_dead_when_hurt` reproduces vanilla's `!isNoAi && nextInt(3)==0 && (nextInt(3) < amount ||
   health/maxHealth < 0.5F) && amount < health && isInWater && attacker != null && !isPlayingDead`.
   Ordering and the OR are correct.
5. **Duplicate doc comment** - fixed in this pass (`axolotl.rs:183`).

## Remaining gaps

1. **Bucket data `Age` / `AgeLocked`** - `crates/pumpkin-core/src/entity/passive/axolotl.rs:374-378`
   Vanilla `Axolotl.saveToBucketTag` is `Bucketable.saveDefaultDataToBucketTag` (custom name plus the
   `NoAI`/`Silent`/`NoGravity`/`Glowing`/`Invulnerable`/`PersistenceRequired` flags and `Health`,
   already covered by `crates/pumpkin-core/src/entity/bucketable.rs:23-47`) plus the
   `minecraft:axolotl/variant` component and `HuntingCooldown`. Nothing in the vanilla path writes
   `Age` or `AgeLocked`, so those two keys look speculative and make a bucketed baby come back as a
   baby (vanilla loses the age through the bucket). `HuntingCooldown` is vanilla and must stay.
   Fix: drop the two `put_int("Age")` / `put_bool("AgeLocked")` writes and the matching reads in
   `load_from_bucket_tag`, then update the bucket round-trip test at `axolotl.rs:1286-1320`.
   Size S. Needs the decompiled `Bucketable` body to confirm before touching.
2. **PlayDead `stop` does not remove Regeneration** - `crates/pumpkin-core/src/entity/passive/axolotl/ai.rs:353-365`
   Vanilla `Axolotl$PlayDead.stop` calls `entity.removeEffect(MobEffects.REGENERATION)` when the
   activity ends; the port only adds the effect on `start`. Result: an axolotl whose play-dead is
   interrupted keeps up to ~200 ticks of Regeneration it should have lost.
   Fix: implement `stop` on `PlayDead` to `remove_effect(&StatusEffect::REGENERATION)`. Size S.
   Needs the decompiled `stop` body to confirm the unconditional removal.
3. **`custom_server_ai_step` play-dead flag** - `crates/pumpkin-core/src/entity/passive/axolotl.rs:635-637`
   Vanilla assigns `setPlayingDead(brain.hasMemoryValue(PLAY_DEAD_TICKS) && !isNoAi())` unconditionally,
   i.e. a no-AI axolotl is forced back to `false`; the port skips the assignment when `is_no_ai()`,
   so a stale `true` survives a `/data merge ... NoAI:1`. Also `ticks > 0` drops the final tick that
   vanilla's memory-presence check still reports as playing dead.
   Fix: `self.set_playing_dead(play_dead_ticks.is_some() && !self.mob_entity.is_no_ai())`.
   Size S; needs the decompiled `customServerAiStep`/`baseTick` line to confirm.

## Checked and matching

Variant pools and the 1/1200 rare roll, group-data spawning and baby-from-third-member rule, air
supply 6000 / rehydrate 1800 / dry-out damage 2, bucket round trip and tropical-fish-bucket food
remainder, breeding food tag, play-dead/attack/fight brain activities, hunting cooldown memory and
the 2400-tick arming, `on_stop_attacking` killer-in-20-blocks regeneration (100 per axolotl, cap
2400, amplifier 0 kept), `AxolotlAttackablesSensor` (8-block cube, in water, always-hostiles always,
hunt targets only without cooldown), head rotation limits, hurt/death/ambient/swim sounds,
`removeWhenFarAway`, `requiresCustomPersistence`, `canBeSeenAsEnemy`, `isPushedByFluid` false,
`getWalkTargetValue` 0.0.
