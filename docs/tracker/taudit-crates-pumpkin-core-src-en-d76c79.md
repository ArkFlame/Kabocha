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

## Gap status
- 1 brain-sensor threats reuse goal path logic (avoid_entity.rs:138-238): STILL OPEN, M.
- 2 confirm applied to ranked nearest only (avoid_entity.rs:201-207): STILL OPEN, S, needs decompiled WolfAvoidEntityGoal (changes RNG draws).
- 3 tick_reach searches flee_distance not FOLLOW_RANGE (avoid_entity.rs:175-194): STILL OPEN, S.
- 4 drop_target per-tick clear (avoid_entity.rs:247-263): STILL OPEN, S, needs vanilla override confirmation.
- Default targeting conditions (avoid_entity.rs:69-74): FIXED in the note (already landed); not re-grepped this pass.

## Result

2026-11-17 second pass. Verified each STILL OPEN row against decompiled vanilla.

- Row 1 (brain-sensor threats reuse goal path logic): LEFT OPEN. `AvoidEntityGoal::nearest_of` /
  `Flee::nearest_of` are dead code (no `.nearest_of(` caller in `crates/pumpkin-core/src`).
  Matching vanilla requires porting `VillagerHostilesSensor` -> `SetWalkTargetAwayFrom`
  (`VillagerGoalPackages`: re-arms only when no WALK_TARGET, only within 6 blocks) and wiring
  it to villagers, both outside this unit's files and a cross-crate/cross-file refactor.
- Row 2 (confirm on ranked nearest only): FIXED. Decompiled `Wolf.WolfAvoidEntityGoal.canUse`
  calls `super.canUse()` first (nearest found with targeting selectors only), then applies
  `avoidLlama` to the already-ranked nearest, drawing RNG once. The tracker premise that
  `scares_wolves` is a `predicateOnAvoidEntity` candidate filter is wrong: the 5-arg
  `super(wolf, avoidClass, maxDist, walk, sprint)` sets it to `NO_CREATIVE_OR_SPECTATOR` with
  `avoidPredicate = t -> true`. Moved `confirm` out of the `find_nearest` accept closure into
  `can_start` after path creation, so one RNG draw happens on the nearest threat. Matches the
  `confirmed_by` doc comment ("The nearest threat must also pass filter").
- Row 3 (tick_reach range): already FIXED in the current tree. `tick_reach` reports
  `Attributes::FOLLOW_RANGE` when `per_type.is_some()`, matching `find_threat`'s sensor predicate;
  no change needed.
- Row 4 (drop_target per-tick clear): FIXED by restoring it. Decompiled
  `Wolf.WolfAvoidEntityGoal` overrides both `start()` and `tick()` and calls `setTarget(null)`
  in each. The tracker premise that vanilla clears only once was wrong; `tick()` now clears the
  mob target when `drop_target` is set, matching vanilla's intentional wipe of a target picked
  up mid-flee.
- Tests: no pure unit seam exists for these paths (they need a spawned mob/world and, for
  `confirm`, a controlled RNG); the existing `#[tokio::test]` flee tests cover goal behaviour.
  No test added. Not compiled here per worker rules; orchestrator will batch-check.
- Files touched: crates/pumpkin-core/src/entity/ai/goal/avoid_entity.rs (rustfmt --edition 2024 run, exit 0).
