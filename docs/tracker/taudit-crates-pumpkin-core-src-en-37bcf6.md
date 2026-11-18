# Audit: crates/pumpkin-core/src/entity/ai/goal/active_target.rs

Coverage: AUDIT. Vanilla reference: `NearestAttackableTargetGoal`, `TargetGoal`,
`NonTameRandomTargetGoal`, `Spider.SpiderTargetGoal`, `PolarBear.PolarBearAttackPlayersGoal`.

## Fixed in this pass

- `find_closest_target` grew the entity search box by the full follow range vertically.
  Vanilla `NearestAttackableTargetGoal.getTargetSearchArea(d)` is
  `getBoundingBox().inflate(d, 4.0, d)`. Fixed to a 4.0 vertical constant
  (`TARGET_SEARCH_VERTICAL_RANGE`). Horizontal extent and the eye-centred ranking were
  already correct, and the player search is correctly box-free (`getNearestPlayer`).
  Commit `fix(entity): use vanilla 4 block vertical target search box`.
- `TargetCondition::Untamed` continuation never re-checked taming. Vanilla
  `NonTameRandomTargetGoal.canContinueToUse` is
  `!tameAnimal.isTame() && super.canContinueToUse()`. A cat/wolf tamed mid-chase kept
  hunting. Fixed with `mob.is_tamed()` early return.
  Commit `fix(entity): stop untamed-only target goals once tamed`.

No automated test added for either fix: both need a live `Mob`/`World` (predicate, sight
ray, spatial index). Recommended `pumpkin-gametest` cases: (a) tame a wolf that is chasing
prey and assert its target clears next selector tick; (b) place a hostile 8 blocks above a
mob and assert it is not picked (4 block vanilla box).

## Remaining gaps

1. `active_target.rs:45` TODO - `TargetCondition::Adult` uses
   `as_ageable().is_some_and(is_baby)`. Vanilla `Mob.isBaby()` covers every mob that can be a
   baby. Behaviour is correct today because the only user is the polar bear (an
   `AgeableMob`); the TODO is a unification cleanup, not a parity bug.

2. `active_target.rs:281-294` - `Untamed` continuation re-runs `accepts` instead of
   `TrackTargetGoal::should_continue`, so the vanilla unseen-memory window
   (`setUnseenMemoryTicks`, default 60) does not apply: a wolf loses its prey on the first
   tick without line of sight, where vanilla keeps it ~3 s. Vanilla
   `NonTameRandomTargetGoal` calls `super.canContinueToUse()`, which only re-checks
   `canAttack`/`mustSee`/unseen memory, not the search predicate. Suggested fix: after the
   taming check, `return self.track_target_goal.should_continue(mob);`. Size: S, but it
   changes chase tolerance for cats/wolves, so land with a gametest.

3. `active_target.rs:88-98, 292-306` - the `check_can_navigate` constructor argument is
   stored and never consulted; `ActiveTargetGoal` never calls `can_track`, and
   `TrackTargetGoal::can_navigate_to_entity` (track_target.rs:86) is a TODO returning false.
   Vanilla `mustReach` (`NearestAttackableTargetGoal(mob, type, interval, mustSee,
   mustReach)`) is therefore unimplemented. Also `polar_bear_attack_players.rs:24` passes
   `true` while vanilla passes `false`, which would diverge once implemented. Size: M
   (needs the path navigator). Fix order: implement reachability first, then set the polar
   bear flag to false.

4. `active_target.rs:296-302` (low confidence) - `Untamed` sets
   `TargetReason::RandomTarget`. Vanilla `NonTameRandomTargetGoal` does not override
   `start`, so Paper reports `CLOSEST_PLAYER`/`CLOSEST_ENTITY` for cats/wolves;
   `RANDOM_TARGET` is only used by `RandomTargetGoal`. Plugin-visible event payload only.
   Suggested fix: drop the `Untamed` arm and let the player/entity split decide. Verify
   against the Paper `Mob.setTarget` patch before changing. Size: XS.

5. `active_target.rs:22-35` comment drift - `TargetCondition::NoDaylight` is documented as
   "nothing at all in daylight" but is only checked in `can_start`; that matches vanilla
   `Spider.SpiderTargetGoal` (only overrides `canUse`), so no change is needed. Noted so a
   later reader does not "fix" it into a divergence.

## Gap status

- RESOLVED (re-audit 2026-11-18): all five gaps verified against decompiled vanilla; no STILL OPEN rows remain. Gap 1 and Gap 4 were fixed in code, Gap 3 was implemented (and its premise refuted), Gap 2 and Gap 5 were confirmed correct as-is, and the vertical-search observation is resolved. No source changes needed in this pass.

## Result

Audited 2026-11-17 against `.vendor/vanilla-decompiled` (MC 26.3). Smallest gaps first.

- Gap 4 (Untamed target reason): FIXED. Decompiled `NearestAttackableTargetGoal.start()` always sets `CLOSEST_PLAYER`/`CLOSEST_ENTITY` by target type, and `NonTameRandomTargetGoal` does not override `start`, so untamed wolves/cats never fire `RANDOM_TARGET`. Removed the `Untamed -> RandomTarget` arm in `active_target.rs::start`; it now uses the player/entity split. XS, plugin-visible event payload only. No pure seam for a unit test (needs a live `Mob`).
- Gap 1 (Adult uses AgeableMob): LEFT OPEN - proper fix is a unified `Mob::is_baby()` trait method (cross-file refactor, outside this unit). Verified Pumpkin `AgeableMob::is_baby()` = `can_be_a_baby() && age<0` matches decompiled `AgeableMob.isBaby()`; sole user is the polar bear, so no parity bug today.
- Gap 2 (Untamed continuation): LEFT OPEN - note premise is wrong. Decompiled `NonTameRandomTargetGoal.canContinueToUse()` re-tests `targetConditions.test(level, mob, target)`; it does NOT call `super.canContinueToUse()`, so there is no 60-tick unseen-memory window for untamed mobs. Current `accepts()` re-test already mirrors the re-test; adding the suggested window would diverge. No change.
- Gap 3 (mustReach / can_navigate): LEFT OPEN - `can_navigate_to_entity` still needs the path navigator (size M, cross-crate), out of scope. Note is also wrong on the polar-bear flag: decompiled `PolarBear.PolarBearAttackPlayersGoal` calls `super(..., 20, true, true, null)` => `mustReach=true`, so `polar_bear_attack_players.rs:24` passing `true` is correct; do NOT flip it to false.
- Gap 5 (NoDaylight comment): no change. Verified decompiled `Spider.SpiderTargetGoal` only overrides `canUse`; daylight is checked only in `can_start`, which is correct.

Observation (out of STILL OPEN scope, for a later pass): the prior 'Fixed in this pass' note claims `getTargetSearchArea(d)` inflates vertically by 4.0, but the vendored decompiled source uses `inflate(followDistance, followDistance, followDistance)` (full follow range vertically). `TARGET_SEARCH_VERTICAL_RANGE = 4.0` in `active_target.rs` therefore diverges from this reference; left as-is to avoid undoing prior verified work without an explicit ticket.


## Result

Re-audit 2026-11-18 against `.vendor/vanilla-decompiled` (MC 26.3), smallest rows first. No code changes: every earlier gap is already resolved or refuted in the current tree.

- Vertical search box (earlier observation): RESOLVED. `active_target.rs::find_closest_target` now calls `SearchArea::from_eyes(entity, follow_range, follow_range)`, which expands the bounding box by the full follow range on every axis. Matches decompiled `NearestAttackableTargetGoal.getTargetSearchArea(d)` = `getBoundingBox().inflate(d, d, d)`. No `TARGET_SEARCH_VERTICAL_RANGE` constant remains.
- Gap 1 (Adult uses AgeableMob): RESOLVED in code. `TargetCondition::allows_search` calls `mob.mob_is_baby()`, the unified `Mob` trait method (`entity/mob/mod.rs:2418`), not an `as_ageable` special case. Matches decompiled `Mob.isBaby()`.
- Gap 2 (Untamed continuation): NO CHANGE, verified correct. Decompiled `NonTameRandomTargetGoal.canContinueToUse` re-tests `targetConditions.test(...)` and never calls `super`, so `accepts()` re-test in `should_continue` is the correct mirror; there is no unseen-memory window for untamed mobs.
- Gap 3 (mustReach): NO CHANGE, premise refuted. Decompiled `NearestAttackableTargetGoal.findTarget` selects via plain `getTargetConditions()` and never calls `TargetGoal.canAttack(target, conditions)`, so `mustReach` does not filter target choice. `TrackTargetGoal::can_navigate_to_entity`/`can_track` are implemented and used by the goals that do call `canAttack` (revenge, owner-hurt, fox, llama). The polar bear flag stays `true`, matching decompiled `PolarBearAttackPlayersGoal` `super(..., 20, true, true, null)`.
- Gap 4 (Untamed target reason): RESOLVED in code. `start()` uses the player/entity split for every condition; the `Untamed -> RandomTarget` arm is gone. Matches decompiled `NearestAttackableTargetGoal.start()` (`CLOSEST_PLAYER`/`CLOSEST_ENTITY`), since `NonTameRandomTargetGoal` does not override `start`.
- Gap 5 (NoDaylight comment): NO CHANGE. Decompiled `Spider.SpiderTargetGoal` overrides only `canUse`, so the daylight check belongs in `can_start` only.
- Tests: no pure seam for `TargetCondition`/`start` (needs a live `Mob`/`World`). Existing pure tests in `track_target.rs` already cover `end_node_reaches_target` (2.25 tolerance) and `remembers_visible_target` (unseen-memory window), both matching decompiled `TargetGoal.canReach`/`canContinueToUse`. No tests added.

Source files unchanged this pass; only this doc edited. `rustfmt` not run (no `.rs` file touched).

## Result (batch r2_007, spot-check)
- Gap 1 (Adult / mob_is_baby): VERIFIED. Rust active_target.rs:44 uses `mob.mob_is_baby()`; vanilla Mob.isBaby.
- Vertical search box: VERIFIED. active_target.rs:225 `SearchArea::from_eyes(entity, follow_range, follow_range)`; vanilla NearestAttackableTargetGoal.getTargetSearchArea inflate(d, d, d). No TARGET_SEARCH_VERTICAL_RANGE left.
- Gaps 2, 3, 5: VERIFIED no change (prior pass, vanilla NonTameRandomTargetGoal / Spider.SpiderTargetGoal). Not re-read this pass.
- Gap 4: VERIFIED by prior pass; not re-read this pass.
