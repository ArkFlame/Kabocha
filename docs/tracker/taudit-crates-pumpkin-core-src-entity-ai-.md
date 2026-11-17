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
