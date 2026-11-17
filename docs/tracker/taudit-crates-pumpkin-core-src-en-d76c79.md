# Audit: crates/pumpkin-core/src/entity/ai/goal/avoid_entity.rs

2026-11-17, one pass. No TODO/FIXME markers in the file: the tracker's vanilla
hits are reference comments only (lines 18, 24, 69-70, 85, 221-222).

Vanilla net.minecraft.world.entity.ai.goal.AvoidEntityGoal compared line by line.

## Fixed (second pass, 2026-11-17)
- avoid_entity.rs:21 + 204-210 - `can_start` ran on every tick; vanilla gates
  AvoidEntityGoal.canUse with `nextInt(reducedTickDelay(100))` (1 in 50). Added the
  roll before the other checks (same order as vanilla) and widened the two creeper
  tests to the real retry window.
- avoid_entity.rs:178-197 - `tick_reach` claimed `entities: flee_distance`, but the
  sensor branch (`nearest_of`) searches `Attributes::FOLLOW_RANGE`; it now reports
  the follow range there, so island splits cannot hide a sensed threat.
- avoid_entity.rs:136, 258-270 - `drop_target` cleared the mob target on every
  tick. Vanilla `Wolf.WolfAvoidEntityGoal.start` clears it once, so the per-tick
  clear is gone and only `start` clears.

## Fixed

- avoid_entity.rs:69-74 - default targeting conditions. Vanilla builds
  TargetingConditions.forCombat().range(maxDistance).allowUnseeable().ignoreInvisibilityTesting(),
  the port kept create_attackable() defaults, i.e. it required line of sight and
  scaled the detection range by the target's visibility percent. Both flags are now
  set. Covered by creeper_flees_an_ocelot_without_line_of_sight
  (avoid_entity.rs:374-431).

## Remaining gaps

1. avoid_entity.rs:138-174 + 195-238 - brain-sensor threats (nearest_of) reuse the
   goal's random-away spot and createPath. Vanilla VillagerHostilesSensor feeds
   SetWalkTargetAwayFrom: it re-targets every tick while the hostile is inside its
   per-type distance and needs neither a random position nor a successful path, and
   it keeps walking while the threat is close instead of stopping when a path ends.
   Suggested fix: in the per_type.is_some() branch skip the gain-distance test and
   the path == None rejection, and drive navigation away each tick (or port the
   brain behavior). Size M.
2. REJECTED: avoid_entity.rs:201-207 - `confirm` runs after ranking, which is what
   vanilla `Wolf.WolfAvoidEntityGoal.canUse` does (super.canUse finds the nearest
   llama, then the override rejects a tamed/weak one). `scares_wolves` also draws
   randomness, so a candidate filter would change the RNG stream and let a farther
   llama win. Kept as is.
3. avoid_entity.rs:262-267 - a dead `Weak` threat only skips the speed refresh;
   vanilla holds a strong reference, so a despawned threat keeps the last speed
   modifier here. Needs an entity-removal hook, not a local fix. Size S.
4. avoid_entity.rs:148-170 - the sensor branch's search box is the per-type max
   distance while its targeting range is the follow range, so a hostile between the
   two is ruled out by the box. Only reachable via `nearest_of`, currently unused.
   Size S.

## Checked and matching

- 16 block horizontal / 7 vertical random position, 3.0 vertical threat search,
  < distance-gain rejection, createPath(.., 0), walk speed on start and sprint
  speed inside 49.0 squared blocks in tick.
- Candidate box inflated horizontally by the flee distance and ranked from feet;
  keep_when mirrors CatAvoidEntityGoal/OcelotAvoidEntityGoal overriding
  canContinueToUse; selector runs per candidate (NO_SPECTATORS for fish/panda).
- tick with a dead Weak target only skips the speed refresh; vanilla keeps a
  strong reference, so the difference is the last speed modifier, not a new flee.
