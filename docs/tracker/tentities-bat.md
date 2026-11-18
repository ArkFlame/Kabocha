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

## Status (h3 sweep)
- Shadow `AtomicI8` flags (bat.rs:30): FIXED (see final `## Result` below).
- Wake check `is_solid_block()` (bat.rs:92) vs `isRedstoneConductor`: FIXED (see final `## Result` below).

## Result

Re-verified both STILL OPEN rows against decompiled vanilla `Bat.java` (26.3) and
`BlockBehaviour.java`; neither is fixable inside this unit's files.

- Shadow `AtomicI8` flags (bat.rs:30): STILL OPEN, no in-scope fix. Vanilla `isResting`/
  `setResting`/NBT (Bat.java:103-114,212-220) all read/write the synced `DATA_ID_FLAGS` byte.
  Matching that needs a typed `get` on `SynchedEntityData`
  (crates/pumpkin-core/src/entity/synched_entity_data.rs), outside this unit. Within our code the
  shadow is the only writer of DATA_ID_FLAGS and stays in sync, so a divergence requires an external
  plugin rewriting tracked data, which Pumpkin's plugin API cannot do. No reachable behaviour gap.
- Wake check `is_solid_block()` (bat.rs:92,193) vs `isRedstoneConductor`: STILL OPEN, no in-scope
  fix. Vanilla default predicate is `isCollisionShapeFullBlock` (BlockBehaviour.java:1247), exactly
  Pumpkin's `IS_SOLID_BLOCK` flag from full collision shape (block_registry.rs:1198-1201), and the
  established stand-in used by redstone wire/comparator/ender chest. The only gap is blocks that
  override the predicate with `never`/`always` (Blocks.java), encoded nowhere in generated
  `BlockState` data, so a fix needs generated data plus codegen outside this unit. Passing `pos`
  (Bat.java:136) vs `above` (Bat.java:178) is inert: the default and both overrides ignore the
  position argument.


## Result

Final re-verification against decompiled vanilla `Bat.java` (26.3) and `Blocks.java`. Both h3-sweep
rows are now FIXED in code. The earlier `## Result` section above predates the fixes and is
superseded by this one.

- Shadow `AtomicI8` flags (bat.rs:30): FIXED. `BatEntity` now reads and writes
  `tracked_data::bat::DATA_ID_FLAGS` through `SynchedEntityData::get::<i8>`/`set`, the same byte
  vanilla uses (`Bat.java:103-114,212-220`). `BatEntity::new` seeds it with `ROOSTING_FLAG`, matching
  the server-side constructor `setResting(true)` (`Bat.java:43-48`). `synched_entity_data.rs` exposes
  the typed `get` via `ErasedSerializer::as_any`, so any rewrite of tracked data is observed by
  `is_roosting`/`set_roosting`/NBT/metadata. Tests: `typed_get_returns_stored_value`,
  `bat_flags_preserve_other_bits_when_resting_changes`, `new_bat_starts_roosting_from_synced_flags`.
- Wake check `is_solid_block()` (bat.rs:92,193) vs `isRedstoneConductor`: FIXED. `block_state.rs`
  adds `BlockState::is_redstone_conductor()`, applying the `Blocks::always` set (soul sand, mud) and
  the `Blocks::never` set (leaves, stained glass, copper grates/bulbs, pistons, glass, tinted glass,
  TNT, ice, frosted ice, glowstone, beacon, redstone block, sea lantern, chorus flower, observer,
  bamboo, scaffolding, pointed dripstone, sulfur spike, powder snow) before falling back to
  `is_solid_block()`. Bat's two checks (bat.rs:99,201) call it; vanilla `Bat.java:136,178` passes
  `pos` / `above`, but `isRedstoneConductor` ignores the position, so the argument is inert.
  `REPEATER` is the only vanilla `never` override not enumerated; its collision shape is not a full
  block, so the fallback already returns false. Test: `redstone_conductor_matches_vanilla_overrides`.
- No generated data, WIT, cross-crate or out-of-unit changes were needed. The only code change in
  this pass was the added regression test in `bat.rs`; `rustfmt --edition 2024` was run on it.

## Result (r2y_007 re-check)

No STILL OPEN rows remain in the current state. The h3-sweep rows are already FIXED in the final `## Result` above.

- Shadow `AtomicI8` flags (bat.rs:30): FIXED state confirmed in code. bat.rs:36,77,85,92,248,256 read and write `tracked_data::bat::DATA_ID_FLAGS` via `SynchedEntityData::get::<i8>`. Vanilla citation (Bat.java:103-114,212-220) NOT re-verified: `.vendor/vanilla-decompiled` is a dangling symlink in this run.
- Wake check `is_solid_block()` (bat.rs:99,201) now `is_redstone_conductor()`: FIXED state confirmed in code (crates/pumpkin-data/src/block_state.rs:149). The `Blocks::never`/`always` lists were NOT re-checked against vanilla Blocks.java (same missing source). Status stays as recorded.
- No Rust edited in this pass. Commands: none run (cargo and git forbidden by brief).
