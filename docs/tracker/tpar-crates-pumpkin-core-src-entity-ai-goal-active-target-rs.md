# Parity rows for crates/pumpkin-core/src/entity/ai/goal/active_target.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-en-37bcf6** [partial/OWN-OPEN] own audit: 2 open, 1 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-en-37bcf6.md) docs/tracker/taudit-crates-pumpkin-core-src-en-37bcf6.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)


## Result

THIRD PASS 2026-11-18. Verified every open row against `.vendor/vanilla-decompiled` (MC 26.3). Source already matches vanilla on all points; no `.rs` change needed. Row `own/taudit-crates-pumpkin-core-src-en-37bcf6` closed as VERIFIED (0 open).

- Vertical target search box: VERIFIED. `active_target.rs::find_closest_target` uses `SearchArea::from_eyes(entity, follow_range, follow_range)`. Decompiled `net/minecraft/world/entity/ai/goal/target/NearestAttackableTargetGoal.java:58-60` `getTargetSearchArea(d)` = `getBoundingBox().inflate(d, d, d)` (full follow range on every axis); player search stays box-free at `:74`.
- Gap 1 (Adult / AgeableMob): VERIFIED. `TargetCondition::allows_search` calls the unified `mob.mob_is_baby()` (`crates/pumpkin-core/src/entity/mob/mod.rs:2418`), matching decompiled `Mob.isBaby()`. Sole user is the polar-bear fox hunt (`PolarBear.java:98` -> `!this.isBaby()`).
- Gap 2 (Untamed continuation): VERIFIED, no change. Decompiled `NonTameRandomTargetGoal.java:23-26` `canContinueToUse()` re-tests `targetConditions.test(...)` and never calls `super`, so no unseen-memory window; Pumpkin's `accepts()` re-test in `should_continue` is the correct mirror.
- Gap 3 (mustReach / can_navigate): VERIFIED, premise refuted. Decompiled `NearestAttackableTargetGoal.findTarget` (`:62-76`) selects via `getTargetConditions()` only and never calls `TargetGoal.canAttack`, so `mustReach` does not filter target choice. `check_can_navigate` is forwarded to `TrackTargetGoal::new` and `can_navigate_to_entity`/`can_track` are implemented (`track_target.rs:91,121`). Polar-bear flag stays `true`: `PolarBear.java:256` `super(PolarBear.this, Player.class, 20, true, true, null)`.
- Gap 4 (Untamed target reason): VERIFIED. `start()` uses the player/entity split (`ClosestPlayer`/`ClosestEntity`) for every condition. Decompiled `NearestAttackableTargetGoal.start()` (`:78-82`) does exactly this; `NonTameRandomTargetGoal` has no `start` override.
- Gap 5 (NoDaylight comment): VERIFIED, no change. Decompiled `Spider.SpiderTargetGoal` (`Spider.java:221-229`) overrides only `canUse`, so the daylight gate belongs in `can_start` only.
- Tests: no pure seam for `TargetCondition`/`start` (needs a live `Mob`/`World`); pure reach/memory tests already live in `track_target.rs`. No test added.

Status: PASS fixed=0 open=0 files=docs/tracker/tpar-crates-pumpkin-core-src-entity-ai-goal-active-target-rs.md
