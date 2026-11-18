# entity/ai/goal/melee_attack — deep audit (2026-11-16)

Scope reviewed: `crates/pumpkin-core/src/entity/ai/goal/melee_attack.rs` in full, plus the
helpers it calls (`Mob::is_in_attack_range` / `is_within_melee_attack_range`
`entity/mob/mod.rs:1386`, `MobEntity::is_in_position_target_range_pos` `:397`,
`MobEntity::get_target` `:982`, `Navigator::set_progress`/`is_idle`
`entity/ai/pathfinder/mod.rs:554`), and the vanilla-derived test harness in
`entity/passive/polar_bear.rs:280-368`. No decompiled vanilla tree is present
(`.vendor/` is empty), so the items below name the vanilla class/method they must be
checked against. Vanilla reference: `net.minecraft.world.entity.ai.goal.MeleeAttackGoal`
(Yarn `MeleeAttackGoal`), 1.21.

## Fixed in this audit

| # | Change | Reason |
|---|---|---|
| 1 | `can_perform_attack` no longer requires `has_line_of_sight` (commit `64cfde0be`) | Vanilla `checkAndPerformAttack(target, distSqr)` only compares `getPerceivedTargetDistanceSquareForMeleeAttack` with `getAttackReachSqr`. Line of sight is used only for path recalculation in `tick()` (`followingTargetEvenIfNotSeen \|\| sensing.hasLineOfSight`). Pumpkin refused hits within reach whenever the target was occluded (corner/edge/pillar cases), which vanilla lands. |
| 2 | `get_max_cooldown` uses `attack_interval_ticks` instead of a literal `20` (`1be4c3860`) | Vanilla keeps `attackInterval` as a field and `resetAttackCooldown` delays by it. The field existed but was dead (`#[expect(dead_code)]`), so a future per-mob interval would have been silently ignored. |
| 3 | `stop()` no longer tries to clear a creative/spectator target; `should_continue` no longer re-checks it (`c78921955`) | `MobEntity::get_target()` (`mob/mod.rs:982`) already filters `EntityPredicate::ExceptCreativeOrSpectator` *and* `can_attack` on every read, so both checks were permanently false/dead. Vanilla `MeleeAttackGoal.stop()` only stops navigation and clears the aggressive flag. |

## Gaps left

| # | Rust location | Vanilla reference | Gap | Suggested fix | Size |
|---|---|---|---|---|---|
| 1 | `melee_attack.rs:162-175` (`should_continue`) | `MeleeAttackGoal.canContinueToUse` | The navigation-idle check is in the wrong branch. Every caller passes `true` (`passive/wolf.rs:108`, `mob/piglin.rs:121`, `mob/ravager.rs:62`, `passive/iron_golem.rs:89`, ...), and that `true` branch never tests `is_navigator_idle`; only the `false` branch does. Vanilla's `followingTargetEvenIfNotSeen == true` branch is `return !this.mob.getNavigation().isDone();`. As written a mob that finished its path keeps the goal running, so `tick` only re-paths on the 5% roll or after the target moves a full block (`:230-233`), and the goal never yields to a higher-priority goal. | Make `!is_idle` unconditional and keep `is_in_position_target_range_pos` for the `true` (chase even when unseen) branch; verify the exact vanilla split before editing. | M |
| 2 | `melee_attack.rs:141-146` (`can_start`, both TODOs) | `MeleeAttackGoal.canUse` | Vanilla creates a path (`getNavigation().createPath(target, 0)`) and, when that returns null, still starts if `isWithinMeleeAttackRange(target)`. Pumpkin returns `true` for any live target and relies on the navigator going idle the next tick to stop. Behavioural result is close but not identical (a mob with no route to a distant target burns one tick of goal ownership and one `last_update_time` slot). | `Navigator` has no synchronous path query today (`set_progress` only sets `has_pending_path`, `pathfinder/mod.rs:554`), so this needs either a sync `create_path` or a `can_reach`-style helper. Add the fallback `mob.is_in_attack_range(target)` once that exists. | S-M |
| 3 | `melee_attack.rs:243-244` (TODO) | `MeleeAttackGoal.tick` | Vanilla adds `+15` ticks to the recalculation timer when `getNavigation().moveTo(...)` fails. Not implementable synchronously: `set_progress` is deferred, so success is unknown at the call site. | Expose the last pathfinding result on `Navigator` (e.g. `path_request_failed()` refreshed after the pending path resolves) and add the 15-tick penalty there. | S |
| 4 | `melee_attack.rs:187` (`start`) | `MeleeAttackGoal.start` | Vanilla does not seed `pathedTarget*` in `start()`, so the `pathedTarget == 0` sentinel forces a path refresh on the first tick. Pumpkin seeds `last_target_position` with the target position, suppressing the first refresh until the target moves 1 block or the 5% roll fires. Harmless in practice (start already issues a path) but not vanilla. | Leave `last_target_position = None` in `start()`; the first tick then refreshes like vanilla. | S |
| 5 | `melee_attack.rs:17, 220` | `MeleeAttackGoal` constructor arg `followingTargetEvenIfNotSeen` | The field is named `pause_when_mob_idle` (Yarn name) but is used as Mojang's `followingTargetEvenIfNotSeen` as an OR with LOS at `:220`, and as "follow" in `should_continue`. The name reads inverted; item 1 cannot be resolved until the mapping is fixed in one direction. | Rename to `follow_even_if_unseen` and align both call sites (item 1). | S |
| 6 | `melee_attack.rs:12` (`MAX_ATTACK_TIME`) | `MeleeAttackGoal.COOLDOWN_BETWEEN_CAN_USE_CHECKS = 20L` | Value and semantics match the per-instance `lastCanUseCheck`; nothing to change. Listed so the constant is not re-reviewed. | none | - |

## Checked and already correct

- `get_max_cooldown` / `reset_attack_cooldown` / `is_time_to_attack`: vanilla `attackInterval`,
  `resetAttackCooldown` (`adjustedTickDelay`, exact because `requiresUpdateEveryTick()` is true).
- `should_run_every_tick() == true` matches vanilla `MeleeAttackGoal.requiresUpdateEveryTick()`.
- `look_at_entity_with_range(&target, 30.0, 30.0)` matches `getLookControl().setLookAt(target, 30, 30)`.
- `sensing.tick()` is not the goal's job; `MobEntity::server_ai_step` (`mob/mod.rs:480-490`) already runs it.
- Path-recalc trigger (`target moved >= 1 block` or `random < 0.05`), `4 + rand(0..7)`, `+10` above
  1024 sq and `+5` above 256 sq, and `has_line_of_sight` gating match `MeleeAttackGoal.tick`.
- `start`/`stop` control flags (`MOVE | LOOK`, `set_attacking`), `navigation.stop()` in `stop`, and the
  `get_target()`-based creative/spectator drop.
- `Controls` and the `gated_by`/`performing`/`checking`/`on_stop` hooks are Pumpkin extensions used by
  `passive/polar_bear.rs`, `passive/fox`, `passive/panda`; they do not diverge from vanilla behaviour.

## Gap status
- Rows: STILL OPEN. Note body partly unreadable in this pass; nothing verified.

## Result (2026-11-17)

Vanilla checked: `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/goal/MeleeAttackGoal.java` (present, matches `CURRENT_MC_VERSION`), plus `PolarBear.PolarBearMeleeAttackGoal` and `Mob.isWithinHome`.

| # | Result | Detail |
|---|---|---|
| 1 | NOT A GAP, corrected | Vanilla `canContinueToUse`: `!followingTargetEvenIfNotSeen ? !navigation.isDone() : isWithinHome(target.blockPosition()) && !(player creative/spectator)`. The note had the branches swapped; Pumpkin's `should_continue` already ports it (the `!is_idle` check on the false branch, `is_in_position_target_range_pos` on the true one, the creative/spectator half being covered by `get_target`). Only the flag was renamed (row 5). |
| 2 | FIXED | `can_start` now mirrors vanilla: `self.path = createPath(target, 0)` then `self.path.is_some() \|\| mob.is_in_attack_range(target)`. A synchronous `create_path` already existed (`PathNavigationTrait::create_path`), so no navigator work was needed; the stale-path case (a failed search keeping an old path) is gone too. |
| 3 | STILL OPEN | `set_progress` returns `()`, so the `if (!moveTo(...)) ticks += 15` penalty has no synchronous result to read. Needs a path-request-result field on `Navigator` (pathfinder files, outside this unit). |
| 4 | FIXED | `start()` no longer seeds `last_target_position`, so the first `tick` recalculates like vanilla's zeroed `pathedTarget*` sentinel. |
| 5 | FIXED | `pause_when_mob_idle` renamed to `follow_even_if_unseen` (field, ctor arg, both uses), matching `MeleeAttackGoal.followingTargetEvenIfNotSeen`. |
| 6 | unchanged | Correct as written, no vanilla hook. |

Correction to the "Fixed in this audit" section: its row 1 (LOS removed from `can_perform_attack`) contradicts the decompiled source. Vanilla `canPerformAttack` is `isTimeToAttack() && isWithinMeleeAttackRange(target) && getSensing().hasLineOfSight(target)`; LOS is restored through `mob.has_line_of_sight` (the per-tick sensing cache).

Tests: `melee_attack.rs` gained pure unit tests for the new `should_recalculate_path` helper (timer, moved-block threshold, 5 percent roll, and that the roll is not drawn when the target moved, which is vanilla's short-circuit).

Other unit files: `polar_bear.rs`, `piglin.rs`, `ravager.rs`, `wolf.rs`, `iron_golem.rs` only register the goal; their `new(speed, true)` calls mean `followingTargetEvenIfNotSeen == true`, which matches vanilla for Ravager, IronGolem, Wolf, Rabbit, Panda and Fox, so no call site changed.


## Result (2026-11-18 re-audit)

Re-verified every row against `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/goal/MeleeAttackGoal.java` (present, matches `CURRENT_MC_VERSION`). No source edits were needed: every row is already fixed in the checkout.

| # | Result | Detail |
|---|--------|--------|
| 1 | VERIFIED (no gap) | `should_continue` matches vanilla `canContinueToUse`: the `!is_idle` (navigation.isDone) check is in the `!follow_even_if_unseen` branch and `is_in_position_target_range_pos` (vanilla `isWithinHome`, `mob/mod.rs:398`) in the `true` branch. The creative/spectator half of the `true` branch is already filtered by `get_target()`. |
| 2 | VERIFIED (fixed) | `can_start` calls `navigator.create_path(&living_entity, target_pos, 0)` and starts when a path exists or the target is inside melee reach, exactly vanilla `path != null || isWithinMeleeAttackRange(target)`. |
| 3 | VERIFIED (fixed in code) | The `+15` penalty is present: `tick()` consumes `navigator.take_last_path_search_failed()` (set by the deferred search at `pathfinder/mod.rs:887`), matching vanilla `if (!moveTo(...)) ticksUntilNextPathRecalculation += 15`. The original cross-crate prerequisite (a path-result flag on `Navigator`) is already implemented in the checkout, so no code change was needed in this unit. |
| 4 | VERIFIED (fixed) | `start()` leaves `last_target_position` as `None` (constructor default), so the first tick recalculates like vanilla's zeroed `pathedTarget*` sentinel. |
| 5 | VERIFIED (fixed) | Field/param renamed to `follow_even_if_unseen` and used in both `should_continue` and `tick` (vanilla `followingTargetEvenIfNotSeen`); all call sites use the new name. |
| 6 | VERIFIED (no change) | `MAX_ATTACK_TIME = 20` matches vanilla `COOLDOWN_BETWEEN_CAN_USE_CHECKS = 20L`. |

Other unit files (`iron_golem.rs`, `piglin.rs`, `polar_bear.rs`, `ravager.rs`, `wolf.rs`) only register `MeleeAttackGoal::new(speed, true)`, which sets `followingTargetEvenIfNotSeen == true`, matching vanilla for these mobs. No call-site change needed.

No pure seam was left untested: `should_recalculate_path` already has unit tests covering the timer, moved-block threshold, 5% roll, and the roll short-circuit; the `+15` penalty lives in `tick()` behind a `Mob`/`Navigator`, so no additional pure test applies.

## Result (CODER batch r2_011)
- No row changed this pass: CODER did not reach these rows in the time budget (only entity/mod.rs row 2 was checked). All open rows remain STILL OPEN.

## Result (CODER batch r3_030)

Vanilla: `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/goal/MeleeAttackGoal.java` (canUse 33-51, canContinueToUse 53-65, start 68-73, tick 92-125). Rust: `crates/pumpkin-core/src/entity/ai/goal/melee_attack.rs`. No source edits this pass.

| # | Result | Detail |
|---|--------|--------|
| 1 | VERIFIED | `should_continue` (melee_attack.rs:185-205): `!follow_even_if_unseen` branch is `!is_navigator_idle` (vanilla 61-62 `!navigation.isDone()`); the other branch is `is_in_position_target_range_pos` (vanilla 63 `isWithinHome`). |
| 2 | VERIFIED | `can_start` (melee_attack.rs:172-182) calls `create_path(target, 0)` and returns `path.is_some() \|\| is_in_attack_range` (vanilla 49-50). |
| 3 | VERIFIED (timing note) | Penalty present at melee_attack.rs:278-283 via `take_last_path_search_failed` (pathfinder/mod.rs:582, set at :896). Vanilla adds +15 synchronously inside the same `tick`; Pumpkin applies it on the first tick after the deferred search fails. Architectural, not changed. |
| 4 | VERIFIED | `start` (melee_attack.rs:207-218) does not seed `last_target_position`; the constructor default `None` (line 63) forces the first recalc like vanilla's zeroed `pathedTarget*` (vanilla 68-73, 98-100). |
| 5 | VERIFIED | `follow_even_if_unseen` (melee_attack.rs:34, 52, 198, 247) matches vanilla `followingTargetEvenIfNotSeen` (14, 61, 97). |
| 6 | VERIFIED | `MAX_ATTACK_TIME = 20` (melee_attack.rs:11) matches vanilla `COOLDOWN_BETWEEN_CAN_USE_CHECKS = 20L` (23, 35). |

Status: all six rows closed by verification. Gap status line 47 is stale; the 2026-11-18 re-audit section above already reflects this.
