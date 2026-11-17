# Audit: crates/pumpkin-core/src/entity/ai/goal/avoid_entity.rs

2026-11-17, one pass. No TODO/FIXME markers in the file: the tracker's vanilla
hits are reference comments only (lines 18, 24, 69-70, 85, 221-222).

Vanilla net.minecraft.world.entity.ai.goal.AvoidEntityGoal compared line by line.

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
2. avoid_entity.rs:201-207 - confirm is applied only to the already ranked nearest
   threat. Vanilla's predicateOnAvoidEntity/avoidPredicate are candidate filters
   inside getNearestEntity/getEntitiesOfClass, so a nearer llama that fails
   scares_wolves still lets a farther scaring llama start the flee. Suggested fix:
   fold confirm into the find_threat accept closure. Verify against decompiled
   Wolf.WolfAvoidEntityGoal first: it changes how many RNG draws llama.rs makes.
   Size S.
3. avoid_entity.rs:175-194 - tick_reach reports entities: flee_distance, but the
   sensor branch searches FOLLOW_RANGE (lines 148-157). With short per-type
   distances an island split can hide a hostile the goal still reacts to. Suggested
   fix: report the follow range when per_type.is_some(). Size S.
4. avoid_entity.rs:247-263 - drop_target clears the mob target in start and again
   every tick. Vanilla WolfAvoidEntityGoal.start clears it once; the per-tick
   clear also wipes a target picked up during the flee. Suggested fix: keep the
   clear in start only, or confirm the vanilla override. Size S.

## Checked and matching

- 16 block horizontal / 7 vertical random position, 3.0 vertical threat search,
  < distance-gain rejection, createPath(.., 0), walk speed on start and sprint
  speed inside 49.0 squared blocks in tick.
- Candidate box inflated horizontally by the flee distance and ranked from feet;
  keep_when mirrors CatAvoidEntityGoal/OcelotAvoidEntityGoal overriding
  canContinueToUse; selector runs per candidate (NO_SPECTATORS for fish/panda).
- tick with a dead Weak target only skips the speed refresh; vanilla keeps a
  strong reference, so the difference is the last speed modifier, not a new flee.
