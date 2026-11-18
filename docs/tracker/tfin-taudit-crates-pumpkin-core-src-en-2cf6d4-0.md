# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-2cf6d4.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

Re-audited every open row against the vendored decompiled vanilla
Re-checked all six open rows against `.vendor/vanilla-decompiled/.../LookAtPlayerGoal.java`,
The three passes above already resolved every open row; this continuation re-read the
Re-verified every open row directly against the vendored decompiled vanilla

## Result
FINAL PASS: re-read every in-scope source file and the vendored decompiled vanilla under
`.vendor/vanilla-decompiled/net/minecraft/` this pass. All rows are already in their correct
terminal state; no `.rs` file needed editing, so rustfmt was not run and no new test was added
(behaviour is locked by the three existing `look_at_entity.rs` tests). Per row:

1. Entity box inflated 3 blocks vertically - VERIFIED.
   `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/goal/LookAtPlayerGoal.java:67`
   searches with `getBoundingBox().inflate(lookDistance, 3.0, lookDistance)`.
   `look_at_entity.rs:193` `SearchArea::from_eyes(entity, self.range.into(), 3.0)`
   expands by (range, 3.0, range); matches. Keep 3.0.
2. Player reach declared 10x too large - FIXED (earlier pass) and VERIFIED.
   `LookAtPlayerGoal.java:44` uses `TargetingConditions.range(lookDistance)`;
   `look_at_entity.rs:151,156` declares `players: if looks_at_players { range } else { 0.0 }`
   with `range = f64::from(self.range)`; the old `range * MAX_VISIBILITY` bound is gone.
3. `look_forward` dead - VERIFIED. `LookAtPlayerGoal.java:97` uses the mob's own eye Y when
   `onlyHorizontal`; `look_at_entity.rs:242-246` drives it identically. `Shulker.java:101` is
   the sole `onlyHorizontal=true` caller and `shulker.rs:94-100` passes `8.0 / 0.02 / true`.
   All in-scope callers pass false, matching every other vanilla caller.
4. Player filter keeps riders out - VERIFIED. `LookAtPlayerGoal.java:42-44` applies
   `EntitySelector.notRiding(mob)` for `Player.class`; `look_at_entity.rs:138-143` installs
   `EntityPredicate::Rides`, which rejects a rider. Keep the filter.
5. Slot target not range-checked when set - VERIFIED.
   `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/panda/Panda.java:82` sets
   `BREED_TARGETING` range 8.0 and `:276` the look goal 6.0; `PandaLookAtPlayerGoal.canUse`
   (`Panda.java:935-959`) only searches when `lookAt == null` and does not recheck a preset
   target, while `canContinueToUse` (`:930-932`, via `LookAtPlayerGoal.java:80-82`) drops it
   past range. `look_at_entity.rs:174-175` keeps a preset target and `:217-219` drops it past
   range. Do not clamp.
6. Controls::LOOK vs empty flag set - VERIFIED. `LookAtPlayerGoal.java:41` calls
   `setFlags(EnumSet.of(Goal.Flag.LOOK))`; `look_at_entity.rs:88` `goal_control: Controls::LOOK`
   matches.

Visibility / distance scaling - VERIFIED.
`.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/targeting/TargetingConditions.java:14-15,26,88-89,96`
keeps line of sight and scales range by visibility with a 2.0 floor
(`MIN_VISIBILITY_DISTANCE_FOR_INVISIBLE_TARGET = 2.0`); `look_at_entity.rs:135-136` uses
`TargetPredicate::create_non_attackable()` with `base_max_distance = f64::from(range)`
(`target_predicate.rs:8 MIN_DISTANCE=2.0`, `:25-26` both checks on); the test
`an_invisible_target_is_only_found_within_two_blocks` locks the floor.

Wandering trader `LookAtTradingPlayerGoal` - VERIFIED.
`.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/goal/LookAtTradingPlayerGoal.java:15-22`
checks only `isTrading` in `canUse` (the 8.0 is the inherited lookDistance);
`wandering_trader.rs:884-888` `can_start` only requires a trading player and `:890-897`
`should_continue` keeps the 8-block test (earlier-pass fix).

All 6 rows terminal: row 2 FIXED earlier, all 6 VERIFIED against vendored vanilla,
0 BLOCKED-DATA, 0 remaining open. Status: PASS fixed=0 open=0.
