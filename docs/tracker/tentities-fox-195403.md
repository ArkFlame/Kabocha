**FIXED** `82261e6bf` (fix(fox): block sleep onset in water or rain). VERIFIED against code: entity/passive/fox/goals.rs:718-721 (no sleep onset in water or rain).

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
| `mod.rs:877` (`get_ambient_sound`) | `Fox.getAmbientSound` gates the night screech on `!level.isBrightOutside()` | The row premise (`isDay()`) was wrong: decompiled vanilla uses `isBrightOutside()` | none: Rust already matches vanilla | S | **VERIFIED (matches vanilla)**

## Notes (not gaps)
- `goals.rs` (`has_shelter`) uses the motion-blocking heightmap; vanilla `Fox$FoxSleepGoal` shelter test needs decompiled confirmation (possible `canSeeSky` difference), left unchanged.
- `mod.rs:628` gates walk-over pickup on `mob_griefing`, matching the shared `mob::pick_up_nearby_items` path; vanilla `Mob.aiStep` has no such check but the difference is engine-wide, not fox-specific.
- `has_global_tick_reach` is a Pumpkin hook for the consumed teleport effect, not a vanilla `Fox` method.
- Render-only fox behaviour (head roll, pose interpolation) is client side; no server change.


## Result
- `mod.rs:877` (`get_ambient_sound`, the only STILL OPEN row): CLOSED as not a gap. Decompiled vanilla `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/fox/Fox.java:693` reads `if (!this.level().isBrightOutside() && this.random.nextFloat() < 0.1F)`, so vanilla uses `isBrightOutside()`, not `isDay()`. Rust `mod.rs:877` uses `!world.is_bright_outside() && rand::rng().random_range(0..10) == 0`, which matches. The screech volume/pitch (`Fox.java:681` vs `mod.rs:899`) and nearby-player/spectator check (`Fox.java:694-698` vs `mod.rs:879-884`) also match. No code change warranted; row also targets mod.rs, outside this unit's editable file (goals.rs only).
- No STILL OPEN rows remained in goals.rs, so no source files were edited and no unit tests were added. No rustfmt run needed.

- Re-verified this pass against decompiled source: Fox.java:693 uses isBrightOutside(), confirming the row suggested isDay() fix is incorrect and Rust at mod.rs:877 already matches. Result stands; no source file in this unit is editable (basenames: none).
- Independent re-check (decompiled source present): Fox.java:693 `!isBrightOutside() && nextFloat() < 0.1F` vs mod.rs:877 `!is_bright_outside() && random_range(0..10) == 0`; both gate on bright-outside and both give a 10% screech chance, so parity holds. is_bright_outside itself is engine-level (world crate), out of this unit's scope. No editable source files (basenames: none), so no code/test changes this pass.

- THIRD PASS: verified the sole STILL OPEN row against decompiled vanilla `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/fox/Fox.java:678-702`. getAmbientSound screech gate is `!level().isBrightOutside() && random.nextFloat() < 0.1F`; Rust `mod.rs:877` is `!world.is_bright_outside() && rand::rng().random_range(0..10) == 0` - same condition and same 10% chance. Nearby-player/spectator check (Fox.java:694-698 vs mod.rs:879-884) and screech volume/pitch (Fox.java:680-681 vs mod.rs:897-899) also match. The proposed `isDay()` fix was based on an incorrect premise and is not applied; no code change warranted. Row marked VERIFIED (matches vanilla); no source files edited this pass.


- RE-VERIFICATION (this pass): independently re-read decompiled vanilla `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/fox/Fox.java:688-702` and Rust `mod.rs:871-891`. Vanilla `getAmbientSound`: sleeping -> FOX_SLEEP; `!level().isBrightOutside() && random.nextFloat() < 0.1F` with NO_SPECTATORS nearby-player check -> FOX_SCREECH; else FOX_AMBIENT. Rust `mod.rs:877` matches exactly: `!world.is_bright_outside() && rand::rng().random_range(0..10) == 0` (10%) with spectator-excluding `get_nearest_player`; screech volume/pitch also match (Fox.java:680-681 vs mod.rs:897-899). The row's `isDay()` premise is incorrect. Row remains VERIFIED (matches vanilla, cite Fox.java:693); no code change warranted. No source files are editable in this unit (basenames: none), so no code/test edits and no rustfmt needed. No STILL OPEN rows remain.
