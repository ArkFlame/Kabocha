# Tracker: entities / Item

Reviewed: `crates/pumpkin-core/src/entity/item.rs` at branch `kab/tentities-item-0024a5`
(re-review after the earlier `kab/tentities-item` review; every row re-checked against the
current file).
Vanilla reference: `net.minecraft.world.entity.item.ItemEntity` (1.21.x), `Player.drop`,
`Player.take`, `Containers.dropContents`, `BundleItem.onDestroyed`. No decompiled tree is
checked out in this worktree (`.vendor/` is absent), so the ordering claims below are marked as
needing a decompiled confirmation.

## Fixed in this review

| Gap | Change |
|---|---|
| A merged never-pickup stack lost the flag. Vanilla `PickupDelay` is an `int`, so `Math.max` keeps `32767` across a merge; Pumpkin's `u8` field saturates at `255` and `never_pickup` was not copied, so `write_custom_nbt` saved `PickupDelay = 255` (which decodes back as a 255-tick delay, not the sentinel). | `target.never_pickup` is now set from the source in `try_merge_with` (`item.rs:383`). |

## Remaining gaps

### 1. Despawn runs before the merge check - size S, needs decompiled confirmation

`item.rs:551` (`process_age_and_merge`): the `age >= LIFETIME` branch discards and returns `false`
before `merge_with_neighbours` (`item.rs:583`). If vanilla `ItemEntity.tick` merges neighbours
before the `age >= 6000` discard, the last tick of a 6000-tick item can still merge into a
neighbour in vanilla but never in Pumpkin. Fix: run the merge block first, then the discard.
Confirm the order in decompiled `ItemEntity.tick` before changing it.

### 2. `is_mergeable` keeps an age guard vanilla does not have - size S

`item.rs:232`. Vanilla `isMergable` is `isAlive() && getCount() < getMaxStackSize()`. The
`item_age >= LIFETIME` guard is unreachable today (the merge only runs when
`process_age_and_merge` returned `true`), so this is dead code, not an observable divergence.
Remove it together with gap 1.

### 3. Persisted `Health` tag is Pumpkin-only - size S

`item.rs:916` writes `Health` and `item.rs:948` reads it back. Vanilla `ItemEntity` has an
in-memory `health = 5` field and `addAdditionalSaveData` stores only `Item`, `Age`,
`PickupDelay`, `Thrower`/`Owner`; health resets to 5 on load. Saving the extra tag makes chunk
NBT differ from vanilla byte-for-byte (harmless to vanilla readers). Fix: drop the write and the
read.

### 4. Owner and Thrower are collapsed into one UUID - size M, needs decompiled confirmation

`item.rs:773` (`on_player_collision`) gates only when the thrower resolves to a player
(`get_player_by_uuid`), so an allay/fox-thrown item is collectable immediately. Vanilla keeps a
separate `owner` (used by `playerTouch`) from `thrower`, and writes both `Owner` and `Thrower` to
NBT (`item.rs:918`). Confirm the 1.21 field split and the exact `playerTouch` condition before
porting.

## Reviewed and matching

Velocity/gravity 0.04, water and lava drag with buoyancy, `noPhysics` push-out, every-4th-tick
resting movement, block friction and half-speed bounce, merge radius 0.5 with rate 2/40 and the
64 cap, atomic reservation plus `ItemMergeEvent` cancel, larger stack becomes the merge target,
max pickup delay and min age on merge (vanilla `ItemEntity.merge` sets
`target.age = Math.min(target.age, source.age)`; the earlier review flagged this as invented, it
is vanilla), Bedrock `UpdateStackSize` resync, `can_be_hurt_by` fire immunity and integer item
health with `mobGriefing`, container and bundle spill on destroy with the default pickup delay,
advancement `ThrownItemPickedUpByPlayer` and the `PickedUp` statistic, Java item metadata on
spawn, `Item`/`Age`/`PickupDelay`/`Thrower` NBT round trip, pickup sound, thrower pickup
protection and the `-32768`/`32767` sentinels.

Known intentional deviation: creative players destroy a stack that fits nowhere
(`item.rs:810`, upstream #4383); vanilla `Inventory.add` leaves it on the ground.

## Gap status
- 1 despawn before merge check (S, needs decompile): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 2 is_mergeable age guard (S, dead code after gap 1): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 3 Pumpkin-only Health NBT tag (S): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 4 owner and thrower collapsed (M): STILL OPEN (size M)
