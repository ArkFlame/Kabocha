# entities / Bat (RE-REVIEW-UNTRACKED)

Source: `crates/pumpkin-core/src/entity/mob/bat.rs`.
Reference: decompiled vanilla 26.3 (target `CURRENT_MC_VERSION::V_26_3`), class
`net.minecraft.world.entity.ambient.Bat` taken from the official server bundle
(`META-INF/versions/26.3/server-26.3.jar`, sha1 `33680f5f2ac32864d6d7cf5e56a705fdb3e05f4c`) and
read with `javap -p -c -constants`. Members compared one by one.

## Verdict
No behaviour gap found. Every vanilla member of `Bat` (plus its only `AmbientCreature` addition) has an
equivalent in the Rust port with matching constants, RNG order and early-return order.

Verified equal:
- `isFlapping`: `tick_count % 10 == 0 && !is_roosting` (`TICKS_PER_FLAP = 10.0f` in 26.3, not the older
  `Mth.ceil(3.4F)`; the current `% 10` is correct).
- `getMovementEmission() -> EVENTS`: already carried by generated data
  (`crates/pumpkin-data/src/generated/entity_type.rs`, BAT entry, `MovementEmission::Events`).
- `getSoundVolume 0.1`, `getVoicePitch base*0.95`, `getAmbientSound` gate `resting && nextInt(4) != 0 -> null`,
  `getHurtSound`/`getDeathSound`.
- `isPushable false`, `doPush`/`pushEntities` no-ops, `canBeLeashed false` (AmbientCreature),
  `isIgnoringBlockTriggers true`, `checkFallDamage` no-op.
- `createAttributes`: `Mob.createMobAttributes()` + `MAX_HEALTH 6.0`; generated bat attributes match
  (health 6.0, movement speed 0.7, follow range 16).
- `isResting`/`setResting`: bit 1 of the synced flags byte, mask `-2` when clearing (bat.rs:72-86 identical).
- `tick()`: resting -> zero velocity + `setPosRaw(x, floor(y) + 1 - bbHeight, z)`; flying -> velocity
  `* (1, 0.6, 1)` (bat.rs:271-283).
- resting AI: `getBlockState(above).isRedstoneConductor(level, above)` (`is_solid_block`),
  `nextInt(200) == 0 -> yHeadRot = nextInt(360)`, `getNearestPlayer(BAT_RESTING_TARGETING, this)` with
  `forNonCombat().range(4.0)`, `setResting(false)` + `levelEvent(1025)` when not silent (bat.rs:88-120).
- flying AI: target invalidation `!isEmptyBlock(target) || y <= minY` (`isEmptyBlock` is `isAir()` only,
  confirmed in `LevelReader`), re-pick on `nextInt(30) == 0` or `closerToCenterThan(..., 2.0)`, offset draws
  `nextInt(7)/nextInt(7)`, `nextInt(6) - 2`, `nextInt(7)/nextInt(7)` in the same order, goal vector
  `(dx, dy + 0.1, dz)`, easing `(signum*0.5 - v)*0.1F` and `(signum*0.7F - v)*0.1F`, yaw
  `atan2(vz, vx)*180/PI - 90`, `zza = 0.5`, re-hang on `nextInt(100) == 0 && getBlockState(above)
  .isRedstoneConductor(...)` (bat.rs:122-196).
- `hurtServer`: `isInvulnerableTo` early return, then wake before `super.hurt` (bat.rs:285-298).
- `addAdditionalSaveData`/`readAdditionalSaveData`: `BatFlags` byte, default 0 (bat.rs:238-247).
- `checkBatSpawnRules`: `y >= worldSurface`, `nextBoolean` 50% reject, `getMaxLocalRawBrightness > nextInt(4)`,
  `BATS_SPAWNABLE_ON` on the block below, then `checkMobSpawnRules` (bat.rs:44-69, identical order).

## Remaining notes (no code change)
- Client-only: `setupAnimationStates` and the `AnimationState` fields are rendering state, nothing to port
  server-side.
- S: bat.rs:29-30 keeps a shadow `AtomicI8` instead of reading the synced `DATA_ID_FLAGS` byte like vanilla;
  divergence only possible if a plugin rewrites tracked data for bats. Leave.
- S: the resting wake check uses `is_solid_block()` where vanilla calls `isRedstoneConductor(level, pos)`;
  Pumpkin's `IS_SOLID_BLOCK` flag is generated from the redstone-conductor notion
  (`crates/pumpkin-data/src/block_state.rs:135`), so equivalent unless a block overrides
  `isRedstoneConductor` without the flag.
