# Tracker: entities / Item

Reviewed: `crates/pumpkin-core/src/entity/item.rs` at branch `kab/tentities-item` (after
`fix(entity): honour item never-despawn and never-pickup NBT sentinels` and
`fix(entity): play the item pickup sound and honour never-pickup items`).
Vanilla reference: `net.minecraft.world.entity.item.ItemEntity` (1.21.x), `Player.take`,
`BlockItem.onDestroyed` / `BundleItem.onDestroyed`.

## Fixed in this review

| Gap | Change |
|---|---|
| `never_despawn` / `never_pickup` were never written, so NBT `Age: -32768` (vanilla "never age or despawn") and `PickupDelay: 32767` (`INFINITE_PICKUP_DELAY`) round-tripped as `4294934528` / `255` and despawned or unlocked after 255 ticks. | `decode_age` / `decode_pickup_delay` map the sentinels, `write_custom_nbt` emits them back (`item.rs:874`, `item.rs:901`), test `nbt_age_and_pickup_sentinels`. |
| No pickup sound: vanilla `Player.take` plays `entity.item.pickup` at volume 0.2 and pitch `((rand-rand)*0.7+1)*2`. | `on_player_collision` plays `Sound::EntityItemPickup` when an amount was taken (`item.rs:766`). |
| `never_pickup` items were only excluded from merging, a player still collected them. | `on_player_collision` returns early for `never_pickup`. |

## Remaining gaps

### 1. Owner pickup protection is missing - size M

`item.rs:766` (`on_player_collision`). Vanilla `ItemEntity.playerTouch`:

```java
if (this.pickupDelay == 0
        && (this.owner == null || this.owner.getUUID().equals(player.getUUID()) || this.age > 60)) { ... }
```

Pumpkin's `thrower` is stored (`item.rs:167`) but the pickup path never consults it, so any
player can instantly grab an item another player threw or dropped. Fix: after the
`pickup_delay` check, return when `item_age <= 60`, the throwing UUID resolves to a player
(`get_player_by_uuid`) and that player is not the picker. Note vanilla's `owner` is a player
reference, so allay/fox throwers must not block pickup.

### 2. Player drops never record the thrower - size M

`crates/pumpkin-core/src/entity/player.rs` item drop path; only
`allay.rs:205`, `fox/mod.rs:492`, `dolphin/goals.rs:561`, `ai/brain/behavior/utils.rs:251`
call `set_thrower`. Vanilla sets the owner on `Player.drop`, so death-drop protection
(3 s against other players) cannot work. Fix: call `set_thrower(player uuid)` on dropped item
entities, then gap 1 becomes observable.

### 3. Never-despawn items do not merge - size S

`item.rs:230` (`is_mergeable`) rejects `never_despawn` and `never_pickup`, and
`item.rs:545` (`process_age_and_merge`) returns before merging for `never_despawn`. Vanilla
`isMergable()` only checks `!isRemoved()` and `count < getMaxStackSize()`; `PickupDelay`
32767 and `Age` -32768 items still merge. Fix: drop the two guards from `is_mergeable`, and
in `process_age_and_merge` skip only the age increment/despawn (keep the merge block).

### 4. Spilled contents have no pickup delay - size S

`item.rs:426` (`on_destroyed`) sets `spilled.set_pickup_delay(0)`. Vanilla
`Containers.dropContents` / `BundleItem.onDestroyed` spawn the contents through
`popResource`, which leaves the default `ItemEntity` delay (10 ticks). Fix: drop the
`set_pickup_delay(0)` call.

### 5. Despawn check runs before the merge check - size S

`item.rs:545` (`process_age_and_merge`). Vanilla `ItemEntity.tick` merges neighbours first
(rate 2/40 on `tickCount`), then increments `age` and discards at 6000. Pumpkin increments
`age` and discards before merging, so the last tick of a 6000-tick item cannot merge. One
tick of skew; fix by moving the age block after `merge_with_neighbours`.

## Reviewed and matching

Gravity 0.04, water/lava drag and buoyancy (`apply_fluid_drag_or_gravity`), `noPhysics`
push-out, every-4th-tick resting movement, ground friction and half-speed bounce, merge
distance 0.5 / rate 2 or 40 / cap 64 with atomic reservation and plugin cancel, item count
resync (Bedrock `UpdateStackSize`), fire immunity from `can_be_hurt_by`, damage health as
truncated int with `mobGriefing`, container and bundle spill on destroy, AdvancementTrigger
`ThrownItemPickedUpByPlayer` plus `PickedUp` statistic, Java item metadata on spawn, NBT
`Item`/`Age`/`PickupDelay`/`Health`/`Thrower`.

Known intentional deviation: creative players destroy a stack that fits nowhere
(`item.rs:786`, upstream #4383); vanilla `Inventory.add` leaves it on the ground.
