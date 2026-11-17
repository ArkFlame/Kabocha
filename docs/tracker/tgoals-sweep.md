# Generic AI goal sweep (crates/pumpkin-core/src/entity/ai/goal)

Scope: constructors, defaults, timers and tick cadence of the generic goals, compared with
vanilla goal classes from memory. No decompiled vanilla source is vendored in this checkout, so
every "match" below is medium confidence and every "unverified" item needs a decompiled-source
check before a fix.

No code changed: no divergence was found where confidence was high enough to edit.

## Matches (code agrees with vanilla)

| Goal | File | Value checked |
|---|---|---|
| RandomStrollGoal (WanderAround) | wander_around.rs | `DEFAULT_INTERVAL` 120, `to_goal_ticks` (ceil half) on the `nextInt` roll, no-action-time limit 100, default position `DefaultRandomPos 10/7` |
| WaterAvoidingRandomStroll | wander_around.rs | `PROBABILITY` 0.001, in-water `LandRandomPos 15/7`, land `10/7` |
| WaterAvoidingRandomFlying / Parrot | water_avoiding_random_flying.rs | hover `8,7,PI/2,3,1` then air/water `8,4,-2`; parrot search `x,z +-3, y -6..6`, leaves/logs under air |
| LookAtPlayerGoal | look_at_entity.rs | default chance 0.02, look time `40 + nextInt(40)` scaled by `get_tick_count` |
| RandomLookAroundGoal | look_around.rs | chance 0.02 (1/50), look time `20 + nextInt(20)` |
| FloatGoal (SwimGoal) | swim.rs | jump when in water above swim height or in lava, roll < 0.8 |
| FollowParentGoal | follow_parent.rs | search 8 wide / 4 tall, min distance 3 (9 sq), max distance 16 (256 sq), recalc `to_goal_ticks(10)` |
| BreedGoal | breed.rs | partner range 8, love ticks >= 60 and distance < 3, spawn xp `nextInt(7)+1`, breeding cooldown 6000 |
| TemptGoal | tempt.rs | stop distance 2.5, calm-down `to_goal_ticks(100)`, decrements each tick |
| MeleeAttackGoal | melee_attack.rs | 20-tick `canUse` gate, attack interval `get_tick_count(20)`, path recalc `4 + nextInt(7)` +5/+10 by distance, LOS only gates recalc, reach via attack box |
| AvoidEntityGoal | avoid_entity.rs | fast speed inside 7 blocks (49 sq), `getPosAway 16/7` |
| EatBlockGoal (EatGrass) | eat_grass.rs | roll `nextInt(baby ? 50 : 1000)`, eat timer `get_tick_count(40)`, eat at timer `get_tick_count(4)` |
| MoveToBlockGoal | move_to_block.rs | give-up 1200, stay `nextInt(nextInt(1200)+1200)+1200`, retry `reducedTickDelay(200 + nextInt(200))`, spiral search order |
| RestrictSunGoal | restrict_sun.rs | navigator avoid-sun flag set on start |
| RangedBowAttack | ranged_attack.rs | see time >= 5 before the shot |

## Divergences found but not edited

- RandomLookAroundGoal: the brief says 1/80. Vanilla is `nextFloat() < 0.02F` (1/50), which is
  what the code has. The brief is wrong here, the code is right.
- FleeSunGoal `find_hide_pos`: the code samples offsets `-10..=10` (x, z) and `-3..=3` (y), 21
  values per axis. Vanilla is believed to be `nextInt(20) - 10` and `nextInt(6) - 3`, which gives
  -10..=9 and -3..=2. Confidence is about 65 percent, so it is left unchanged. Verify against
  decompiled `FleeSunGoal.getHidePos` before editing.
- OpenDoorGoal `forget_time`: hardcoded 20. Vanilla may use `adjustedTickDelay(20)`. Since this
  goal already runs every tick (`should_run_every_tick`), the two are equivalent, so no change.
- PanicGoal: speed is a caller argument (`EscapeDangerGoal::new(speed)`), so the 1.25 to 2.0 range
  is set by mob code, not by the goal. Out of scope for this sweep.
- MeleeAttackGoal: the TODO for +15 ticks on failed path requests is still open. Not a confident
  divergence, left as is.

## Unverified (not in the generic sweep)

- RangedAttackGoal, ReactToBell, OpenDoor/door interaction details, TemptGoal range per mob.
  Each needs a decompiled-source check.

## Re-verification against decompiled vanilla (x1)

- FIXED: FleeSunGoal `find_hide_pos` offsets. Vanilla `FleeSunGoal.getHidePos` uses `nextInt(20) - 10`, `nextInt(6) - 3`, `nextInt(20) - 10`, so x/z span -10..=9 and y -3..=2. Code now uses `random_range(-10..10)` and `random_range(-3..3)`. The predicate still differs (solid-block check instead of `!canSeeSky && getWalkTargetValue < 0`); left as is, needs sky-visibility and path-malus data.
- VERIFIED: RangedAttackGoal (`see_time >= 5` with `<=` radius, attack-time lerp and floor, power clamp 0.1..1.0, LOS return before shot).
- VERIFIED: OpenDoorGoal `forget_time = 20` is a literal in vanilla too.
- Not re-verified in this pass: ReactToBell, TemptGoal range per mob.
- No automated test: the change is a random roll inside a world-dependent loop.
