# Deep audit: crates/pumpkin-core/src/entity/projectile/fishing_bobber.rs

Row: `audit | crates/pumpkin-core/src/entity/projectile/fishing_bobber.rs | AUDIT`
Date: 2026-11-17. Reference: vanilla `net.minecraft.world.entity.projectile.FishingHook`
(Yarn `FishingBobberEntity`) plus CraftBukkit `FishingHook.retrieve`. The decompiled class is
not present in this worktree (`.vendor/` is absent), so items marked *unverified* were compared
from knowledge of the 1.21 sources and must be confirmed from a decompiled copy before patching.

The previous pass (`docs/tracker/tentities-fishing-bobber.md`) covered the file broadly. This
pass re-checked it line by line and corrects three of its claims (see "Corrections" below), fixes
two gaps, and records the remainder.

## Fixed in this pass

| Gap | Rust location | Vanilla | Fix |
|-----|---------------|---------|-----|
| Bobber spawned exactly at eye height. | `fishing_bobber.rs:88-94` | `FishingHook` constructor: `setPos(x - sin*0.3, getEyeY() - 0.1, z - cos*0.3)` | `fix(fishing): spawn the bobber 0.1 below the owner eye` (eye height - 0.1). |
| Caught items had pickup delay 0. | `fishing_bobber.rs:227-231` | `new ItemEntity(level, x, y, z, stack)` keeps the default 10-tick `pickupDelay` | `fix(fishing): delay pickup of caught items by ten ticks` (`new_with_velocity(.., 10)`). |

## Remaining gaps

| # | Gap | Rust location | Vanilla / Paper | Suggested fix | Size |
|---|-----|---------------|-----------------|---------------|------|
| 1 | No open-water tracking; treasure is catchable from a 1x1 pond because the loot predicate only probes the hook's own block. | `fishing_bobber.rs:49-60`; `world/loot/predicates.rs:363-378` | `FishingHook.openWater` + `calculateOpenWater`/`getOpenWaterTypeForBlock/Area` (5x4x5 scan) | Add `open_water: AtomicBool` updated each BOBBING tick while no bite is pending; expose it to the predicate. *Unverified* scan bounds/ordering. | M |
| 2 | `CAUGHT_FISH` fires once per loot stack and each stack spawns its own 1..=6 XP orb. | `fishing_bobber.rs:203-242` | CraftBukkit `FishingHook.retrieve`: one event with `setExpToDrop(random.nextInt(6)+1)`, cancel discards the hook, then one orb at `player.y+0.5, z+0.5` after the item loop; vanilla itself drops no XP | Hoist the event and the XP roll out of the loop, spawn one orb of `event.exp_to_drop` after it, discard on cancel. Changes plugin-visible event count, so land with the next plugin API break and note it there. | S |
| 3 | `PlayerFishState::InGround` is never fired. | `plugin/api/events/player/fish.rs:11-32`; `reel_in` `fishing_bobber.rs:124-259` | CraftBukkit fires `IN_GROUND` in `retrieve` when `onGround()` | Fire at the end of `reel_in` when `on_ground` and damage is 2. | S |
| 4 | `FailedAttempt` fire point differs from Bukkit: fired when the bite window expires, never on an empty reel. | `fishing_bobber.rs:563` vs `124-259` | CraftBukkit fires `FAILED_ATTEMPT` in `retrieve` when there is no hooked entity and no nibble | Keep for the next API break (existing fire point is documented plugin behaviour). | S |
| 5 | Lure wait reduction scale looks 4x too large. `modify_fishing_time_reduction` sums the data-driven `fishing_time_reduction` effect (value 20 per level in vanilla data), then `fishing_rod.rs:65` multiplies by 20 -> 400 ticks per Lure level. | `fishing_rod.rs:63-66`; `fishing_bobber.rs:108, 679` | `FishingHook`: `timeUntilLured -= lureSpeed * 20 * 5` with `lureSpeed` the enchantment level, i.e. 100 ticks (5 s) per level | *Unverified*: read `EnchantmentHelper.getFishingTimeReduction` and the `lure` effect value from a decompiled copy, then pick the single matching constant. Verify the effect value before changing; do not guess. | S |
| 6 | `catching_fish` only runs while the float's block is water; a dry float freezes all three countdowns. | `fishing_bobber.rs:398-415` | `FishingHook.tick` calls `catchingFish(blockPos)` on every server-side BOBBING tick | *Unverified*: confirm ordering against the decompiled BOBBING branch, then call it unconditionally and let the out-of-water counter handle the dry case. | S |
| 7 | `out_of_water_time` semantics. Current: decrement (saturating) in water, increment clamped to 10 when dry, `discard()` at `>= 10`. | `fishing_bobber.rs:386-426` | `FishingHook.outOfWaterTime`: *unverified* whether it resets or clamps, and whether the threshold is `>= 10` or `> 10` | Keep as is unless a decompiled check says otherwise; the discard already exists (the older tracker note claiming it was missing is stale). | S |
| 8 | `FISHING_BOBBER` has no `from_type` arm, so a loaded/summoned bobber degrades to a plain `Entity`. | `entity/type.rs:151-365` | `EntityType.FISHING_BOBBER` reconstructs `FishingHook`; null owner discards on tick | Add an arm rebuilding the bobber from `ProjectileOwner` NBT, or discard on load without an owner (MVP). | M |
| 9 | The bite kick (`velocity.y = -0.4 * (r*0.4+0.6)`) draws from the UUID-seeded synced random. | `fishing_bobber.rs:266-271, 640-642, 399` | Vanilla takes that kick in `onSyncedDataUpdated` using the shared `this.random`, while `syncedRandom` only damps the biting float (`-0.1 * nextFloat * nextFloat`) | Cosmetic RNG divergence only; leave unless a deterministic replay test needs it. | XS |

## Corrections to the previous audit

- Gap "`PlayerFishState::Fishing` never fired" is stale: `net/java/play/use_item.rs:177-204`
  fires it for a cast and honours `cancelled()`. Only `InGround` is still missing (gap 3).
- Gap "`out_of_water_time` never acts / no discard" is stale: `fishing_bobber.rs:424-426`
  discards at `>= MAX_OUT_OF_WATER_TIME`.
- Gap "XP orb has no random Y offset / wrong X" is wrong: `fishing_bobber.rs:234-237` matches
  CraftBukkit (`player.y + 0.5, z + 0.5`). Only the per-stack count is wrong (gap 2).

## Verified as matching vanilla (re-checked)

- Throw setup: two triangle draws per axis with deviation `0.0103365`, origin `-sin*0.3/-cos*0.3`,
  velocity `(-sin, clamp(-sin_pitch/cos_pitch, ±5), -cos) * (0.6/len + triangle)` per axis.
- FLYING -> BOBBING sets `velocity * (0.3, 0.2, 0.3)` and returns; gravity `0.03` only while
  airborne and dry; inertia `0.92`; grounded `life` reset vs 1200-tick despawn.
- `checkCollision` predicate (`not self`, `not spectator`, `can_be_hit_by_projectile || ITEM`,
  owner gate) and `onHitBlock` momentum shortening.
- `retrieve` damage 3/5, ground override 2, pull velocity `delta * 0.1`, entity event 31,
  loot `luck_bonus + player luck`.
- `catchingFish` constants: rain `0.25`/sky `0.5` speed modifiers, nibble `20..=40`,
  hooked `20..=80`, lured `100..=600`, triangle `9.188` angle drift, tease table and splash
  particles, bite splash sound `1.0 + (a-b)*0.4`, bite particles `1 + width*20`.
- `shouldStopFishing` (rod in either hand, `canInteractWithLevel`, distance^2 > 1024).
- Synced data: `HOOKED_ENTITY = id + 1`, `DATA_BITING` boolean round trip.
