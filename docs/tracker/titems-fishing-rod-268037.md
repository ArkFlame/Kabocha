# items / Fishing Rod (RE-REVIEW)

Source: `crates/pumpkin-core/src/item/items/fishing_rod.rs`,
`crates/pumpkin-core/src/entity/projectile/fishing_bobber.rs`.
Vanilla reference: `net.minecraft.world.item.FishingRodItem`, `FishingHook`
(1.21.x mappings; no decompiled `.vendor/` copy in this worktree, so compared from the
known vanilla source). Master has no `docs/tracker/` entry for this row; this is the first
written review after the source changed.

## Fixed in this re-review

| Gap | Change |
|---|---|
| `fishing_rod.rs` removed whatever entity the id in `player.fishing_bobber` resolved to, even when that entity was not a `FishingBobberEntity`. A hook removed outside `discard()` (e.g. `/kill`) leaves a stale id; once the server reuses that entity id, the next reel-in removed an unrelated entity. Vanilla `FishingHook.retrieve` ends in `discard()`, which only removes the hook. | `remove()` moved inside the downcast branch (`fishing_rod.rs:96-107`). |
| `FishingHook` constructor clamps luck with `this.luck = Math.max(1, luck)`, so a rod without Luck of the Sea still fishes with base luck 1. Pumpkin clamped to 0, shifting the treasure/junk weights of `minecraft:gameplay/fishing`. | `luck_bonus.max(1)` (`fishing_bobber.rs:103`). |

## Remaining gaps

### 1. `fish_caught` statistic only counts tagged fish - size S

`fishing_bobber.rs:235`. Vanilla `FishingHook.retrieve` awards
`Stats.FISH_CAUGHT` for every looted entry of `gameplay/fishing`, so junk (leather boots,
lily pad) and treasure (bow, fishing rod, ...) also increment the stat. Pumpkin gates the
increment on the `minecraft:fishes` tag, so those catches never count. Fix: increment
`CustomStatistic::FishCaught` unconditionally in the loot loop. Verify the exact vanilla
placement before changing (Bukkit does not expose this stat per catch type).

### 2. Initial bite timer is rolled on cast, not on first water tick - size S

`fishing_bobber.rs:105-106`. Vanilla `FishingHook` starts with `timeUntilLured = 0`; the
first `catchingFish` call inside water takes the `else` branch and rolls
`100..=600 - lureSpeed`. Pumpkin rolls the same value in `new()`, so the countdown starts
2-3 ticks early (the ticks spent flying before the hook touches water) and `catchingFish`'s
re-roll branch is never used for the first cast. Behaviourally a 2-3 tick offset; fix by
initialising `wait_countdown` to 0.

### 3. `reel_in` bails out before applying retrieval effects - size S

`fishing_bobber.rs:126-128`. Vanilla `retrieve` only returns 0 on the client or without an
owner; it does not consult `shouldStopFishing`. Pumpkin returns 0 damage and skips the
entity pull / loot / rod damage when the owner is more than 1024 blocks away or holds no
rod in either hand. Reachable only in the single tick before `tick` discards such a hook,
so the observable difference is a lost pull and no durability loss. Fix: drop the
`should_stop_fishing` guard and let `retrieve` run, matching vanilla.

### 4. Extra arm swing on reel-in - size S

`fishing_rod.rs:106`. Vanilla `FishingRodItem.use` never calls `swing`, the client predicts
the animation and the server only forwards `ServerboundSwing`. Pumpkin calls
`swing_hand` explicitly, which can double-broadcast the animation for vanilla clients that
also send the swing packet. Verify with a client before removing; harmless if the packet is
idempotent.

## Reviewed and matching

Cast/retrieve sound volume and pitch `0.4 / (random * 0.4 + 0.8)` plus hover sound
categories; lure reduction `floor(modify * 20)`, luck bonus
`floor(modify_fishing_luck_bonus)`; `ITEM_USED` statistic and `ITEM_INTERACT_START/FINISH`
game events; throw origin/velocity from yaw+pitch with `triangle(0.5, 0.0103365)` per axis;
`ProjectileOwner` spawn data; `shouldStopFishing` rod-in-either-hand and 1024-distance
check plus `canInteractWithLevel`; grounded 1200-tick expiry with 2 damage on retrieve;
water entry damping `(0.3, 0.2, 0.3)`; `bob_velocity` damping toward the water surface;
10-tick out-of-water discard; `catchingFish` rain/sky speed, nibble/hook/lure branches,
tease chances `0.15 / +0.05 / +0.02 / +0.01`, splash particles and splash sound
`triangle(1, 0.4)`, bite window `20..=40`, `DATA_BITING` sync and `-0.4*(r*0.4+0.6)` kick
from the UUID-seeded `synced_random`; entity/block ray-cast hooking with `HOOKED_ENTITY`
sync (`id + 1`); retrieve damage `3` (item entity) / `5` (other) / `1` (fish) / `2`
(on ground, overriding); loot roll through `gameplay/fishing` with tool, position, hook
entity and luck `hook_luck + player LUCK`; item entity velocity `delta*0.1 + sqrt(sqrt(d2))*0.08`
and `1..=6` XP orbs; `PlayerFishEvent` states `Fishing`, `Bite`, `FailedAttempt`,
`CaughtEntity`, `CaughtFish` with cancellation; `FISHING_ROD_HOOKED` advancement trigger.
