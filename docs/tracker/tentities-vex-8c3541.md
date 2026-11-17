# Tracker audit: entities / Vex

Source: `crates/pumpkin-core/src/entity/mob/vex.rs`. Vanilla reference: `net.minecraft.world.entity.monster.Vex` (from memory, no `.vendor/` decompile present in this worktree; verify before porting).

| # | Location | Vanilla | Gap | Suggested fix | Size |
|---|----------|---------|-----|---------------|------|
| 1 | vex.rs:49 | `VexChargeAttackGoal` at prio 4 | Pumpkin uses generic `MeleeAttackGoal`, no charge state or charge cooldown | Port charge goal (bound-origin charge, `isCharging` flag) | M. FIXED feat(entity): port vex charge, random move, look goals and lifespan |
| 2 | vex.rs:50 | `VexRandomMoveGoal` at prio 8 | Pumpkin uses `WanderAroundGoal` at prio 5 (ground wander, wrong priority and selection logic) | Port `VexRandomMoveGoal` (random flight target within bound origin) | S. FIXED feat(entity): port vex charge, random move, look goals and lifespan |
| 3 | vex.rs:48 | No `SwimGoal` in vanilla Vex registerGoals | Extra goal at prio 0 | Remove if verified against decompile | S. STILL OPEN: no decompiled `Vex.java` in this worktree and `docs/VANILLA_REF.md` does not list goals, so the swim goal is kept unverified |
| 4 | vex.rs:55 | `LookAtPlayerGoal` (player, 3.0) prio 9 and `LookAtPlayerGoal` (Mob, 8.0) prio 10 | Pumpkin has one `LookAtEntityGoal` prio 6 (8.0) and `RandomLookAroundGoal` prio 7 | Replace with the two vanilla look goals and priorities | S. FIXED feat(entity): port vex charge, random move, look goals and lifespan |
| 5 | vex.rs:59 | `VexCopyOwnerTargetGoal` at target prio 2 | Missing; the TODO is the only trace | Port copy-owner-target goal (owner from summoner UUID) | M. STILL OPEN: evoker summon (`evoker.rs` `from_type(&EntityType::VEX, ..)`) stores no owner, and reading the owner's Mob target from `EntityBase` needs a downcast that does not exist yet |
| 6 | vex.rs:58 | `HurtByTargetGoal(Raider).setAlertOthers()` at target prio 1 | Uses `raider::revenge_goal()`; alert-others behaviour not confirmed | Confirm `revenge_goal` sets alert-others, else add | S. FIXED (verified, no code change): `raider::revenge_goal()` already calls `alerting_others()` |
| 7 | vex.rs (absent) | `Vex.setLimitedLife` / `tick` lifetime decrement, removal when expired (evoker-summoned vexes live ~30-60 s) | No summoned lifetime management | Add limited-life field to `MobEntity` or `VexEntity`, decrement in tick, discard at zero; set from evoker summon | M. PARTIAL, STILL OPEN: `VexEntity::set_limited_life`, the per-tick decrement and starve (1 damage every 20 ticks at zero, as vanilla) are in place (feat(entity): port vex charge, random move, look goals and lifespan). Still open: evoker summon never calls `set_limited_life`, so no vex has a limit yet. Vanilla does not remove the vex at zero, it starves it |
| 8 | vex.rs:43 | `Vex.tick` calls `setNoGravity(true)` each tick | Set once at construction only | Keep construction-time set; only needs re-check if gravity toggling can occur | S (likely fine) |

Notes:
- Pass 2 (feat(entity): port vex charge, random move, look goals and lifespan): gaps 1, 2, 4, 6 fixed, 7 partially. Gaps 3 and 5 remain open (see rows). Gap 8 untouched.
- Charge and random move keep their state in the goal, not in an entity flag. Their `tick_reach` is the default `None`, which is correct because `VexEntity` does not set `has_audited_tick`.
- Not verified: exact goal priorities and the `setAlertOthers` flag. Re-check against decompiled `Vex.java` before implementing.
