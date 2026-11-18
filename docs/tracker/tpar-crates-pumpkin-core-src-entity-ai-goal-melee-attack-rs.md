# Parity rows for crates/pumpkin-core/src/entity/ai/goal/melee_attack.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-en-f70612** [partial/OWN-OPEN] own audit: 4 open, 1 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-en-f70612.md) docs/tracker/taudit-crates-pumpkin-core-src-en-f70612.md
- **own/taudit-crates-pumpkin-core-src-entity-ai-** [partial/OWN-OPEN] own audit: 2 open, 3 fixed, 6 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-entity-ai-.md) docs/tracker/taudit-crates-pumpkin-core-src-entity-ai-.md

## Result (2026-11-18 third pass)

All rows verified against `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/goal/MeleeAttackGoal.java` (present, matches `CURRENT_MC_VERSION`). Every row is already fixed in the checkout; no source edit was needed this pass. No BLOCKED-DATA (no generated field or missing subsystem involved).

own/taudit-crates-pumpkin-core-src-en-f70612 (5 rows):

| row | status | evidence |
|---|---|---|
| 1 +15 retry backoff on failed moveTo (M) | VERIFIED (fixed) | `tick` consumes `navigator.take_last_path_search_failed()` and adds 15 (melee_attack.rs:276-284); flag set by the deferred search at `pathfinder/mod.rs:896`, cleared in `set_progress`, exposed via `take_last_path_search_failed`, and `navigator()` returns a `MutexGuard` so the `&mut self` call type-checks. Matches vanilla `if (!moveTo(...)) ticks += 15` (MeleeAttackGoal.java:115-117). |
| 2 stop() dead ForgotTarget block (S) | VERIFIED (no gap) | current `stop()` (melee_attack.rs:220-229) has no target-clear, only `set_attacking(false)` + `navigator().stop()`; `MobEntity::get_target` filters creative/spectator on every read, so the vanilla `stop()` clear (MeleeAttackGoal.java:77-80) is unreachable. |
| 3 perceived box-to-hitbox distance (S) | VERIFIED (no gap) | vanilla uses `this.mob.distanceToSqr(target)` centre-to-centre (MeleeAttackGoal.java:108), exactly matching `mob_pos.squared_distance_to_vec(&current_target_pos)` (melee_attack.rs:258). `getPerceivedTargetDistanceSquareForMeleeAttack` does not exist in the vendor tree; the original gap note was erroneous. |
| 4 constructor arg naming / inverted meaning (S) | VERIFIED (fixed) | field/arg renamed to `follow_even_if_unseen` (melee_attack.rs:34,52), used in `should_continue` (198) and `tick` (247), matching vanilla `followingTargetEvenIfNotSeen` (MeleeAttackGoal.java:14,61,97). `creaking.rs:176` passes `false`. |
| 5 attack_interval_ticks unused (S) | VERIFIED (fixed) | `get_max_cooldown` (melee_attack.rs:107-110) returns `get_tick_count(self.attack_interval_ticks)`, used by `reset_attack_cooldown`; default 20 matches vanilla `attackInterval`/`adjustedTickDelay(20)` (MeleeAttackGoal.java:21,136,152). |

own/taudit-crates-pumpkin-core-src-entity-ai- (6 rows):

| row | status | evidence |
|---|---|---|
| 1 should_continue idle branch (M) | VERIFIED (no gap) | `!follow_even_if_unseen` -> `!is_navigator_idle()` (melee_attack.rs:198-201, vanilla navigation.isDone at MeleeAttackGoal.java:61-62); true branch -> `is_in_position_target_range_pos` (melee_attack.rs:203-204, vanilla `isWithinHome` at :63), with the creative/spectator half covered by `get_target`. |
| 2 can_start createPath + reach fallback (S-M) | VERIFIED (fixed) | `navigator.create_path(&living_entity, target_pos, 0)` then `self.path.is_some() || mob.is_in_attack_range(target)` (melee_attack.rs:172-182), matching vanilla `path != null || isWithinMeleeAttackRange(target)` (MeleeAttackGoal.java:49-50). |
| 3 +15 on failed moveTo (S) | VERIFIED (fixed) | same as en-f70612 row 1; flag/getter live in `pathfinder/mod.rs` (outside this unit) and are already present. |
| 4 start() seeds last_target_position (S) | VERIFIED (fixed) | `start()` leaves `last_target_position = None` (constructor default, melee_attack.rs:63,207-218); first tick recalculates like vanilla's zeroed `pathedTarget*` sentinel (MeleeAttackGoal.java:68-73,100). |
| 5 constructor arg rename (S) | VERIFIED (fixed) | `pause_when_mob_idle` -> `follow_even_if_unseen` (melee_attack.rs:34,52). |
| 6 MAX_ATTACK_TIME constant (-) | VERIFIED (no change) | `MAX_ATTACK_TIME = 20` (melee_attack.rs:11) matches `COOLDOWN_BETWEEN_CAN_USE_CHECKS = 20L` (MeleeAttackGoal.java:23). |

Also re-checked the corrected LOS row: `can_perform_attack` requires `mob.has_line_of_sight` (melee_attack.rs:123-127), matching vanilla `getSensing().hasLineOfSight(target)` (MeleeAttackGoal.java:143-145). `get_tick_count` is identity for this goal (`should_run_every_tick() == true`, goal/mod.rs:117), so the deferred `+15` placement at melee_attack.rs:281-283 is behaviourally equivalent to vanilla applying `adjustedTickDelay` after the penalty.

Pure-unit-test seam: already present. `should_recalculate_path` has tests for the timer, moved-block threshold, 5% roll and the roll short-circuit (melee_attack.rs:303-330); `a_blocked_view_holds_the_attack` covers the LOS gate (melee_attack.rs:332-396). No new test needed.

Files edited: none (source already matched vanilla). rustfmt: not applicable, no Rust source touched.
