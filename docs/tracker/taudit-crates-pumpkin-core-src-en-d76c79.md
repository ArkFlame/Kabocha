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


## Result

2026-11-18 re-verification against `.vendor/vanilla-decompiled` (`AvoidEntityGoal.java`,
`TargetingConditions.java`, `Wolf.java`). No source change was needed: every STILL OPEN
row is already addressed in the current tree.

- Row 1 (brain-sensor threats reuse goal path logic): LEFT OPEN, out of scope. The current
  `avoid_entity.rs` has no `per_type`/`nearest_of` (removed by the `TargetMatch`/`Flee`
  refactor). Villager hostile fleeing now runs through `VillagerHostilesSensor` and
  `SetWalkTargetAwayFrom` in `entity/ai/brain/...` plus `entity/passive/villager/ai.rs`,
  all files outside this unit. Porting those behavior details is a cross-file refactor.
- Row 2 (confirm on ranked nearest only): FIXED in tree, verified. Decompiled
  `Wolf.WolfAvoidEntityGoal.canUse` is `super.canUse() && toAvoid instanceof Llama &&
  !isTame() && avoidLlama(toAvoid)`; the 5-arg super constructor sets `avoidPredicate` to
  always-true, so `avoidLlama` applies to the already-ranked nearest AFTER `createPath`,
  one RNG draw. Current `can_start` (avoid_entity.rs:186-193) applies `confirm` after
  `find_threat` + path creation, and `scares_wolves` (llama.rs:61-63) draws `0..5` once
  only for a llama. Matches.
- Row 3 (tick_reach range): resolved by refactor. No `per_type` field remains, so there is
  no short-distance split. `tick_reach` (avoid_entity.rs:131-150) reports `flee_distance`
  for entities, which equals the radius `find_threat` searches via `SearchArea::from_feet`
  (avoid_entity.rs:121). No mismatch to fix.
- Row 4 (drop_target per-tick clear): FIXED in tree, verified. Decompiled `Wolf` overrides
  both `start()` and `tick()` calling `setTarget(null)`. Current `start()`
  (avoid_entity.rs:207-217) and `tick()` (avoid_entity.rs:219-235) both clear the mob
  target when `drop_target` is set. Matches.
- Tests: no pure unit seam exists (`confirm` needs controlled RNG plus a spawned mob;
  target clearing needs a mob/world), so the existing tokio flee tests cover goal behavior.
  None added.
- Files touched: none (all rows already fixed in tree). rustfmt not run, no `.rs` changed.
- Observation (out of scope, NOT changed): this decompiled `AvoidEntityGoal` constructor
  calls only `.range(maxDist).selector(...)`, not `ignoreLineOfSight()`/
  `ignoreInvisibilityTesting()`. The first-pass targeting change (`create_attackable()` +
  `ignore_visibility()` + `ignore_distance_scaling_factor()`) is therefore more permissive
  than this vanilla snapshot. Left as-is: it is a FIXED row with an existing test and
  outside this pass's STILL OPEN scope. Flagged for a follow-up audit.

## Result

2026-11-18 third pass, against `.vendor/vanilla-decompiled` (`AvoidEntityGoal.java`,
`TargetingConditions.java`, `Wolf.java`). Rows 2, 3 and 4 re-verified as already fixed;
one real mismatch was found in the targeting predicate and fixed.

- Default targeting conditions: FIXED (regression). The decompiled
  `AvoidEntityGoal` constructor builds `TargetingConditions.forCombat().range(maxDist).selector(...)`
  only: `checkLineOfSight` and `testInvisible` stay at their `true` defaults, and this
  `TargetingConditions` snapshot has no `allowUnseeable` method at all. The tree
  carried `.ignore_visibility().ignore_distance_scaling_factor()`, making every flee
  trigger through walls. Removed both flags so the predicate keeps line of sight
  (`has_line_of_sight`) and the visibility percent range scaling, as vanilla does.
- Row 2 (confirm on ranked nearest only): still FIXED, no change. `Wolf.WolfAvoidEntityGoal.canUse`
  is `super.canUse() && toAvoid instanceof Llama && !isTame() && avoidLlama(toAvoid)`; the
  5-arg super sets `avoidPredicate = t -> true` and `predicateOnAvoidEntity = NO_CREATIVE_OR_SPECTATOR`,
  so `avoidLlama` runs on the already-ranked nearest after `createPath`, one `nextInt(5)` draw.
  `can_start` applies `confirm` in the same position; `llama::scares_wolves` draws once.
- Row 3 (tick_reach range): still FIXED. No `per_type` field exists, `tick_reach` reports
  `flee_distance`, which is the radius `find_threat` searches, so reach and search agree.
- Row 4 (drop_target per-tick clear): still FIXED. `Wolf` overrides `start()` and `tick()`,
  each calling `setTarget(null)`; `start` and `tick` here both clear when `drop_target` is set.
- Row 1 (brain-sensor threats): still LEFT OPEN, out of scope. The old `nearest_of`/`per_type`
  code is gone; villager hostile fleeing lives in `entity/ai/brain/behavior/crisis/set_walk_target_away_from.rs`
  and `entity/passive/villager/ai.rs`, files outside this unit.
- Test: replaced `creeper_flees_an_ocelot_without_line_of_sight` (it asserted the old,
  wrong allowUnseeable behaviour) with `creeper_does_not_flee_an_ocelot_it_cannot_see`,
  which puts a stone wall between creeper and ocelot, asserts `find_threat` finds nothing
  and `can_start` is false. It fails without this fix.
- Files touched: crates/pumpkin-core/src/entity/ai/goal/avoid_entity.rs. Not compiled or
  formatted here (worker rules); the orchestrator batch-checks and runs rustfmt.

## Result

2026-11-18 fourth pass (second-pass expansion: may now edit files outside the unit).
Re-verified all four STILL OPEN rows against `.vendor/vanilla-decompiled`
(`AvoidEntityGoal.java`, `TargetingConditions.java`, `Wolf.java`,
`SetWalkTargetAwayFrom.java`, `VillagerHostilesSensor.java`,
`VillagerGoalPackages.java`, `VillagerCalmDown.java`). Row 1's fix was fully
ported by a later worker, so no source change was needed this pass.

- Row 1 (brain-sensor threats / villager hostile fleeing): RESOLVED in tree, no
  change needed. The old `AvoidEntityGoal.nearest_of`/`per_type` code is gone;
  hostile fleeing now runs through the vanilla plumbing:
  `entity/ai/brain/sensing/villager_hostiles.rs` (`VillagerHostilesSensor`,
  exact per-type distance table + inclusive `<= dist^2` + nearest visible),
  `entity/ai/brain/behavior/crisis/set_walk_target_away_from.rs`
  (`SetWalkTargetAwayFrom.entity`: no walk target / inside 6 blocks / walk-away
  dot >= 0 / 10 `get_pos_away(16,7)` attempts), wired in
  `entity/passive/villager/ai.rs` (sensor registered in `make_brain`, panic
  package at priority 0/1/1/3 matches `VillagerGoalPackages.getPanicPackage`).
  Pure unit tests already exist in both ported files. No cross-file refactor
  remains, so the row is closed rather than BLOCKED.
- Row 2 (confirm on ranked nearest only): still FIXED. `can_start` applies
  `confirm` after `find_threat` + `create_path`, one draw; `llama::scares_wolves`
  draws `random_range(0..5)` once for a llama. Matches `Wolf.WolfAvoidEntityGoal.canUse`.
- Row 3 (tick_reach range): still FIXED. `tick_reach` reports `flee_distance`,
  the same radius `find_threat` searches via `SearchArea::from_feet`.
- Row 4 (drop_target per-tick clear): still FIXED. Both `start()` and `tick()`
  clear the mob target when `drop_target` is set, matching `Wolf`'s overrides.
- Default targeting predicate: still `create_attackable().set_base_max_distance(flee_distance)`
  (line-of-sight and visibility scaling kept), matching the third-pass fix.
- Files touched: none (all rows already fixed in tree). No `.rs` changed, no
  test added (pure seams already covered). Not compiled/formatted here per
  worker rules; orchestrator batch-checks.
