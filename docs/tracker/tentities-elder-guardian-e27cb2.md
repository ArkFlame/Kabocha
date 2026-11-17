# Tracker: entities/Elder Guardian (RE-REVIEW)

Sources reviewed (2026-11-16):
- crates/pumpkin-core/src/entity/mob/elder_guardian.rs
- crates/pumpkin-core/src/entity/mob/guardian.rs
- crates/pumpkin-data/src/generated/entity_type.rs (ELDER_GUARDIAN attributes)
- crates/pumpkin-data/src/generated/sound.rs (elder guardian sounds)
No decompiled vanilla tree is vendored in this worktree; comparison is against
`net.minecraft.world.entity.monster.ElderGuardian` / `Guardian` (1.21.x).

## Fixed in this pass
- elder_guardian.rs:32-33 - vanilla `ElderGuardian` constructor calls
  `setPersistenceRequired()`. Added `mob_entity.set_persistence_required()` so elder
  guardians do not despawn.
- elder_guardian.rs:85-97 - `getAmbientSound()` was absent, so elder guardians played no
  ambient sound (the generic `Mob::play_ambient_sound` only consults this override).
  Vanilla `ElderGuardian#getAmbientSound` returns `ELDER_GUARDIAN_AMBIENT` in water and
  `ELDER_GUARDIAN_AMBIENT_LAND` otherwise; added it plus the inherited
  `getAmbientSoundInterval() == 160` from `Guardian`.

## Remaining gaps
1. **Beam attack not implemented** (L). elder_guardian.rs:40-52 registers no attack goal;
   vanilla `Guardian$GuardianAttackGoal` (priority 0) stops navigation, looks at the
   target, charges 20 ticks, sets `DATA_ID_ATTACK_TARGET` metadata (drives the client
   beam), then deals 2.0 `indirectMagic` damage, and `ElderGuardian#getAttackDuration()`
   returns 60 instead of Guardian's 80. Needs a new goal file, a synced metadata index,
   and `DamageType::INDIRECT_MAGIC`. Shared with the Guardian tracker.
2. **Vanilla water travel missing** (M). elder_guardian.rs has no `travel_in_water`
   override; vanilla `Guardian#travel` does `moveRelative(speed)` + `move(SELF)`, scales
   delta by 0.9 and adds -0.005 y sink when `getTarget() == null`, while the generic
   `LivingEntity::travel_in_water` uses drag 0.8, so idle elders drift differently.
3. **GuardianMoveControl missing** (M). Vanilla `Guardian#GuardianMoveControl` is
   water-only, scales speed by 0.1 and retargets to a random point near
   `blockPosition().above(10)` every 80 ticks (160 with no target). Pumpkin uses the
   default `MoveControl`, so path shapes and the hover behaviour differ.
4. **Effect application is broader than vanilla** (S). elder_guardian.rs:99-113 applies
   Mining Fatigue to every player within 50 and sends the game event to all of them.
   Vanilla `MobEffectUtil.addEffectToPlayersAround(level, this, pos, 50.0, effect, 1200)`
   only affects (and only notifies) players that do not already hold `DIG_SLOWDOWN` with
   at least 1200 ticks remaining. Gameplay effect is close (refresh every 1200 ticks);
   fix only if the redundant packets matter.
5. **Curse sound not played** (S, needs vanilla source). elder_guardian.rs:99-113 sends
   only the game event; `Sound::EntityElderGuardianCurse` exists in pumpkin-data and
   vanilla plays `ELDER_GUARDIAN_CURSE` when the effect is applied. Confirm the exact
   fire point/volume in decompiled `ElderGuardian` before adding.
6. **Thorns reflection uses the wrong entity kind** (S, needs verification).
   elder_guardian.rs:116-122 reflects 2.0 THORNS onto `source`; vanilla requires
   `source.getDirectEntity() instanceof LivingEntity`, so an arrow hit must not reflect.
   Confirm what `on_damage` passes as `source`; if it is the causing entity, use the
   direct entity. Shared with the Guardian tracker.
