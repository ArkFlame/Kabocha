# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-2cf6d4.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

All five open markers closed. fixed=0 open=0.
No rows remain open. fixed=0 open=0 for this pass.

## Result (FINAL PASS - every row terminal, re-verified against vendored vanilla)

Re-read the on-disk source for every in-scope file and re-checked each row against the
decompiled vanilla under `.vendor/vanilla-decompiled/net/minecraft/` this pass. No source
file needed editing: every row was already in the settled, vanilla-matching state from
earlier passes (row 2 was FIXED in an earlier pass). No .rs file was touched, so rustfmt
was not run and no new test was added (the three existing look_at_entity.rs tests already
lock the behaviour). Rows in order:

1. Entity box inflated 3 blocks vertically - VERIFIED.
   `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/goal/LookAtPlayerGoal.java:67`
   searches non-player entities with `getBoundingBox().inflate(lookDistance, 3.0, lookDistance)`.
   `look_at_entity.rs:193` `SearchArea::from_eyes(entity, self.range.into(), 3.0)` expands the
   box by (range, 3.0, range). Matches; keep 3.0.
2. Player reach 10x too large - FIXED (earlier pass) and VERIFIED. `LookAtPlayerGoal.java:44`
   uses `TargetingConditions.forNonCombat().range(lookDistance)`; `look_at_entity.rs:156`
   declares `players: if looks_at_players { range } else { 0.0 }` (the 10x MAX_VISIBILITY
   bound is gone).
3. `look_forward` dead - VERIFIED. `LookAtPlayerGoal.java:97` uses the mob's own eye Y when
   `onlyHorizontal`; `look_at_entity.rs:242-246` drives it identically. Shulker.java:101 is
   the sole `onlyHorizontal=true` caller and `shulker.rs:94-100` passes `8.0 / 0.02 / true`.
   All other callers pass false via `with_default` or an explicit `false`.
4. Rider filter - VERIFIED. `LookAtPlayerGoal.java:42-44` applies `EntitySelector.notRiding(mob)`
   for Player.class; `look_at_entity.rs:137-143` installs `EntityPredicate::Rides`, rejecting a
   rider. Keep the filter.
5. Slot target not range-checked at set time - VERIFIED. Vanilla PandaLookAtPlayerGoal.canUse
   (`.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/panda/Panda.java:940`) only
   searches while `lookAt == null` (BREED_TARGETING range 8.0 at :82 vs goal 6.0 at :276) and
   does not recheck a preset target; canContinueToUse (:930-931, via LookAtPlayerGoal.java:81)
   range-drops it. `look_at_entity.rs:174-176` keeps a preset target and :217-219 drops it past
   `range`. Do not clamp.
6. Controls::LOOK vs empty flag set - VERIFIED. `LookAtPlayerGoal.java:41` calls
   `setFlags(EnumSet.of(Goal.Flag.LOOK))`; `look_at_entity.rs:88` `goal_control: Controls::LOOK`
   matches.

Visibility / distance scaling - VERIFIED. `LookAtPlayerGoal.java:44` uses
`TargetingConditions.forNonCombat().range(...)`, which keeps line of sight and scales range by
the target's visibility percent with a 2 block floor. `look_at_entity.rs:135-136` uses
`TargetPredicate::create_non_attackable()` with `base_max_distance = f64::from(range)`; the
test `an_invisible_target_is_only_found_within_two_blocks` locks the floor.

Wandering trader LookAtTradingPlayerGoal - VERIFIED.
`.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/goal/LookAtTradingPlayerGoal.java:15-22`
checks only `isTrading` in canUse (8.0F is the inherited lookDistance at :10).
`crates/pumpkin-core/src/entity/passive/wandering_trader.rs:884-888` can_start only requires a
trading player and :890-897 should_continue keeps the 8-block test (earlier-pass fix, present).

All rows terminal: row 2 FIXED in an earlier pass, all rows VERIFIED against vendored vanilla,
0 BLOCKED-DATA, 0 remaining open. fixed=0 open=0 for this pass.
