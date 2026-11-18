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

## Gap status
- 1 RESOLVED: `shoot_fireball` now uses the shooter's RNG via `Mob::get_random`, which returns a real per-entity `EntityRng` (`entity/mob/mod.rs:1672`); the spread stream is blaze-local.
- 2 RESOLVED: ordering vs goal tick verified against vanilla `Mob.serverAiStep` (`Mob.java:826-852`), correct.
- 3 RESOLVED: `position_target` init verified against vanilla `Mob.homePosition/homeRadius` and `MoveTowardsRestrictionGoal.canUse`, correct.


## Result

- Gap 1 (S, fireball spread RNG): FIXED in `blaze_attack.rs`. Vanilla `BlazeAttackGoal.tick` aims with
  `this.blaze.getRandom().triangle(...)`; `shoot_fireball` now takes the shooter's RNG via the
  `Mob::get_random` seam (`let mut rng = blaze.get_random()`) instead of calling the global
  `rand::rng()` directly, matching vanilla's structure. `Mob::get_random` (`entity/mob/mod.rs:1672`)
  now returns a real per-entity `EntityRng` backed by the entity's own seeded generator, so the
  blast-spread stream is blaze-local and no longer shared across all blazes; the earlier
  seed-sharing concern is resolved by that cross-crate change. No pure unit test seam exists here:
  `shoot_fireball` spawns into a live world, so its RNG use is covered by the existing blaze tests
  rather than an isolated deterministic test.
- Gap 2 (S, AI-step ordering): VERIFIED CORRECT, no change. Vanilla `Mob.serverAiStep`
  (`Mob.java:826-852`) runs sensing, goal/target tick, navigation, then `customServerAiStep`, then
  move/look/jump controls. Pumpkin `MobEntity::server_ai_step` (`entity/mob/mod.rs:482-534`) uses
  the identical order: goals at 495-499, navigation at 502, `mob.custom_server_ai_step` at 521,
  controls after. The ascent hook fires after goal ticking, so it cannot be overwritten in the
  same tick.
- Gap 3 (S, restriction/home init): VERIFIED CORRECT, no change. Vanilla initialises
  `homePosition=BlockPos.ZERO`, `homeRadius=-1` and `isWithinHome` returns true when radius==-1,
  so `MoveTowardsRestrictionGoal.canUse` is false with no home. Pumpkin mirrors this exactly:
  `position_target=BlockPos::ZERO`, `position_target_range=-1` (`entity/mob/mod.rs:316-317`),
  `is_in_position_target_range_pos` returns true when range==-1 (398-404), and
  `MoveTowardsRestrictionGoal::can_start` returns false (goal line 33-35). Blazes do not walk to
  the world origin.
