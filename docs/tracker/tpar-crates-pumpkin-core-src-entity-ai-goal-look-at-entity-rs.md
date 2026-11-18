# Parity rows for crates/pumpkin-core/src/entity/ai/goal/look_at_entity.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-en-2cf6d4** [partial/OWN-OPEN] own audit: 9 open, 2 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-en-2cf6d4.md) docs/tracker/taudit-crates-pumpkin-core-src-en-2cf6d4.md

## Result
Third pass: every row re-verified directly against the vendored decompiled vanilla. The
linked audit's own Result sections already closed all six rows; this pass confirmed them on
disk. All rows match vanilla, so no Pumpkin source file needed editing (no rustfmt run and no
new test added: the three existing look_at_entity tests already lock the behaviour).

- Row 1 (entity search inflates the box 3 blocks vertically) - VERIFIED. `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/goal/LookAtPlayerGoal.java:67` searches `getBoundingBox().inflate(lookDistance, 3.0, lookDistance)`. `look_at_entity.rs:193` `SearchArea::from_eyes(entity, self.range.into(), 3.0)` expands by (range, 3.0, range) and ranks from eye height. Keep 3.0.
- Row 2 (player reach 10x too large) - FIXED in an earlier pass, VERIFIED here. `LookAtPlayerGoal.java:44/46` bounds candidates by `range(lookDistance)`; `look_at_entity.rs:156` declares `players: range` and `create_target_predicate` sets `base_max_distance = f64::from(range)` (:136). The old `MAX_VISIBILITY` 10x bound is gone.
- Row 3 (`look_forward` dead) - VERIFIED. `LookAtPlayerGoal.java:97` uses own eye Y when `onlyHorizontal`; `look_at_entity.rs:242-246` drives it identically, and `shulker.rs:94-100` passes `8.0/0.02/true` (the sole `true` caller, matching `Shulker.java:101`). Other in-scope callers (`interact.rs:20`, `panda.rs:281/1209`, `pillager.rs:81/91`, `wandering_trader.rs`) pass false. No wiring needed.
- Row 4 (rider filter) - VERIFIED. `LookAtPlayerGoal.java:42-44` applies `EntitySelector.notRiding(mob)` for `Player.class`; `look_at_entity.rs:137-143` uses `EntityPredicate::Rides(mob_arc.get_entity())`, which keeps non-riders and rejects riders. Keep the filter.
- Row 5 (slot target not range-checked when set) - VERIFIED. `Panda.java:82` BREED_TARGETING range 8.0 vs look goal 6.0 (:276); `PandaLookAtPlayerGoal.canUse` (`Panda.java:935-959`) only searches when `lookAt == null` and does not recheck a preset target, and `canContinueToUse` (:930-932 via `LookAtPlayerGoal.java:80-82`) drops it past 6.0. `look_at_entity.rs:174-175` sets the slot target without a clamp and :217 drops it past `range`. Do not clamp.
- Row 6 (`Controls::LOOK` vs empty flag set) - VERIFIED. `LookAtPlayerGoal.java:41` calls `setFlags(EnumSet.of(Goal.Flag.LOOK))`; `look_at_entity.rs:88` `goal_control: Controls::LOOK` matches. No arbitration change needed.
- Visibility / distance scaling (regression fixed in an earlier pass) - VERIFIED. `LookAtPlayerGoal.java:44/46` uses `TargetingConditions.forNonCombat()`, which keeps line of sight and scales range by visibility with a 2.0 floor (`ai/targeting/TargetingConditions.java`). `create_target_predicate` uses `TargetPredicate::create_non_attackable()` (:135) and the test `an_invisible_target_is_only_found_within_two_blocks` locks the floor.
- Wandering trader `LookAtTradingPlayerGoal` (fixed in an earlier pass) - VERIFIED. `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/goal/LookAtTradingPlayerGoal.java:15-22` `canUse` only checks `isTrading`; `wandering_trader.rs:884-888` `can_start` only requires a trading player and :890-897 `should_continue` keeps the 8-block test (inherited `LookAtPlayerGoal.java:80-82`).

No source file changed this pass; no rustfmt or new test needed (behaviour already locked by the three look_at_entity tests). fixed=0 open=0.
