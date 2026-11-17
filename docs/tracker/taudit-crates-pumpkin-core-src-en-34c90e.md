---
row: audit
path: crates/pumpkin-core/src/entity/ai/behavior/hunting.rs
status: audited
date: 2026-11-17
---

# Audit: entity/ai/behavior/hunting.rs

## Markers
No TODO/FIXME. The 3 "vanilla" hits are doc references only:
- `hunting.rs:15` short `NearestAttackableTargetGoal` ctor defaults (reciprocal 10, mustReach false) - correct.
- `hunting.rs:76` `TargetingConditions.selector` equivalent - correct.
- `hunting.rs:83` `setUnseenMemoryTicks` - correct (TargetGoal default 60 also mirrored in `TrackTargetGoal::new`).

## Verified parity
- `Hunt::every` -> `ActiveTargetGoal.reciprocal_chance` -> `to_goal_ticks` = vanilla `Goal.reducedTickDelay` (ceil n/2); 0 = every tick, `random_range(0..n) != 0` matches `nextInt(n) != 0`.
- No reach check by default: vanilla short ctor has mustReach=false; `TrackTargetGoal::with_default` sets `check_can_navigate=false`.
- Flags: TARGET via `TrackTargetGoal::new` (vanilla `setFlags(TARGET)`).
- `within_four_blocks_height` (hunting.rs:132) == vanilla Slime/Ghast predicate `Math.abs(target.getY() - this.getY()) <= 4.0`; `pos.load().y` is the feet Y, same as `getY()`. Used by slime.rs:38 and ghast.rs:24, both must_see true, matching vanilla.
- `baby_turtles` (hunting.rs:127) == `NearestAttackableTargetGoal<>(mob, Turtle.class, 10, mustSee, false, Turtle.BABY_ON_LAND_SELECTOR)`; callers (fox, wolf, cat, ocelot, skeleton) match vanilla mustSee values.

## Remaining gaps
1. `hunting.rs:127` `baby_turtles` cannot carry vanilla `Turtle.BABY_ON_LAND_SELECTOR`'s `TargetingConditions.forNonCombat()`. `Hunt::build` always uses `ActiveTargetGoal::with_default` -> `TargetPredicate::create_attackable()`, while vanilla non-combat conditions set `attackable = false` (and `ignoreLineOfSight()` inside the conditions). Observable effect is nil for baby turtles (always attackable), so left unfixed. Fix: add `Hunt::non_combat()` + `non_combat` flag on `ActiveTargetGoal` setting `target_predicate.attackable = false`; also expose `ignore_visibility` (already on `TargetPredicate`). Size: S/M (2 files).
2. `hunting.rs:28,92` `range_scale` only scales `FOLLOW_RANGE`. Vanilla selectors can pin an absolute range (`BABY_ON_LAND_SELECTOR` uses `range(16.0)`); callers' FOLLOW_RANGE is 16 today, so no divergence yet. Fix only if a caller needs a non-follow-range radius. Size: S.

## Not gaps
- Predicate ordering: `ActiveTargetGoal::accepts` runs the selector before the distance/sight test; vanilla tests range/sight before the selector. Pure predicates only (`within_four_blocks_height`), no observable difference.
