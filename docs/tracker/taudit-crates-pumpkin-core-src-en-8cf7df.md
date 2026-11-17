# ai/goal/track_target.rs - audit (AUDIT)

Reviewed crates/pumpkin-core/src/entity/ai/goal/track_target.rs against vanilla
TrackTargetGoal (1.21.x memory; no .vendor tree in this worktree) and its callers
(active_target.rs, revenge.rs, owner_hurt_target.rs, owner_hurt_by_target.rs,
defend_village_target.rs, fox/goals.rs, trader_llama.rs, teleport_towards_player.rs).

## Matches vanilla (no action)
- Field defaults: max_time_without_visibility 60, can_navigate_flag UNSET,
  cooldown 0; to_goal_ticks (goal/mod.rs:78) is reducedTickDelay
  (positiveCeilDiv 2), so 60 server ticks == 60 goal ticks (2:1 cadence).
- start(): resets navigate flag, cooldown and unseen counter.
- can_track(): predicate first, then restriction range, then the
  checkCanNavigate cooldown/flag logic, with the cooldown only decremented when
  checkCanNavigate is set - same order and arithmetic as vanilla canAttack().
- can_navigate_to_entity(): cooldown = reducedTickDelay(10 + nextInt(5)),
  matching vanilla canNavigateToEntity.
- should_continue(): target -> targetMob fallback, canAttack, follow-range
  squared distance, then the mustSee unseen-tick window; re-sets target
  (TargetReason::Unknown) so the handle stays fresh.
- stop(): clears target with FORGOT_TARGET (skipped when a plugin cancelled the
  target change in start, an intentional Pumpkin extension).

## Remaining gaps
1. track_target.rs:88-94 - can_navigate_to_entity() always returns false.
   Vanilla builds a real path (getNavigation().createPath(target, 0), with a
   wider-radius retry when that fails). Pumpkin has no path-finding API
   (only straight-line NavigatorGoal, entity/mob/mod.rs:955), so
   check_can_navigate goals always report CANNOT_TRACK and never chase.
   Suggested fix: land a pathfinder (createPath(entity, radius) on the
   navigator) and then return path.is_some(), keeping the 10+rnd cooldown.
   Vanilla: TrackTargetGoal.canNavigateToEntity. Size L (blocked on pathfinder).

## Fixed in this audit
- track_target.rs:124 - the isWithinRestriction (home/position target range)
  check was a TODO, so a mob with a home chased targets outside it. Now uses the
  existing MobEntity::is_in_position_target_range_pos (same helper as
  melee_attack.rs:176), which implements distSqr < r*r with radius -1 meaning no
  restriction. Vanilla: TrackTargetGoal.canAttack -> Mob.isWithinRestriction.

## Gap status
- can_navigate_to_entity pathfinder (row 1): STILL OPEN, no path-finding API (only NavigatorGoal, entity/mob/mod.rs:955). Prior fix track_target.rs:125 FIXED.

## Result

- Row 1 (can_navigate_to_entity pathfinder): FIXED. The prior blocker (no path
  API) is obsolete: crates/pumpkin-core/src/entity/ai/pathfinder/mod.rs now
  provides Navigator::create_path. Verified against decompiled vanilla
  .vendor/vanilla-decompiled/net/minecraft/world/entity/ai/goal/target/TargetGoal.java
  (canReach): set the reducedTickDelay(10 + nextInt(5)) cooldown, build
  createPath(target, 0), require a non-null end node within squared 2.25 blocks
  of the target block position. Implemented in track_target.rs using the same
  steered/navigator() pattern as move_through_village.rs and sniffer.rs; the
  path check stays in can_start via can_track, so a failed path cannot evict
  the current target goal. Added a pure unit test for the end-node tolerance.
  Ran rustfmt --edition 2024 on track_target.rs: clean.
