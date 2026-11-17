# Tracker: entities / Blaze

Source: `crates/pumpkin-core/src/entity/mob/blaze.rs`, `crates/pumpkin-core/src/entity/ai/goal/blaze_attack.rs`.
Vanilla reference: `net.minecraft.world.entity.monster.Blaze` (`registerGoals`, `customServerAiStep`,
`causeFallDamage`, nested `BlazeAttackGoal`).

## Fixed in this pass

- Goal set: `MoveTowardsRestrictionGoal(this, 1.0)` at priority 5 was missing and
  `WaterAvoidingRandomStrollGoal(this, 1.0, 0.0)` sat at 5. Vanilla uses 4 attack / 5 move
  towards restriction / 7 stroll / 8 look. `blaze.rs` now registers restriction at 5 and stroll
  at 7.
- `Blaze.causeFallDamage` returns `false` in vanilla; the mob inherited fall damage.
  `Mob::cause_fall_damage` now returns `false` (`blaze.rs`, test `fall_damage_is_ignored`).
- `mob_tick` applied a ghast-style `y * 0.6` drag while descending and dealt its own 1.0
  `DROWN` damage in water. Vanilla `Blaze` has no `tick` override (no vertical drag) and
  `LivingEntity.baseTick` already applies the water/rain sensitivity damage for
  `isSensitiveToWater()` mobs (`living.rs:2109`), so the override was double-damaging blazes in
  water and slowing their fall. Removed.

## Remaining gaps

1. `crates/pumpkin-core/src/entity/ai/goal/blaze_attack.rs:172-175` - fireball spread uses the
   process-wide `rand::rng()` instead of the blaze's entity random
   (`vanilla Blaze.BlazeAttackGoal#tick` uses `this.blaze.getRandom().triangle(...)`).
   Not player-visible in a single-player-parity sense, but it decouples the sequence from the
   mob seed and shares the generator across all blazes. Fix: pass the shooter's random
   (as `self.blaze` -> `RandomExt::triangle`) into `shoot_fireball`. Size: S.
2. `crates/pumpkin-core/src/entity/mob/blaze.rs:150` - vanilla `customServerAiStep` performs the
   ascent impulse and then calls `super.customServerAiStep()` (sensing + goal tick). Confirm the
   Pumpkin hook runs in the same position of `MobEntity`'s tick relative to goal ticking; if the
   hook fires before goals, the impulse can be overwritten in the same tick. Size: S.
3. `crates/pumpkin-core/src/entity/mob/blaze.rs:81` - the newly added restriction goal needs the
   mob's position target (restrict center) populated at spawn. If `position_target` stays at
   `(0,0,0)` for blazes, the goal walks them toward world origin instead of their spawn chunk.
   Verify against `MobEntity.position_target` initialisation. Size: S.

Notes: attributes (max health 20, attack damage 6, speed 0.23, follow range 48), xp reward 10,
fire immunity, and the ambient/hurt/death sounds all come from `assets/entities.json` and need no
Rust override. `isOnFire` returning the charged flag is already ported.
