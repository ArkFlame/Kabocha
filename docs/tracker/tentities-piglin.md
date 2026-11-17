# entities / Piglin (RE-REVIEW)

Reviewed: `crates/pumpkin-core/src/entity/mob/piglin.rs`, `piglin_ai.rs`, `piglin_brute.rs`, `zombified_piglin.rs`.

Fixed: `celebration_timer` was never set and `set_dancing(true)` never called (`wants_to_dance`, `CELEBRATION_TIME` dead), and `hunt_cooldown_timer` was never set so `can_hunt()` was always true. Both now set from `killed_entity`. Commit: fix(entity): piglin celebration and hunt cooldown on kill.

## Remaining gaps

| file:line | vanilla | suggested fix | size |
|---|---|---|---|
| piglin_ai.rs:32 (`BABY_FLEE_DURATION` unused), piglin.rs:115 | `BabyPiglinTasks` / `Piglin` baby goals: babies flee 100 ticks from an attacker and play instead of fighting | give babies their own goal set (no `MeleeAttackGoal`/hunt, add flee goal wired to `BABY_FLEE_DURATION`) | M |
| piglin_ai.rs:36 (`DESIRED_DISTANCE_FROM_ZOMBIFIED` unused) | `PiglinAi` `avoidZombified`: adult piglins keep 6 blocks from zombified piglin and zoglin | add an avoidance goal/target-goal exclusion for `ZOMBIFIED_PIGLIN`/`ZOGLIN` at 6 blocks for adults | S |
| piglin_ai.rs:28 (`ANGER_DURATION` unused) | `PiglinAi.angerNearbyPiglins`: hurting one piglin (or its gold) angers nearby piglins for 600 ticks, targeting the attacker | add an anger timestamp + "angered by" field to `PiglinEntity`, set it in `was_hurt_by`, and target players via that flag instead of only `RevengeGoal` | M |
| piglin.rs:1189 (`killed_entity`, added) | `PiglinTasks.setHuntedRecently` also runs when a hunt ends without a kill (target lost) | set the 600..=2400 `hunt_cooldown_timer` when the current target is a hoglin and it goes invalid/dead, not only on kill | S |
| piglin.rs:805 (`check_piglin_spawn_rules`) | `Piglin.checkPiglinSpawnRules` also honours the spawn type; verify Pumpkin's caller does not apply the nether-wart rule to non-spawn paths | verify the call site in `world/natural_spawner.rs` and pass the spawn type through if it differs | S (verify) |
