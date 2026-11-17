# Tracker: entities / Item

Reviewed: `crates/pumpkin-core/src/entity/item.rs` at branch `kab/tentities-item-0024a5`
(re-review: the source changed after the `kab/tentities-item` review, so every row below was
re-checked against the current file).
Vanilla reference: `net.minecraft.world.entity.item.ItemEntity` (1.21.x), `Player.drop`,
`Player.take`, `Player.giveExperiencePoints`, `Containers.dropContents`, `BundleItem.onDestroyed`.

## Fixed in this review

| Gap | Change |
|---|---|
| `is_mergeable` refused never-despawn (`Age: -32768`) and never-pickup (`PickupDelay: 32767`) items, and `process_age_and_merge` returned before merging them, so vanilla items that should stack never did. Vanilla `isMergable` only checks `!isRemoved()` and `count < getMaxStackSize()`. | Removed both guards (`item.rs:230`) and let never-despawn items fall through to the merge block with a frozen age (`item.rs:546`). |
| `decrement_pickup_delay` counted the `INFINITE_PICKUP_DELAY` sentinel (`u8::MAX`) down, so a merged never-pickup stack became collectable after ~255 ticks. Vanilla `ItemEntity.tick` skips `pickupDelay == 32767`. | Early return on `u8::MAX` (`item.rs:450`). |
| Spilled container/bundle contents were spawned with `set_pickup_delay(0)`; vanilla spills through `popResource` -> `new ItemEntity(...)`, which keeps the default 10-tick delay. | Removed the reset (`item.rs:398` `on_destroyed`), so `Self::new` keeps `DEFAULT_PICKUP_DELAY`. |
| Player drops never recorded a thrower, so the owner pickup protection could not work. Vanilla `Player.drop(stack, dropAround)` delegates to `drop(..., traceItem = true)` and calls `setThrower(this)`. | `item_entity.set_thrower(self.gameprofile.id)` in `Player::drop_item` (`player.rs:6628`). |
| `on_player_collision` ignored the thrower entirely; vanilla `ItemEntity.playerTouch` only lets the thrower (or any player after `age > 60`) collect. | Early return while `item_age <= 60` and the thrower resolves to a different player (`item.rs:768`). Non-player throwers (allay, fox) do not block, matching the previous review note. |

Death drops are unaffected: `Player` death uses `world.drop_stack` (`player.rs:6271`), which
spawns items without a thrower, like vanilla `Inventory.dropAll` with `traceItem = false`.

## Remaining gaps

### 1. Despawn still runs before the merge check - size S

`item.rs:546` (`process_age_and_merge`). The function increments `age`, discards at `LIFETIME`
and returns `false`, and only the surviving path calls `merge_with_neighbours` (`item.rs:575`).
Vanilla `ItemEntity.tick` merges neighbours before the `age >= 6000` discard, so the last tick
of a 6000-tick item can still merge into a neighbour. One tick of skew. Fix: run the merge block
before the despawn block and return `false` after a successful despawn.

### 2. `is_mergeable` keeps an age guard vanilla does not have - size S

`item.rs:232`. Vanilla `isMergable` is `!isRemoved() && count < getMaxStackSize()`; Pumpkin also
rejects `item_age >= LIFETIME`. Observable only on the despawn tick, so it is harmless today but
it becomes a divergence once gap 1 is fixed (the item must merge on that tick). Remove the guard
together with gap 1.

### 3. A merged never-pickup stack does not carry the flag - size S

`item.rs:380` (`try_merge_with`). `pickup_delay` is merged with `fetch_max`, which saturates at
`u8::MAX` and therefore keeps the item unpickable, but `never_pickup` is not propagated.
`write_custom_nbt` (`item.rs:889`) then writes `PickupDelay = 255` instead of the `32767`
sentinel, and a reload decodes 255 as never-pickup, so the value self-heals but the chunk NBT is
not byte-compatible with vanilla. Fix: `target.never_pickup.store(target || source)` before
`on_count_changed`.

### 4. Merger takes the minimum age, vanilla leaves the target age - size S

`item.rs:385`. `target.item_age.fetch_min(source.item_age)` shortens the target's lifetime when
merging into a fresher item; vanilla `ItemEntity.merge` only takes the max pickup delay and does
not touch `age` (only `target` survives, with its own age). Low impact, but it is an invented
rule; delete the `fetch_min` if vanilla ordering is confirmed against decompiled source.

## Reviewed and matching

Velocity/gravity 0.04, water and lava drag with buoyancy, `noPhysics` push-out, every-4th-tick
resting movement, block friction and half-speed bounce, merge radius 0.5 with rate 2/40 and the
64 cap, atomic reservation plus `ItemMergeEvent` cancel, Bedrock `UpdateStackSize` resync,
`can_be_hurt_by` fire immunity, integer health with `mobGriefing`, container and bundle spill on
destroy, advancement `ThrownItemPickedUpByPlayer` and the `PickedUp` statistic, Java item
metadata on spawn, `Item`/`Age`/`PickupDelay`/`Health`/`Thrower` NBT round trip, pickup sound.

Known intentional deviation: creative players destroy a stack that fits nowhere
(`item.rs:790`, upstream #4383); vanilla `Inventory.add` leaves it on the ground.
