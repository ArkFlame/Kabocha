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

### 2. Player reach is declared 10x too large (S)
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
