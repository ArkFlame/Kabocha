# Axolotl merge comparison: current upstream vs ours (931a0b60e) vs vanilla

Reference: `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/axolotl/*.java`, `ai/sensing/AxolotlAttackablesSensor.java`, `ai/behavior/TryFindLand.java`.
Current: HEAD (upstream #4502 rewrite). Old: `git show 931a0b60e:<path>`.

| Behaviour | Current | Old | Vanilla | Action |
|---|---|---|---|---|
| Play-dead roll (1/3 chance, damage or half health, damage < health, in water, has attacker, not no-AI, not already dead) | Rolled in `mob_hurt_server`, before the base hit, with pre-hit health | Rolled in post/on-damage hooks from a captured pre-hit snapshot | Rolled at the top of `hurtServer` with `getHealth()` before `super.hurtServer` | Keep current (same rules and order, simpler) |
| Attacker check | `source.is_some() \|\| cause.is_some()` | Same | `getEntity() != null \|\| getDirectEntity() != null` | Keep. Risk: depends on what Pumpkin passes as `source`/`cause` |
| Play-dead length 200 ticks | Same | Same | `TOTAL_PLAYDEAD_TIME = 200` | Keep |
| Regeneration buff for killer player (20 blocks, +100 ticks, cap 2400, renew under 2400 left) | Same | Same | `applySupportingEffects` | Keep |
| Stronger regeneration (amplifier > 0) left alone | Same | Same | Vanilla `addEffect` keeps the stronger one | Keep |
| Mining fatigue removed on buff | Same | Same | Same | Keep |
| `onStopAttacking` (killer player, dead target) | Same | Same | Same | Keep |
| Playing-dead flag follows PLAY_DEAD_TICKS, not updated under NoAI | Same | Same | `customServerAiStep` | Keep; test ported |
| Move/look controls frozen while playing dead | Same | Same | `AxolotlMoveControl`/`LookControl` | Keep |
| Move control 85/10/0.1/0.5 and look control 20 | Same | Same | Same | Keep |
| Travel in water | Move, drag 0.9, no sinking | Adds `-0.005` y without a target, plus a NoAI branch to `super.travel` | `travelInWater`: moveRelative, move, scale 0.9. No sink, no NoAI branch | Keep current. Old sinking is not in the reference |
| Air supply 6000, dry-out 2.0 at -20 air, rehydrate +1800 | Same | Same | Same | Keep |
| `baseTick` air with NoAI | `tick_air_supply` when NoAI, else `handle_air_supply` | Same | `super.baseTick()` then `handleAirSupply` if not NoAI | Keep. Minor risk: vanilla always runs `super.baseTick` |
| Bucket NBT: Age, AgeLocked | Same | Same | Same | Keep |
| Bucket NBT: HuntingCooldown | Written when `time_until_expiry` is Some | Gated on `has_memory_value`, then expiry | Gated on `hasMemoryValue(HAS_HUNTING_COOLDOWN)` | PORTED: gate on `has_memory_value` (edit in `save_to_bucket_tag`) |
| Bucket load (Age, AgeLocked, HuntingCooldown erase/set) | Same | Same | Same | Keep |
| Bucket variant component and from-bucket flag | Same | Same | Same | Keep |
| Variant NBT (`Variant` int, `FromBucket`), unknown id falls to Lucy | Same | Same | `LEGACY_CODEC`, `OutOfBoundsStrategy.ZERO` | Keep |
| Breeding: rare blue | 1 in **1200** | Same | `RARE_VARIANT_CHANCE = 1200` (`useRareVariant`) | Keep. The brief said 1/2000; the reference says 1200 |
| Breeding: parent or partner, 50/50 | Same | Same | Same | Keep |
| Spawn variants (common pool) and group babies (group size >= 2 gives baby, age -24000) | Same | Same | `finalizeSpawn`, `AxolotlGroupData` | Keep. Old RNG-scripted test not ported (see below) |
| Rare spawn variant (blue only) | Same | Same | `getRareSpawnVariant` | Keep |
| Hunt sensor: 8 blocks (64 sq), in water, always-hostile or hunt tag, no HAS_HUNTING_COOLDOWN | Identical logic | Identical | `AxolotlAttackablesSensor` | Keep |
| Hunt cooldown 2400 ticks on leaving Fight | Same | Same | `updateActivity` | Keep |
| Fight: chase speed 0.6 water / 0.15 land, melee cooldown 20, erase on breeding | Same | Same | Same | Keep |
| Idle: speed 0.5 water / 0.15 land; love 0.2; adult follow 0.6 water | Same | Same | Same | Keep |
| Idle: `TryFindLiquid` 6 blocks, every 40 ticks, retry 22 | Same logic | Same | Vanilla decompile has `TryFindLand` only; no separate `TryFindWater` file | Keep. Unverified against vanilla source |
| Tempt/food items: `AXOLOTL_FOOD` tag, tropical fish bucket returns water bucket | Same | Same | `isFood`, `usePlayerItem` | Keep |
| Attack sound | Plays from `on_attack` (brain and goal melee, after landed hit only) | Same | `playAttackSound` from `doHurtTarget` after landed hit | PORTED: override `play_attack_sound` so every `do_hurt_target` caller gets it |
| Ambient sound: water/air idle; silent while playing dead | Same | Same | Same | Keep |
| Hurt, death, swim, splash sounds | From entity type data (`hurt_sound`, `death_sound`, `swim_sound`) | Same | Hurt/death/swim sounds | Keep (verified in `pumpkin-data`) |
| Leash and persistence (`can_be_leashed` true; leashed or from bucket requires persistence) | Same | Same | `canBeLeashed` true; `requiresCustomPersistence` | Keep |
| Pushed by fluids: false | Same | Same | Same | Keep |
| Walk target value 0 | Same | Same | Same | Keep |
| Head rotation 1/1 | Same | Same | Same | Keep |
| Navigator: amphibious, WATER malus 0 | Same | Same | `AmphibiousPathNavigation`, WATER malus 0 | Keep |
| Remove when far away (not from bucket, no custom name) | Same | Same | Same | Keep |
| Playing dead hides from enemies and silences ambient | Same | Same | `canBeSeenAsEnemy`, `playAmbientSound` | Keep; test ported |
| Spawn obstruction and spawn rules (`AXOLOTLS_SPAWNABLE_ON`) | Present in `entity/type.rs` | Same | `checkAxolotlSpawnRules` | Keep (not re-verified in depth) |
| Water-animal max air (300) | `MAX_AIR` = 300 | 300 | 300 | Keep |
| Baby dimensions (0.375 x 0.21, eye 0.09375) | Not implemented | Not implemented | `getDefaultDimensions` | GAP in both. Not ported (needs a mob dimension hook) |
| `canSetWalkTargetFromLookTarget` on unloaded chunk | `world.get_fluid` | Same | `getFluidIfLoaded`, returns false if unloaded | GAP in both. Not changed |
| Brain: 5 sensors, 4 activities, ValidatePlayDead, PlayDead (regen 200 at amp 0, water only) | Same | Same | Same | Keep |

## Files changed

- `crates/pumpkin-core/src/entity/passive/axolotl.rs`
  - `save_to_bucket_tag`: HuntingCooldown gated on `has_memory_value`.
  - `on_attack` renamed to `play_attack_sound` (vanilla `playAttackSound`).
  - Tests added: `the_playing_dead_flag_follows_the_memory_unless_there_is_no_ai`, `playing_dead_hides_it_from_enemies_and_silences_it`.
- `crates/pumpkin-core/src/entity/passive/axolotl/ai.rs`
  - Test added: `playing_dead_outranks_a_fight_and_holds_the_activity`.
- `docs/investigations/axolotl-merge-compare.md` (this file).

## Old tests not ported

- `spawn_group_draws_its_variants_once_and_makes_babies_from_the_third_member`, `breeding_rolls_the_rare_variant_one_in_1200`, `breeding_takes_this_parent_or_the_partner`: scripted-RNG draw mirrors, close to formula mirroring. Logic is unchanged from old.
- `air_runs_down_on_land_and_refills_in_water`: not reviewed in this pass.
- `a_blocked_hit_still_rolls_for_play_dead`: current `hits_set_the_play_dead_memory_only_when_the_conditions_hold` already drives the roll through `damage_with_context`.
- `play_dead_roll_uses_pre_hit_health_like_vanilla`, `the_play_dead_roll_runs_after_the_hit_is_applied`: test the old deferred-hook design. Current rolls before the hit, so these do not apply.
- `playing_dead_regenerates_in_the_water_only`, `a_breeding_axolotl_is_offered_no_attack_target`: use `BrainState`/`BrainTick` field shapes not re-checked against current code.
- `navigation_is_amphibious_like_vanilla`: uses `is_stable_destination`, which is not in current pumpkin-core.

## Unverified risks

- Nothing was compiled or run (no cargo in this pass). All new test code and the two code edits need `scripts/check.sh pumpkin-core` and clippy before landing.
- Attacker check (`source`/`cause`) vs vanilla `getEntity()`/`getDirectEntity()`: depends on how Pumpkin fills those arguments.
- `travel_in_water` and `base_tick` no-AI behaviour follows the decompiled reference, which differs from the old implementation.
- The brief's rare-variant chance (1/2000) is not what the reference says (1200). Kept 1200.
- Baby dimensions and the unloaded-chunk look-target check are gaps in both old and current.
