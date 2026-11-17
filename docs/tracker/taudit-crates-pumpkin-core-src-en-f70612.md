# Deep audit: crates/pumpkin-core/src/entity/ai/goal/melee_attack.rs

Changed: `can_start` now builds the vanilla `createPath(target, 0)` route (falling back to `is_in_attack_range` when no path exists), stores it, and `start` installs it with `move_to_path` like vanilla `moveTo(path, speed)`. Done.

| file:line | vanilla behaviour | gap | suggested fix | size |
|---|---|---|---|---|
| melee_attack.rs:263 | `MeleeAttackGoal.tick`: `if (!navigation.moveTo(target, speed)) ticksUntilNextPathRecalculation += 15;` | async `set_progress` never reports a failed search, so the retry backoff is missing and mobs with no route re-search every 4-10 ticks | let the navigator record whether the pending search failed and read it on the next tick, then add 15 | M |
| melee_attack.rs:204-214 | `stop()` only calls `navigation.stop()` and `setAggressive(false)` | the clear-target branch is unreachable: `MobEntity::get_target` (mob/mod.rs:982) already filters creative/spectator, so the `ForgotTarget` event never fires despite the comment | remove the dead block, or add a raw stored-target getter if the 26.3 vanilla source really clears it | S |
| melee_attack.rs:247-262 | `tick` thresholds use `getPerceivedTargetDistanceSquareForMeleeAttack` (attack box to target hitbox) | centre-to-centre distance shifts the +10/+5 thresholds for wide mobs (ravager, hoglin) | compute the box-to-hitbox distance like `is_within_melee_attack_range` | S |
| melee_attack.rs:19,38 | constructor arg is `followingTargetEvenIfNotSeen` (true = keep chasing without line of sight) | named `pause_when_mob_idle` with inverted meaning; creaking.rs:119 passes true where vanilla `CreakingMeleeAttackGoal` passes false (docs/tracker/tentities-creaking.md:26) | rename across callers and fix creaking | S |
| melee_attack.rs:26,86 | `attackInterval = 20` feeds `resetAttackCooldown` | `attack_interval_ticks` is dead and `get_max_cooldown` hardcodes 20 | use the field in `get_max_cooldown` | S |
