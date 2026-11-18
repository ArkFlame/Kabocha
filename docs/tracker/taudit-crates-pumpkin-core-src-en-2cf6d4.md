# Tracker: audit / entity/ai/goal/look_at_entity.rs

Source under review: `crates/pumpkin-core/src/entity/ai/goal/look_at_entity.rs`.
No decompiled reference is vendored (`.vendor/` absent); vanilla
`net.minecraft.world.entity.ai.goal.LookAtEntityGoal` / `LookAtPlayerGoal` (1.21.x)
compared from knowledge, plus the callers in `crates/pumpkin-core/src/entity/**`.

Markers in the file: no `TODO`/`FIXME` left (one `// TODO` on `base_max_distance` was
already replaced by `f64::from(range)`); the two `vanilla` mentions are doc comments
(`:13`, `:102`) documenting `PandaLookAtPlayerGoal`, not gaps.

## Fixed in this pass
- Candidate search applied the targeting predicate (`respects_visibility`,
  `use_distance_scaling_factor`), so a look-at target needed line of sight and its range
  shrank with `LivingEntity.getVisibilityPercent` (sneaking 0.8, invisible down to 0.07).
  Vanilla `LookAtEntityGoal.canUse` only ranks by distance (`getNearestPlayer(mob, dist)`
  and `getEntitiesOfClass(type, box.inflate(dist), this::canLookAt)`), so mobs do look at
  players behind walls and at invisible ones. `create_target_predicate` now calls
  `ignore_visibility().ignore_distance_scaling_factor()`; test
  `an_invisible_target_is_still_looked_at`.

## Remaining gaps

### 1. Entity search inflates the box 3 blocks vertically (S)
`look_at_entity.rs:195` uses `SearchArea::from_eyes(entity, range, 3.0)`; vanilla
`LookAtEntityGoal` inflates only horizontally (`getBoundingBox().inflate(lookDistance)`).
`AvoidEntityGoal` is the vanilla goal with `inflate(dist, 3.0, dist)`, so the 3.0 looks
copied from there: a target 2 blocks above a cow can be picked although vanilla cannot.
Suggested fix: pass `0.0` vertical and confirm against the decompiled class. Size: S.

### 2. Player reach is declared 10x too large (S) - FIXED fix(core): bound look-at player reach by range
`look_at_entity.rs:155` sets `players: range * MobVisibilityImpl::MAX_VISIBILITY` (10x).
`MAX_VISIBILITY` is the bound of the `minecraft:mob_visibility` data component, not a
game range; the predicate's real maximum is `range * visibility_percent <= range`.
Over-declaring only coarsens `entity_islands` (no gameplay effect), but it is wrong.
Suggested fix: `players: range`. Size: S.

### 3. `look_forward` is dead (S)
`look_at_entity.rs:55` / `:244` implement vanilla's only-horizontal look (target Y = the
mob's own eye Y), but every caller passes `false`
(`with_default`, `LookAtEntityGoal::new(..., false)`, `interact.rs:20`,
`wandering_trader.rs:251`, `pillager.rs:82`). Either wire the vanilla callers that use
`onlyHorizontal` or drop the flag. Size: S.

### 4. Player filter keeps riders out (S)
`look_at_entity.rs:136-141` adds `EntityPredicate::Rides(mob)`, i.e. it rejects a player
who rides the mob. Vanilla `getNearestPlayer` has no such filter (spectators are already
excluded by `can_be_seen_by_anyone`). Verify against `PandaLookAtPlayerGoal`, which is the
only vanilla reason for this slot, then drop the filter if it is not vanilla. Size: S.

### 5. Slot target is not range-checked when set (S)
`look_at_entity.rs:176` returns `true` for any target set through `LookAtSlot`.
`PandaBreedGoal::nearest_player` searches `SULK_LOOK_RANGE` (8.0) while the goal's
`lookTime`/`should_continue` range is 6.0 (`panda.rs:261`), so a target 6-8 blocks away
starts the goal and is dropped on the next tick. Confirm vanilla
`PandaLookAtPlayerGoal`/sulk behaviour and either clamp the slot target to `range` or give
the panda goal the matching distance. Size: S.

### 6. `Controls::LOOK` vs vanilla's empty flag set (S)
`look_at_entity.rs:89` claims `LOOK`. Vanilla `LookAtEntityGoal` calls no `setFlags`, so
it can run alongside `RandomLookAroundGoal` (the later goal wins the look control each
tick). Pumpkin's flag makes the two mutually exclusive. Matching vanilla would need the
goal arbitration to allow overlapping flags; low behavioural impact. Size: S/M.
VERIFIED (1): .vendor/vanilla-decompiled/net/minecraft/world/entity/ai/goal/LookAtPlayerGoal.java:67 inflates (lookDistance, 3.0, lookDistance); SearchArea::from_eyes(..., 3.0) in look_at_entity.rs matches. Keep 3.0.
VERIFIED (3): Shulker.java:101 is the sole onlyHorizontal=true caller; shulker.rs:94-100 passes 8.0/0.02/true. All in-scope callers pass false. Nothing to wire.
VERIFIED (4): LookAtPlayerGoal.java:42-44 applies EntitySelector.notRiding(mob) for Player.class; EntityPredicate::Rides (predicate/mod.rs:47-63) returns false for a rider. Keep the filter.
VERIFIED (5): Panda.java:82 BREED_TARGETING range 8.0 vs look goal 6.0 (:276); PandaLookAtPlayerGoal.canUse does not recheck a preset target (:925-959) and canContinueToUse (:930-932, super :81) drops it past range. Pumpkin matches; do not clamp.
VERIFIED (6): LookAtPlayerGoal.java:41 calls setFlags(EnumSet.of(Goal.Flag.LOOK)); goal_control: Controls::LOOK matches. No arbitration change needed.


## Result

Re-audited every STILL OPEN row against the vendored decompiled vanilla
(.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/goal/LookAtPlayerGoal.java,
world/entity/animal/panda/Panda.java and the full new LookAtPlayerGoal(...) caller list).
No Pumpkin source change was required: each open row rested on a premise that the missing
decompiled reference now disproves. No .rs file was touched, so rustfmt was not run and
no new unit test was added (there is no behaviour change to lock in).

### 1. Entity box inflated 3 blocks vertically - CONFIRMED VANILLA (no change)
LookAtPlayerGoal.java:67 searches non-player entities in
getBoundingBox().inflate(lookDistance, 3.0, lookDistance) - the 3.0 vertical is genuinely
vanilla, copied from this very goal (not AvoidEntityGoal as the audit guessed).
SearchArea::from_eyes(entity, range, 3.0) expands the box by (range, 3.0, range) and ranks
from eye height, so the current value matches. The audit's "horizontal only" suggestion was
based on a misremembered signature; do not pass 0.0.

### 3. look_forward flag - IMPLEMENTATION CORRECT; only caller out of scope (LEFT OPEN)
LookAtPlayerGoal.java:21,97 drives onlyHorizontal exactly as tick already does
(self.look_forward ? own eye Y : target eye Y). Across all decompiled callers the only one
passing true is Shulker (Shulker.java:101: new LookAtPlayerGoal(this, Player.class, 8.0F, 0.02F, true));
every other caller, including all in-scope files, passes false and already matches.
Left open solely because wiring Shulker needs shulker.rs, outside this unit's files.

### 4. Rider filter - CONFIRMED VANILLA (no change)
LookAtPlayerGoal.java:42-44 applies EntitySelector.notRiding(mob) to the targeting
conditions for every Player.class look-at goal (inherited unchanged by
PandaLookAtPlayerGoal, Panda.java:917-967). EntityPredicate::Rides returns false for a
rider, so the goal's predicate keeps non-riders and rejects riders - identical to vanilla.
The audit's "getNearestPlayer has no such filter" claim was wrong; keep the filter.

### 5. Slot target not range-checked at set time - CONFIRMED VANILLA (no change)
PandaBreedGoal.canUse (Panda.java:823-824) sets the look target from BREED_TARGETING,
range 8.0 (Panda.java:82), while PandaLookAtPlayerGoal has lookDistance 6.0
(Panda.java:276). Vanilla PandaLookAtPlayerGoal.canUse (:935-959) does NOT range-check a
preset target; canContinueToUse (:930-932, via super :80-82) drops it once past 6.0.
Pumpkin's "start at any distance, drop on next tick" behaviour matches this exactly. Do not
clamp the slot target and do not widen the goal's distance.

### 6. Controls::LOOK vs "empty flag set" - CONFIRMED VANILLA (no change)
LookAtPlayerGoal.java:41 calls setFlags(EnumSet.of(Goal.Flag.LOOK)). Pumpkin's
goal_control: Controls::LOOK matches; no goal-arbitration change is needed and the two
goals are mutually exclusive in vanilla too. The audit's "calls no setFlags" premise was wrong.

## Result (second pass, decompiled vanilla available)

### REGRESSION FOUND AND REVERTED: visibility / distance scaling in the look predicate
The "Fixed in this pass" change above was wrong. `LookAtPlayerGoal.java:44,46` targets with
`TargetingConditions.forNonCombat()`, which leaves `checkLineOfSight = true` and
`testInvisible = true` (TargetingConditions.java:14-15, 96-97) and applies `range(range)`
(:39). So vanilla look-at goals *do* require line of sight and *do* scale the range by
`getVisibilityPercent` with the `max(range * modifier, 2.0)` floor (:87-93).
`look_at_entity.rs:create_target_predicate` now uses plain
`TargetPredicate::create_non_attackable()` (visibility and distance scaling both kept) and the
test asserts the 2 block floor: an invisible cow is rejected at 4 blocks and accepted at 1.
The old `an_invisible_target_is_still_looked_at` test encoded the deviation and is replaced by
`an_invisible_target_is_only_found_within_two_blocks`.

### 3. look_forward - CLOSED (already vanilla)
`Shulker.java:101` is the only `LookAtPlayerGoal(..., true)` caller in the whole decompiled
tree, and `crates/pumpkin-core/src/entity/mob/shulker.rs:94-100` already passes `true` with
8.0 / 0.02. Every other goal keeps `false`. Nothing left to wire.

### Wandering trader LookAtTradingPlayerGoal - FIXED
`LookAtTradingPlayerGoal.java:15-22` only checks `villager.isTrading()` in `canUse`; the
`lookDistance` of 8.0 lives in the inherited `canContinueToUse`. Pumpkin's
`wandering_trader.rs` also range-checked in `can_start`, so a trading player further than 8
blocks never started the goal (and the trader never turned to look). `can_start` now only
requires a trading player; `should_continue` keeps the 8 block test. No decompiled panda
equivalent of the attack-target copy was needed: `LookAtPlayerGoal.java:56-58` assigns
`getTarget()` but lines 62/64 always overwrite it, and `PandaLookAtPlayerGoal.canUse`
(Panda.java:935-959) does not read the attack target at all, so Pumpkin's slot path already
matches.

### 1, 4, 5, 6 - CONFIRMED VANILLA, no change
Same conclusions as the previous pass, re-verified against the vendored sources:
`inflate(lookDistance, 3.0, lookDistance)` (LookAtPlayerGoal.java:67),
`EntitySelector.notRiding(mob)` (:43), the preset-target path in `PandaLookAtPlayerGoal`
(Panda.java:925-959) and `setFlags(EnumSet.of(Goal.Flag.LOOK))` (:41).


## Result (third pass, re-verified against vendored decompiled vanilla)

Re-checked all six STILL OPEN rows against `.vendor/vanilla-decompiled/.../LookAtPlayerGoal.java`,
`targeting/TargetingConditions.java`, `animal/panda/Panda.java` and the full
`new LookAtPlayerGoal(...)` caller list. Every row is already correct in Pumpkin; no source file
was changed, so rustfmt was not run and no new test was added (the three existing tests already
lock in the behaviour).

- Row 1 (box inflated 3.0 vertically) - CONFIRMED VANILLA. LookAtPlayerGoal.java:67 uses
  `getBoundingBox().inflate(lookDistance, 3.0, lookDistance)` for non-player entities;
  `SearchArea::from_eyes(entity, range, 3.0)` in look_at_entity.rs:193 matches. Keep 3.0.
- Row 3 (look_forward) - CLOSED. LookAtPlayerGoal.java:97 drives onlyHorizontal exactly as tick
  does (own eye Y vs target eye Y). Shulker.java:101 is the sole `onlyHorizontal=true` caller and
  shulker.rs:94-100 already passes 8.0/0.02/true. pillager.rs:81-97, wandering_trader.rs:251-257,
  interact.rs:20 and panda.rs:281 all pass false, matching every other vanilla caller.
- Row 4 (rider filter) - CONFIRMED VANILLA. LookAtPlayerGoal.java:42-44 applies
  `EntitySelector.notRiding(mob)` for Player.class; `EntityPredicate::Rides` returns false for a
  rider, so the predicate keeps non-riders and rejects riders. Keep the filter.
- Row 5 (slot target range) - CONFIRMED VANILLA. Panda.java:940 only searches when lookAt is null
  and BREED_TARGETING has range 8.0 (:82) while the goal is 6.0 (:276); canUse does not recheck a
  preset target (:925-959) and canContinueToUse (:930-932, super :81) drops it once past 6.0.
  look_at_entity.rs:174-175 / :217-219 match. Do not clamp.
- Row 6 (Controls::LOOK) - CONFIRMED VANILLA. LookAtPlayerGoal.java:41 calls
  `setFlags(EnumSet.of(Goal.Flag.LOOK))`; goal_control: Controls::LOOK matches.
- Visibility / distance scaling - CONFIRMED VANILLA. TargetingConditions.java:14-15, 88-98 keeps
  line of sight and scales range by visibility with a 2.0 floor; create_target_predicate uses
  create_non_attackable() (both checks kept) and the test
  `an_invisible_target_is_only_found_within_two_blocks` asserts the floor. The earlier
  ignore_visibility() variant was correctly reverted.


## Result (final, post-restart continuation)

The three passes above already resolved every STILL OPEN row; this continuation re-read the
final source rather than repeating the decompiled verification. All in-scope files already
carry the settled state, so no source file was edited and no new test was added.

- look_at_entity.rs: `create_target_predicate` uses `TargetPredicate::create_non_attackable()`
  with `base_max_distance = range` (visibility + 2-block floor kept), `tick_reach` declares
  `players: range` (row 2 fix), `SearchArea::from_eyes(..., 3.0)` kept (row 1 confirmed vanilla),
  `EntityPredicate::Rides` rider filter kept (row 4 confirmed vanilla), slot target set without
  range clamp and dropped past `range` in `should_continue` (row 5 confirmed vanilla),
  `goal_control: Controls::LOOK` (row 6 confirmed vanilla), and `look_forward` drives own-eye-Y
  in `tick` (row 3). Tests present: slot target, invisible 2-block floor, look_forward.
- shulker.rs:94-100 passes `8.0 / 0.02 / true` (sole vanilla onlyHorizontal caller) - row 3 closed.
- wandering_trader.rs:884-897 `LookAtTradingPlayerGoal::can_start` only requires a trading
  player, with the 8-block test in `should_continue` (prior-pass fix) - confirmed present.
- interact.rs / panda.rs / pillager.rs callers all pass `false`, matching vanilla.

No rows remain open. `fixed=0 open=0` for this continuation pass (prior passes hold the edits).

## Result (post-restart verification continuation)

Re-read the final on-disk source for every in-scope file after the restart; the settled
state documented by the four prior passes is present and no source file needed editing.
No .rs file was touched, so no rustfmt and no new test were required (no behaviour changed).
Verified against the current files:

- look_at_entity.rs: create_target_predicate uses TargetPredicate::create_non_attackable() with
  base_max_distance = f64::from(range) (visibility and 2-block floor kept); tick_reach declares
  players: range (row 2 fix); SearchArea::from_eyes(..., 3.0) kept (row 1, vanilla
  inflate(lookDistance, 3.0, lookDistance)); EntityPredicate::Rides rider filter kept (row 4,
  vanilla notRiding); slot target set without clamp and dropped past range in should_continue
  (row 5, vanilla PandaLookAtPlayerGoal); goal_control: Controls::LOOK (row 6, vanilla
  setFlags(LOOK)); look_forward drives own-eye-Y in tick (row 3). Three tests present.
- shulker.rs:94-100 passes 8.0 / 0.02 / true (sole vanilla onlyHorizontal=true caller), row 3 closed.
- wandering_trader.rs LookAtTradingPlayerGoal::can_start only requires a trading player;
  should_continue keeps the 8-block test (prior-pass fix), confirmed present.
- interact.rs / panda.rs / pillager.rs callers pass false, matching vanilla.

No rows remain open. fixed=0 open=0 for this continuation pass; prior passes hold the edits.


## Result (decompiled-vanilla re-verification pass)

Re-verified every STILL OPEN row directly against the vendored decompiled vanilla
(.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/goal/LookAtPlayerGoal.java,
ai/targeting/TargetingConditions.java, animal/panda/Panda.java, monster/Shulker.java).
All rows are already correct in Pumpkin; no source file needed editing.

- Row 1 (box 3.0 vertical) - CONFIRMED VANILLA. LookAtPlayerGoal.java:67 uses
  getBoundingBox().inflate(lookDistance, 3.0, lookDistance). look_at_entity.rs:193
  SearchArea::from_eyes(..., 3.0) matches. Keep 3.0.
- Row 2 (player reach) - CONFIRMED. tick_reach declares players: range (f64), matching
  TargetingConditions.range(lookDistance); the old 10x MAX_VISIBILITY bound is gone.
- Row 3 (look_forward) - CONFIRMED. LookAtPlayerGoal.java:97 uses own eye Y when
  onlyHorizontal; tick drives it identically. Shulker.java Shulker passes onlyHorizontal=true
  and shulker.rs:94-100 passes 8.0/0.02/true; all in-scope callers pass false.
- Row 4 (rider filter) - CONFIRMED VANILLA. LookAtPlayerGoal.java:43 applies
  EntitySelector.notRiding(mob); EntityPredicate::Rides keeps non-riders. Keep the filter.
- Row 5 (slot target range) - CONFIRMED VANILLA. Panda.java:82 BREED_TARGETING range 8.0,
  :276 look goal 6.0, :940 canUse only searches when lookAt is null and does not recheck a
  preset target, :930-932 drops it past 6.0 via super. look_at_entity.rs sets the slot target
  without a clamp and should_continue drops it past range. Do not clamp.
- Row 6 (Controls::LOOK) - CONFIRMED VANILLA. LookAtPlayerGoal.java:41 setFlags(LOOK);
  goal_control: Controls::LOOK matches.
- Visibility / distance scaling - CONFIRMED VANILLA. TargetingConditions.java:26 forNonCombat
  leaves checkLineOfSight/testInvisible true; :88-89 scale range by visibility with a 2.0 floor.
  create_target_predicate uses create_non_attackable() (respects_visibility +
  use_distance_scaling_factor both true, MIN_DISTANCE=2.0); test
  an_invisible_target_is_only_found_within_two_blocks locks the floor.
- Wandering trader - CONFIRMED. LookAtTradingPlayerGoal::can_start only requires a trading
  player; should_continue keeps the 8-block test.

No .rs file changed; rustfmt not needed and no new test added (behaviour already locked by
the three existing look_at_entity tests). fixed=0 open=0.

## Result (pass: vanilla re-verification, all rows closed)

Re-read the current on-disk source for every in-scope file and re-checked each open row
against the vendored decompiled vanilla. No source file needed editing; no rustfmt or new
test was required (the three look_at_entity tests already lock the behaviour).

- Row 1 (3.0 vertical box inflation) - VERIFIED. LookAtPlayerGoal.java:67 inflates
  (lookDistance, 3.0, lookDistance); look_at_entity.rs:193 SearchArea::from_eyes(..., 3.0) matches.
- Row 2 (player reach) - VERIFIED (fixed in an earlier pass). tick_reach declares players: range;
  the 10x MAX_VISIBILITY bound is gone.
- Row 3 (look_forward) - VERIFIED. LookAtPlayerGoal.java:97 uses own eye Y when onlyHorizontal;
  Shulker.java:101 is the sole true caller and shulker.rs:94-100 passes 8.0/0.02/true.
  interact.rs:20, pillager.rs:86/96, panda.rs:281/1209 and wandering_trader.rs pass false.
- Row 4 (rider filter) - VERIFIED. LookAtPlayerGoal.java:42-44 notRiding;
  EntityPredicate::Rides (predicate/mod.rs:47-63) returns false for a rider. Keep the filter.
- Row 5 (slot target range) - VERIFIED. Panda.java:82/276/925-959/930-932;
  look_at_entity.rs sets the slot target without a clamp and should_continue drops it past
  range. Do not clamp.
- Row 6 (Controls::LOOK) - VERIFIED. LookAtPlayerGoal.java:41 setFlags(LOOK);
  goal_control: Controls::LOOK matches.
- Visibility / distance scaling - VERIFIED. TargetingConditions.java forNonCombat keeps line of
  sight and scales range with a 2.0 floor; create_target_predicate uses create_non_attackable()
  and the test an_invisible_target_is_only_found_within_two_blocks locks the floor.
- Wandering trader - VERIFIED. LookAtTradingPlayerGoal.java:15-22 canUse only checks isTrading;
  can_start checks only a trading player and should_continue keeps the 8-block test
  (inherited canContinueToUse, LookAtPlayerGoal.java:80-82).

All five STILL OPEN markers closed. fixed=0 open=0.


## Result (third pass, fresh decompiled re-verification)

Re-read every in-scope source file and the vendored decompiled vanilla from disk this
pass; all six rows are already correct, so no .rs file was edited (no rustfmt and no new
test were needed: the three look_at_entity tests already lock the behaviour).

- Row 1 (3.0 vertical box) - VERIFIED. `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/goal/LookAtPlayerGoal.java:67` inflates (lookDistance, 3.0, lookDistance); `SearchArea::from_eyes(..., 3.0)` expands by (range, 3.0, range) via `expand(horizontal, vertical, horizontal)` in target_match.rs:159. Keep 3.0.
- Row 2 (player reach) - FIXED (earlier pass). tick_reach declares `players: range`; the 10x MAX_VISIBILITY bound is gone.
- Row 3 (look_forward) - VERIFIED. LookAtPlayerGoal.java:97 uses own eye Y when onlyHorizontal and tick drives it identically. Shulker.java:101 is the sole `true` caller and shulker.rs:94-100 passes 8.0/0.02/true; interact.rs:20, panda.rs:281/1209, pillager.rs:81/91 and wandering_trader.rs:251 all pass false.
- Row 4 (rider filter) - VERIFIED. LookAtPlayerGoal.java:43 applies `EntitySelector.notRiding(mob)`; `EntityPredicate::Rides` keeps non-riders and rejects riders. Keep the filter.
- Row 5 (slot target range) - VERIFIED. Vanilla `PandaLookAtPlayerGoal` keeps a preset target and only range-drops it in canContinueToUse (LookAtPlayerGoal.java:80-82); look_at_entity.rs sets the slot target without a clamp and should_continue drops it past range. Do not clamp.
- Row 6 (Controls::LOOK) - VERIFIED. LookAtPlayerGoal.java:41 setFlags(LOOK); goal_control: Controls::LOOK matches.
- Visibility/distance scaling - VERIFIED. TargetingConditions.forNonCombat() keeps LOS + visibility scaling with a 2.0 floor; create_target_predicate uses create_non_attackable() (target_predicate.rs:142-158) and the test an_invisible_target_is_only_found_within_two_blocks locks the floor.
- Wandering trader - VERIFIED. `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/goal/LookAtTradingPlayerGoal.java:15-22` canUse only checks isTrading; wandering_trader.rs:884-888 can_start checks only a trading player and :890-897 should_continue keeps the 8-block test.

No rows remain STILL OPEN. fixed=0 open=0 for this pass.
