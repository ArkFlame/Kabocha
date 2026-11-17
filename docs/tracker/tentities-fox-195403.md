# entities / Fox (re-review)
Reviewed: `crates/pumpkin-core/src/entity/passive/fox/` (mod.rs, goals.rs) against vanilla `net.minecraft.world.entity.animal.Fox` and its inner goals (no decompiled source in this worktree; compared against vanilla behaviour).
Verdict: structure complete (goal list/priorities, variants, spawn variant, breeding, NBT, sounds, berries, pounce/stalk/shelter/sleep/perch/search, defend-trusted, alertable scan). The rain-handling divergence is fixed below; one small divergence remains open.

## Fixed in this audit
- `mod.rs` (`tick_states`): vanilla `Fox.tick` clears sleep and sitting with `isInWaterOrRain()`; Rust used water only. Now uses `is_in_water_or_rain`.
- `goals.rs` (`SleepGoal::can_start`, `should_continue`): vanilla `Fox$FoxSleepGoal` rejects water and rain; Rust checked water only. Now both use `is_in_water_or_rain`.
- `goals.rs` (`SleepGoal::can_start`): missing vanilla one-in-ten retry (`random.nextInt(adjustedTickDelay(10)) != 0`) made sleep onset deterministic. Added, gated to the start path only so `should_continue` does not re-roll.

## Open gaps

| file:line | vanilla | gap | fix | size |
|---|---|---|---|---|
| `mod.rs:877` (`get_ambient_sound`) | `Fox.getAmbientSound` gates the night screech on `!level.isDay()` | Rust uses `world.is_bright_outside()`, which is false in rain/thunder, so a fox screeches on a rainy day and stays silent on a dark-but-raining night | use a day-time check (`world.is_day()`) instead of `is_bright_outside` | S |

## Notes (not gaps)
- `goals.rs` (`has_shelter`) uses the motion-blocking heightmap; vanilla `Fox$FoxSleepGoal` shelter test needs decompiled confirmation (possible `canSeeSky` difference), left unchanged.
- `mod.rs:628` gates walk-over pickup on `mob_griefing`, matching the shared `mob::pick_up_nearby_items` path; vanilla `Mob.aiStep` has no such check but the difference is engine-wide, not fox-specific.
- `has_global_tick_reach` is a Pumpkin hook for the consumed teleport effect, not a vanilla `Fox` method.
- Render-only fox behaviour (head roll, pose interpolation) is client side; no server change.
