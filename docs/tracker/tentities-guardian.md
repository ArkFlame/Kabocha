# Tracker: entities/Guardian (RE-REVIEW)

Sources reviewed (2026-11-16):
- crates/pumpkin-core/src/entity/mob/guardian.rs
- crates/pumpkin-core/src/entity/mob/elder_guardian.rs
- crates/pumpkin-data/src/generated/entity_type.rs (GUARDIAN attributes, ambient sounds)
No decompiled vanilla tree is vendored in this worktree; comparison is against
`net.minecraft.world.entity.monster.Guardian` / `ElderGuardian` (1.21.x).

## Fixed in this pass
- guardian.rs:59 / elder_guardian.rs:41 - goals used `FloatGoal` (SwimGoal) at 0 and
  `RandomStrollGoal` (WanderAroundGoal) at 4. Vanilla has no float goal and uses
  `RandomSwimmingGoal(this, 1.0, 10)` at 4. Replaced with `RandomSwimmingGoal::new(1.0, 10)`,
  removed the float goal.
- guardian.rs:101 - `getAmbientSound()` was absent, so guardians played no ambient sound
  (entity_type ambient sound is not consulted by `Mob::play_ambient_sound`). Vanilla returns
  `GUARDIAN_AMBIENT` in water and `GUARDIAN_AMBIENT_LAND` otherwise. Added the override.
- elder_guardian.rs:67 - vanilla `ElderGuardian#customServerAiStep` calls
  `restrictTo(blockPosition(), 16)` when `!hasRestriction()`. Added via
  `MobEntity::set_home_to(..., 16)`, which `random_pos`/`melee_attack` honour.

## Remaining gaps
1. **Beam attack not implemented** (L). guardian.rs:46-73 registers no attack goal; vanilla
   `Guardian$GuardianAttackGoal` at priority 0: stops navigation, looks at target, charges for
   20 ticks, sets `DATA_ID_ATTACK_TARGET` metadata (the client beams), then deals 2.0
   `indirectMagic` damage and clears the target when line of sight is lost. Needs: new goal
   file, synced entity metadata index (guardian attack target), damage
   `DamageType::INDIRECT_MAGIC` with source = guardian, and cancel-on-hurt clearing.
2. **Vanilla water travel missing** (M). guardian.rs has no `Mob::travel_in_water` override;
   vanilla `Guardian#travel` does `moveRelative(speed)` + `move(SELF)`, scales delta by 0.9 and
   adds -0.005 y sink when `getTarget() == null`. The generic
   `LivingEntity::travel_in_water` (living.rs:2289) uses drag 0.8 and no target-based sink, so
   idle guardians drift differently.
3. **GuardianMoveControl missing** (M). Vanilla `Guardian#GuardianMoveControl`: water-only
   movement, speed scaled 0.1, retargets to a random point near `blockPosition().above(10)`
   every 80 ticks (160 when no target). Pumpkin uses the default `MoveControl`, so path shapes
   and the hover-above-player behaviour differ.
4. **Thorns reflection uses the wrong entity kind** (S, needs verification).
   guardian.rs:113 and elder_guardian.rs:96 reflect 2.0 THORNS onto `source`. Vanilla requires
   `source.getDirectEntity() instanceof LivingEntity`, i.e. an arrow/direct projectile hit does
   not reflect. Confirm whether `on_damage_with_context(source, cause)` maps `source` to the
   causing entity; if so use `cause` (direct entity) for the reflection.
5. **Elder guardian effect application is broader than vanilla** (S). elder_guardian.rs:70-87
   applies Mining Fatigue to every player within 50 and sends the game event to all. Vanilla
   `MobEffectUtil.addEffectToPlayersAround(..., 1200)` only affects (and only notifies) players
   that do not already hold DIG_SLOWDOWN with sufficient remaining duration. Net gameplay is
   close because the effect is refreshed every 1200 ticks; fix only if the packet spam matters.
6. **Fluid push** (S, unverified). Check whether vanilla `Guardian` overrides
   `isPushedByFluid()` (only `WaterAnimal` certainly does); if it does, add
   `Mob::mob_is_pushed_by_fluids() -> false` in guardian.rs. Do not change without a vanilla
   source to confirm.
