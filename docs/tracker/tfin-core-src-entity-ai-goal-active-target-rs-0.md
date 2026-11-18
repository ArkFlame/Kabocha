# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-entity-ai-goal-active-target-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

All items below are closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)

## Result

FINAL PASS 2026-11-18. Re-verified the one carried-over row against `.vendor/vanilla-decompiled` (MC 26.3). Source already matches vanilla on every point; no `.rs` change needed. Row `own/taudit-crates-pumpkin-core-src-en-37bcf6` (the sole item in the parent `tpar-...-active-target-rs.md`) closes as VERIFIED (0 open).

- Vertical target search box: VERIFIED. `active_target.rs::find_closest_target` calls `SearchArea::from_eyes(entity, follow_range, follow_range)`; `target_match.rs::from_feet` expands the bounding box by `(horizontal, vertical, horizontal)` so the box grows by the full follow range on every axis. Matches decompiled `net/minecraft/world/entity/ai/goal/target/NearestAttackableTargetGoal.java:58-60` `getTargetSearchArea(d)` = `getBoundingBox().inflate(d, d, d)`. Player search stays box-free (`players_anywhere()`, vanilla `getNearestPlayer` at `:74`).
- Gap 1 (Adult / AgeableMob): VERIFIED. `TargetCondition::allows_search` calls the unified `mob.mob_is_baby()` (`entity/mob/mod.rs:2427`), matching decompiled `Mob.isBaby()`. Sole user is the polar-bear fox hunt (`net/minecraft/world/entity/animal/polarbear/PolarBear.java:98` `(target, level) -> !this.isBaby()`); no parity bug.
- Gap 2 (Untamed continuation): VERIFIED, no change. Decompiled `net/minecraft/world/entity/ai/goal/target/NonTameRandomTargetGoal.java:23-26` `canContinueToUse()` re-tests `targetConditions.test(level, mob, target)` and never calls `super`, so there is no unseen-memory window; Pumpkin's `accepts()` re-test in `should_continue` is the correct mirror.
- Gap 3 (mustReach / can_navigate): VERIFIED, premise refuted. Decompiled `NearestAttackableTargetGoal.findTarget` (`:62-76`) selects via `getTargetConditions()` only and never calls `TargetGoal.canAttack`, so `mustReach` does not filter target choice. `check_can_navigate` is forwarded to `TrackTargetGoal::new`; polar-bear flag stays `true`, matching `PolarBear.java:256` `super(PolarBear.this, Player.class, 20, true, true, null)`.
- Gap 4 (Untamed target reason): VERIFIED. `start()` uses the player/entity split (`ClosestPlayer`/`ClosestEntity`) for every condition. Decompiled `NearestAttackableTargetGoal.start()` (`:78-82`) does exactly this; `NonTameRandomTargetGoal` has no `start` override.
- Gap 5 (NoDaylight comment): VERIFIED, no change. Decompiled `net/minecraft/world/entity/monster/spider/Spider.java:221-230` `SpiderTargetGoal` overrides only `canUse`, so the daylight gate belongs in `can_start` only.
- Tests: no pure seam for `TargetCondition`/`start` (needs a live `Mob`/`World`); pure reach/memory tests already live in `track_target.rs`. No test added.

Status: PASS fixed=0 open=0 files=docs/tracker/tfin-core-src-entity-ai-goal-active-target-rs-0.md
