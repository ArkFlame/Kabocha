# entities / Fox (re-review)

Reviewed: `crates/pumpkin-core/src/entity/passive/fox/mod.rs` and `goals.rs` against vanilla
`net.minecraft.world.entity.animal.Fox` and its inner goals. No decompiled source is vendored in this
worktree (`.vendor/` absent), so rows marked "verify" name the vanilla member they need checking against.

Verdict: structure is complete (goal list/priorities, flags, variants, biome spawn, breeding, NBT, sounds,
berries, pounce/stalk/shelter/sleep/perch/search, defend-trusted, alertable scan). One certain gap fixed;
four smaller divergences remain open.

## Fixed in this audit
- `mod.rs:380` (`tick_states`): vanilla `Fox.tick` clears sleep on `isInWater() || isInPowderSnow ||
  getTarget() != null || level.isThundering()`; the Rust path only checked water, target and thunder.
  Powder snow now wakes the fox too (commit 61bf3620d).

## Remaining gaps

| file:line | vanilla | gap | fix | size |
|---|---|---|---|---|
| `goals.rs:687` (`SleepGoal::can_sleep`) | `Fox.FoxSleepGoal.canUse` | Sleep onset is decided from the countdown, shelter, daylight and `is_alertable` only. Vanilla additionally rolls the retry gate `random.nextInt(adjustedTickDelay(10)) != 0` before trying and rejects `isInWaterOrRain`; the Rust goal is more deterministic and can start sleeping in rain. | Add the random retry and the water/rain rejection after confirming them in the decompiled goal. | S |
| `goals.rs:450` (`PounceGoal::can_start`) | `Fox.FoxPounceGoal.canUse` | Only `is_fully_crouched()` (server-side `crouch_amount`) is checked, not `is_crouching()`. `crouch_amount` retains 5.0 for the tick after crouching is cleared, so a stale full crouch can still start a pounce. | Gate on `fox.is_crouching() && fox.is_fully_crouched()` (verify against the decompiled `canUse` first). | S |
| `goals.rs:891-893` (`FoxSearchForItemsGoal::can_start`) | `Fox.FoxSearchForItemsGoal.canUse` | Requires `game_rules.mob_griefing`; item pickup is not a griefing action in vanilla and the goal's own `should_continue` is the only place the rule matters elsewhere. | Drop the mobGriefing gate here after confirming `canUse`/pickup path in the decompiled goal. | S |
| `mod.rs:551-567` (`tick_eating`) | `Fox.tickEating` | `ticks_since_eaten` is incremented unconditionally; vanilla advances it only while `canEat(getMainHandItem())` holds (food in mouth, no target, on ground, not sleeping), so a sleeping or airborne fox banks meal time and eats instantly on waking. | Move the increment inside the `can_eat` branch and keep the >560/>600 checks relative to it. | S |

## Notes (not gaps)
- Fox head roll and sitting pose interpolation are render-only (client side).
- `has_global_tick_reach` is a Pumpkin hook for the teleport-randomly consume effect, not vanilla `Fox`.
- `mod.rs:628` walk-over pickup gated on `mob_griefing` mirrors the shared `mob::pick_up_nearby_items`
  path; engine-wide, not fox-specific.
