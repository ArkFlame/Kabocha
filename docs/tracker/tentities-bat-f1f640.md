# entities / Bat (RE-REVIEW-UNTRACKED)

Source: `crates/pumpkin-core/src/entity/mob/bat.rs` (rewritten in 3522114a4, test line changed in ddd13a220).
Reference: vanilla `net.minecraft.world.entity.ambient.Bat` (26.3 decompiled review in
`docs/tracker/tentities-bat.md`, re-checked against the current Rust source).
Previous verdict ("no gap") was re-audited; one member was missing.

## Fixed in this review
- S: `playStepSound(BlockPos, BlockState)` is an empty override in vanilla Bat; the Rust port
  inherited `Mob::mob_play_step_sound`, which plays the block step sound (and the amethyst chime).
  Bats fly/land silently, so this was audible divergence. Added a no-op override
  (`bat.rs:232-233`). No automated test: the only observable effect is a sound packet, which the
  unit-test harness cannot capture; verified by reading the override like the bee/allay/happy_ghast
  silent-step mobs.

## Remaining gaps
- M, needs verification: vanilla Bat's constructor sets `setNoGravity(true)` server side (older
  versions at least); `BatEntity::new` (`bat.rs:34-42`) never calls `set_has_no_gravity(true)`, so
  bats get gravity 0.08 from `LivingEntity::get_effective_gravity` (`entity/living.rs:1935-1947`)
  while flying. Fix: call `entity.set_has_no_gravity(true)` in `new` and update the
  `flying_applies_air_physics_before_bat_vertical_damping` expectation
  (`entity/living.rs:6207-6231`, currently asserts the gravity-included 0.24696 damping). Confirm
  against the 26.3 constructor first: the earlier javap review did not list the constructor, so the
  line may have been removed upstream.
- S, verify: vanilla Bat overrides `getBaseExperienceReward` to `random.nextInt(3)` in some versions;
  the Rust port falls through to `Mob::get_base_experience_reward` (`entity/mob/mod.rs:2904-2935`),
  which returns the generated `experience_reward` (0 for bats). If 26.3 still has the override, add
  `fn get_base_experience_reward(&self) -> u32 { rng.next_int(3) }` using the mob's RNG.
- S: resting wake check uses `BlockState::is_solid_block` (`bat.rs:92`) where vanilla calls
  `isRedstoneConductor(level, pos)`; equal for generated data
  (`crates/pumpkin-data/src/block_state.rs:135`) unless a block overrides the predicate. Leave.
- S: `flags` is a shadow `AtomicI8` (`bat.rs:30`) instead of reading `DATA_ID_FLAGS`; only diverges if
  a plugin rewrites tracked data. Leave.
- S: bat decisions use thread RNG (`rand::rng()` in `tick_flying`/`tick_roosting`,
  `bat.rs:94,135,193`) where vanilla uses the entity `RandomSource`. Functional behaviour matches;
  only determinism/seed sharing differs. Leave.
- S: roosting snap uses `Entity::set_pos` (`bat.rs:277`) where vanilla uses `setPosRaw`, so the snap
  goes through the regular position/collision bookkeeping. Harmless unless bats visually jitter on
  chunk borders.

## Confirmed equal (no change)
`isFlapping` (`% 10`), `tick` resting snap / flying `y * 0.6`, resting AI (200-chance head turn,
targeting `forNonCombat().range(4.0)`, liftoff `WorldEvent 1025`), flying AI (target invalidation,
`nextInt(30)`, `nextInt(7)/nextInt(6)-2/nextInt(7)` draw order, easing constants, yaw wrap,
`zza = 0.5`, re-roost on `nextInt(100)`), `hurtServer` wake-before-hurt order,
`BatFlags` NBT, `checkBatSpawnRules` order, sounds/volume/pitch, `canBeLeashed`,
`isIgnoringBlockTriggers`, `shouldCheckFallDamage`/fall distance, push gating, metadata
(`bat_flags`), spawn registration (`entity/type.rs:166,618`).
