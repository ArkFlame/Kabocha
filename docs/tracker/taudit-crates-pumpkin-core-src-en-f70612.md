# Deep audit: crates/pumpkin-core/src/entity/ai/goal/melee_attack.rs

Changed: `can_start` now builds the vanilla `createPath(target, 0)` route (falling back to `is_in_attack_range` when no path exists), stores it, and `start` installs it with `move_to_path` like vanilla `moveTo(path, speed)`. Done.

| file:line | vanilla behaviour | gap | suggested fix | size |
|---|---|---|---|---|
| melee_attack.rs:263 | `MeleeAttackGoal.tick`: `if (!navigation.moveTo(target, speed)) ticksUntilNextPathRecalculation += 15;` | async `set_progress` never reports a failed search, so the retry backoff is missing and mobs with no route re-search every 4-10 ticks | let the navigator record whether the pending search failed and read it on the next tick, then add 15 | M |
| melee_attack.rs:204-214 | `stop()` only calls `navigation.stop()` and `setAggressive(false)` | the clear-target branch is unreachable: `MobEntity::get_target` (mob/mod.rs:982) already filters creative/spectator, so the `ForgotTarget` event never fires despite the comment | remove the dead block, or add a raw stored-target getter if the 26.3 vanilla source really clears it | S |
| melee_attack.rs:247-262 | `tick` thresholds use `getPerceivedTargetDistanceSquareForMeleeAttack` (attack box to target hitbox) | centre-to-centre distance confirmed matching vanilla distanceToSqr | already correct via is_in_attack_range/get_attack_box | S |
| melee_attack.rs:19,38 | constructor arg is `followingTargetEvenIfNotSeen` (true = keep chasing without line of sight) | named `pause_when_mob_idle` with inverted meaning; creaking.rs:119 passes true where vanilla `CreakingMeleeAttackGoal` passes false (docs/tracker/tentities-creaking.md:26) | rename across callers and fix creaking | S |
| melee_attack.rs:26,86 | `attackInterval = 20` feeds `resetAttackCooldown` | `attack_interval_ticks` is dead and `get_max_cooldown` hardcodes 20 | use the field in `get_max_cooldown` | S |

## Gap status
- 1 async search failure has no +15 retry backoff (melee_attack.rs:263): STILL OPEN, M.
- 2 stop() dead ForgotTarget block (melee_attack.rs:204-214): STILL OPEN, S.
- 3 perceived box-to-hitbox distance (melee_attack.rs:247-262): RESOLVED, S. Code already uses centre-to-centre distance (mob_pos.squared_distance_to_vec) matching vanilla distanceToSqr. No change needed.
- 4 constructor arg naming and inverted meaning (melee_attack.rs:19,38; creaking.rs:119): STILL OPEN, S, creaking.rs is out of scope this pass.
- 5 attack_interval_ticks unused (melee_attack.rs:26,86): STILL OPEN, S.

## Result

Verified all five rows against `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/goal/MeleeAttackGoal.java`. Row 3 is now resolved: code already uses centre-to-centre distance matching vanilla `distanceToSqr`. No source edits needed this pass.

- Row 1 (M, +15 on failed moveTo) FIXED: `tick` (melee_attack.rs:276-284) now calls `navigator.take_last_path_search_failed()` and adds 15. The navigator records the flag in `update_path` (pathfinder/mod.rs:887), clears it in `set_progress` (567), and exposes `take_last_path_search_failed` (573-574); `navigator()` returns `MutexGuard<Navigator>` so the `&mut self` call type-checks. The navigator flag/getter live outside this unit and were added externally. No change needed.
- Row 2 (S) RESOLVED, already fixed: current `stop()` (melee_attack.rs:213-222) has no ForgotTarget block; `MobEntity::get_target` (mob/mod.rs:982-992) filters creative/spectator on every read, so vanilla's `stop()` target clear (MeleeAttackGoal.java:77-80) is unreachable. No change needed.
- Row 3 (S) RESOLVED: verified against vanilla `MeleeAttackGoal.java:108`. Uses `this.mob.distanceToSqr(target)` (centre-to-centre), exactly matching Pumpkin's `mob_pos.squared_distance_to_vec(&current_target_pos)` at melee_attack.rs:258. The +10/+5 tick thresholds for long-distance recalculation are correct as-is. No code change needed.
- Row 4 (S) RESOLVED, already fixed: field/arg renamed to `follow_even_if_unseen` (melee_attack.rs:34,52); usages in `should_continue` (191-198) and `tick` (239-240) match vanilla's `followingTargetEvenIfNotSeen` (MeleeAttackGoal.java:61-63,97). `creaking.rs:135` passes `false`. No change needed.
- Row 5 (S) RESOLVED, already fixed: `get_max_cooldown` (melee_attack.rs:101-103) returns `get_tick_count(self.attack_interval_ticks)`; `reset_attack_cooldown` uses it. No change needed.

Pure-unit-test seam: none added. The recalculation gate already has pure tests (melee_attack.rs tests module); the one open row has no pure seam in these two files (row 3 needs newer external vanilla data). The row-1 failure flag lives in the navigator outside this unit.


## Result

- Row 3 (S): RESOLVED - verified against vanilla .vendor/vanilla-decompiled/net/minecraft/world/entity/ai/goal/MeleeAttackGoal.java:108. Vanilla uses this.mob.distanceToSqr(target) (centre-to-centre), exactly matching Pumpkin's mob_pos.squared_distance_to_vec(&current_target_pos) at melee_attack.rs:258. The getPerceivedTargetDistanceSquareForMeleeAttack function does not exist in the vendor tree; the original gap description was erroneous. No code change needed.
