# entities / Bat (RE-REVIEW-UNTRACKED)

Source: `crates/pumpkin-core/src/entity/mob/bat.rs`.
Reference: decompiled vanilla 26.3, class `net.minecraft.world.entity.ambient.Bat` from
`/tmp/mc263/META-INF/versions/26.3/server-26.3.jar` read with `javap -p -c -constants`
(615 lines), compared member by member against the current Rust source.

## Fixed in this review
- S: the previous review added a no-op `mob_play_step_sound` claiming vanilla `Bat.playStepSound`
  is empty. That member does not exist in vanilla 26.3: the `javap` member table of `Bat` is
  constructor, `isFlapping`, `defineSynchedData`, `getSoundVolume`, `getVoicePitch`,
  `getAmbientSound`, `getHurtSound`, `getDeathSound`, `isPushable`, `doPush`, `pushEntities`,
  `createAttributes`, `isResting`, `setResting`, `tick`, `customServerAiStep`,
  `getMovementEmission`, `checkFallDamage`, `isIgnoringBlockTriggers`, `hurtServer`,
  `readAdditionalSaveData`, `addAdditionalSaveData`, `checkBatSpawnRules`,
  `setupAnimationStates` and the static initializer. No `playStepSound` override, so vanilla bats
  emit the inherited block step sound. Removed the override (and the unused `BlockState` import);
  bats now go through `Mob::mob_play_step_sound` -> `play_block_step_sound` like every other mob.
  Commit 66fb92cd2.

## Resolved earlier open questions (evidence from the 26.3 bytecode)
- The constructor only calls `setResting(true)` when `!level.isClientSide`; there is no
  `setNoGravity` in it. Earlier note "bat should set no gravity" is invalid; `BatEntity::new`
  (bat.rs:34-42) correctly leaves gravity alone, and the flying RNG/velocity ordering already
  matches.
- `Bat` declares no `getBaseExperienceReward`, so falling through to
  `Mob::get_base_experience_reward` is correct.
- `BAT_RESTING_TARGETING` is `TargetingConditions.forNonCombat().range(4.0)` (static initializer),
  which is exactly `TargetPredicate::create_non_attackable().set_base_max_distance(4.0)`
  (bat.rs:101-102).

## Re-review 2026-11-17 (tracker: source changed)
- Re-compared `crates/pumpkin-core/src/entity/mob/bat.rs` with the 26.3 member table recorded below.
  The file is unchanged from the state reviewed last time: every line range previously cited still
  matches (`is_roosting`/`set_roosting` 72-86, `tick_roosting` 88-120, `tick_flying` 122-196,
  `mob_write_nbt`/`mob_read_nbt` 238-247, `post_tick` 271-283, `pre_damage_with_context` 285-298).
  The only moved reference is the spawn-rule dispatch, now `crates/pumpkin-core/src/entity/type.rs:629-631`
  (previously 620-622); the drift comes from unrelated insertions above it, the dispatch is unchanged.
- Checked the one member whose semantics could have drifted, `readAdditionalSaveData`: bat.rs:242-247
  stores the raw `BatFlags` byte into the synced flag, which matches vanilla
  `this.entityData.set(DATA_ID_FLAGS, nbt.getByte("BatFlags"))`. Preserving bits 1-7 is therefore
  intended, and `bat_flags_preserve_other_bits_when_resting_changes` is a valid regression test, not
  an over-test.
- Checked the spawn wiring end to end: `entity/type.rs:629-631` -> `BatEntity::check_bat_spawn_rules`,
  and bat arrives through the generated AMBIENT biome spawner list in `world/natural_spawner.rs:989`
  (no hardcoded per-mob spawn entry is missing). Wiring is complete.
- Conclusion: no new behaviour gap, no code change in this pass. The four remaining S items below are
  unchanged and still not worth a speculative edit.
## Remaining gaps (all low risk, no behaviour difference observed)
- S: `is_roosting`/`set_roosting` read and write a shadow `AtomicI8` (bat.rs:30) plus the synced
  `DATA_ID_FLAGS`, while vanilla `isResting` reads only the synced byte. Diverges only if a plugin
  rewrites the tracked value. Suggested fix: back the flag with `get_synced_data(DATA_ID_FLAGS)`
  and delete the atomic. Leave unless plugin tracked-data writes must be honoured.
- S: `tick_roosting` uses `BlockState::is_solid_block` (bat.rs:92) where vanilla calls
  `BlockState.isRedstoneConductor(level, pos)`; Pumpkin's `IS_SOLID_BLOCK` flag is generated from
  the redstone-conductor notion, so it is equal for generated data unless a block overrides the
  predicate. Suggested fix: add a `is_redstone_conductor(world, pos)` helper and use it here and
  in `tick_flying` (bat.rs:193).
- S: bat decisions use the thread RNG (`rand::rng()`, bat.rs:94/135/193) where vanilla uses the
  entity `RandomSource`; observable behaviour matches, only determinism/seed sharing differs.
  Suggested fix: switch to `self.get_random()` (`LivingEntity` RNG) for reproducible tests.
- S: the resting snap uses `Entity::set_pos` (bat.rs:277) where vanilla uses `setPosRaw`, so the
  snap runs the regular position/collision bookkeeping. Suggested fix: only if bats are seen
  jittering on chunk borders; no functional evidence.
- Client-only: `setupAnimationStates` and `fly/restAnimationState` are rendering state; nothing to
  port server-side.

## Confirmed equal
Constructor resting default, `isFlapping` (`tickCount % 10 == 0`, `TICKS_PER_FLAP = 10.0f`),
`getSoundVolume 0.1`, `getVoicePitch base*0.95`, ambient gate, hurt/death sounds,
`isPushable`/`doPush`/`pushEntities`, `createAttributes` (health 6.0), `setResting` mask `-2`,
`tick` resting snap/flying `y*0.6`, resting AI order (silent -> isRedstoneConductor -> 200 head
turn -> nearest player -> liftoff event 1025), flying AI (invalidate on `!isEmptyBlock` or
`y <= minY`, `nextInt(30)`, `nextInt(7)/nextInt(6)-2/nextInt(7)` draw order, easing `0.1F` with
`0.5/0.7/0.5`, `zza = 0.5` before yaw, re-roost on `nextInt(100)`), `hurtServer` wake-before-super
order, `BatFlags` NBT default 0, `checkBatSpawnRules` early-return order, `getMovementEmission
EVENTS` (generated), `checkFallDamage` no-op, `isIgnoringBlockTriggers`, spawn wiring
(`entity/type.rs:620-622`).
