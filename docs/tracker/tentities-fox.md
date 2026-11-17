# entities / Fox
Reviewed: `crates/pumpkin-core/src/entity/passive/fox/` (mod.rs, goals.rs) against vanilla `net.minecraft.world.entity.animal.Fox` and its inner goals.
Verdict: implementation is complete in structure (goal list and priorities, variants, biome spawn variant, breeding, NBT, sounds, berries, pounce/stalk/shelter/sleep/perch/search, defend-trusted, alertable scan). Two small divergences were fixed; one open item remains.

## Fixed in this audit
- `mod.rs` (tick_states): vanilla `Fox.tick` only sends the level event 2001 block burst when `BlockState` is not air; the Rust path fired it unconditionally (air state id 0). Fixed to skip air. Commit 02fe6397b.
- `mod.rs` (`add_trusted`): vanilla `Fox.addTrustedUUID` removes `trustedUUIDs.get(0)` when the list is full, so the newest entry lands last; Rust overwrote the second slot and kept the oldest. Fixed to shift the surviving entry. Commit 8879c9000.

## Open gaps

| file:line | vanilla | gap | fix | size |
|---|---|---|---|---|
| `goals.rs:687` (`SleepGoal::can_sleep`) | `Fox.FoxSleepGoal.canUse` | Rust decides sleep from a fixed countdown, shelter, daylight and `is_alertable`; vanilla additionally rolls `random.nextInt(adjustedTickDelay(10)) != 0` before trying and rejects `isInWaterOrRain`. Sleep onset is therefore more deterministic and can start in rain. | Add the random retry gate and the water/rain rejection after confirming against decompiled `Fox$FoxSleepGoal`. | S |

## Notes (not gaps)
- `fox/mod.rs:628` gates the walk-over pickup on `mob_griefing`, matching the shared `mob::pick_up_nearby_items` path (`entity/mob/mod.rs:1369`); vanilla `Mob.aiStep` has no such check, but the difference is engine-wide and not fox-specific.
- `has_global_tick_reach` is a Pumpkin hook for the consumed teleport effect, not a vanilla `Fox` method; it matches the teleport-randomly consume path.
- Fox render-only behaviour (head roll, sitting pose interpolation) is client side; no server change.
