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
STILL OPEN (1): no decompiled reference; the audit's "horizontal only" claim conflicts with vanilla `LookAtPlayerGoal`, which I recall as `inflate(lookDistance, 3.0, lookDistance)`. Code matches that, so the 3.0 is not changed without `.vendor/` evidence.
STILL OPEN (3): wiring vanilla `onlyHorizontal` callers needs the decompiled callers list, not available.
STILL OPEN (4): PandaLookAtPlayerGoal's behaviour (rider filter) cannot be verified without a decompiled reference.
STILL OPEN (5): sulk-range vanilla behaviour cannot be verified; clamping is a behaviour choice, not certain.
STILL OPEN (6): needs goal-arbitration change for overlapping flags, which is M/L and not certain.


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
