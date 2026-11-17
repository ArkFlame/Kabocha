# Tracker audit: entities / Vex

Source: `crates/pumpkin-core/src/entity/mob/vex.rs`. Vanilla reference: `net.minecraft.world.entity.monster.Vex` (from memory, no `.vendor/` decompile present in this worktree; verify before porting).

| # | Location | Vanilla | Gap | Suggested fix | Size |
|---|----------|---------|-----|---------------|------|
| 1 | vex.rs:49 | `VexChargeAttackGoal` at prio 4 | Pumpkin uses generic `MeleeAttackGoal`, no charge state or charge cooldown | Port charge goal (bound-origin charge, `isCharging` flag) | M |
| 2 | vex.rs:50 | `VexRandomMoveGoal` at prio 8 | Pumpkin uses `WanderAroundGoal` at prio 5 (ground wander, wrong priority and selection logic) | Port `VexRandomMoveGoal` (random flight target within bound origin) | S |
| 3 | vex.rs:48 | No `SwimGoal` in vanilla Vex registerGoals | Extra goal at prio 0 | Remove if verified against decompile | S |
| 4 | vex.rs:55 | `LookAtPlayerGoal` (player, 3.0) prio 9 and `LookAtPlayerGoal` (Mob, 8.0) prio 10 | Pumpkin has one `LookAtEntityGoal` prio 6 (8.0) and `RandomLookAroundGoal` prio 7 | Replace with the two vanilla look goals and priorities | S |
| 5 | vex.rs:59 | `VexCopyOwnerTargetGoal` at target prio 2 | Missing; the TODO is the only trace | Port copy-owner-target goal (owner from summoner UUID) | M |
| 6 | vex.rs:58 | `HurtByTargetGoal(Raider).setAlertOthers()` at target prio 1 | Uses `raider::revenge_goal()`; alert-others behaviour not confirmed | Confirm `revenge_goal` sets alert-others, else add | S |
| 7 | vex.rs (absent) | `Vex.setLimitedLife` / `tick` lifetime decrement, removal when expired (evoker-summoned vexes live ~30-60 s) | No summoned lifetime management | Add limited-life field to `MobEntity` or `VexEntity`, decrement in tick, discard at zero; set from evoker summon | M |
| 8 | vex.rs:43 | `Vex.tick` calls `setNoGravity(true)` each tick | Set once at construction only | Keep construction-time set; only needs re-check if gravity toggling can occur | S (likely fine) |

Notes:
- Gaps 1, 2, 4 and 5 are behaviour gaps, not compile gaps. None was fixed here because each needs the vanilla goal bodies to be ported, and this pass made no code change.
- No small, certain code fix was found in this pass, so no commit besides this file.
- Not verified: exact goal priorities and the `setAlertOthers` flag. Re-check against decompiled `Vex.java` before implementing.
