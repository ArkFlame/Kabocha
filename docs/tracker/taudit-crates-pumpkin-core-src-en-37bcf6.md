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
