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

## Gap status
- 1 owner pickup protection (M): VERIFIED (matches vanilla 1.21.x decompile)
- 2 player drops record thrower (M): VERIFIED (matches vanilla 1.21.x decompile)
- 3 never-despawn items do not merge (S): VERIFIED (doc premise wrong; code already matches vanilla)
- 4 spilled contents pickup delay (S): VERIFIED (doc premise wrong; code already matches vanilla)
- 5 despawn before merge (S): VERIFIED (matches vanilla 1.21.x decompile)

## Result

Verified 2026-11-17 against `.vendor/vanilla-decompiled/net/minecraft/` (Paper/Canvas-flavoured
1.21.x; unmarked lines are Mojang vanilla). Every STILL OPEN row is already fixed in the main
checkout by the item.rs / player.rs owners; this unit's editable files (allay.rs, utils.rs) need
no change. No .rs files touched, so no rustfmt run.

- 1 owner pickup protection (M): ALREADY FIXED in `item.rs:780-792`. Code returns when
  `item_age <= 60 && thrower resolves to a player via get_player_by_uuid && thrower != picker`,
  exactly the proposed fix; allay/fox throwers are not players so they do not block. Note the
  current decompiled `ItemEntity.playerTouch` (ItemEntity.java:490) actually gates on
  `pickupDelay == 0 && (target == null || target.equals(player.getUUID()))` with no age/thrower
  check; the doc's quoted `owner`/`age > 60` rule is older vanilla. No action.
- 2 player drops record thrower (M): ALREADY FIXED in `player.rs:6851`
  (`item_entity.set_thrower(self.gameprofile.id)` beside the drop velocity construction). The
  existing allay.rs:205 and utils.rs:289 `set_thrower` calls match vanilla
  `BehaviorUtils.throwItem` (BehaviorUtils.java:100-105) and Fox/Dolphin (Fox.java:535,
  Dolphin.java:534). No action.
- 3 never-despawn items do not merge (S): ALREADY FIXED in `item.rs:230-239` (is_mergeable now
  checks only `!removed && count < max`) and `item.rs:549-566` (age held at -32768, merge kept).
  VERIFICATION WARNING for the item.rs owner: vanilla `isMergable` (ItemEntity.java:276-279) DOES
  check `pickupDelay != 32767 && age != -32768 && age < despawnRate`; the current Pumpkin code now
  merges never-pickup/never-despawn items where vanilla does not. Doc premise was wrong; leaving
  item.rs untouched (outside this unit) but flagging the divergence.
- 4 spilled contents pickup delay (S): ALREADY FIXED in `item.rs:423-428` (`set_pickup_delay(0)`
  removed; `Self::new` defaults pickup delay to 10). VERIFICATION WARNING for the item.rs owner:
  vanilla `Containers.dropItemStack` (Containers.java:42-47) and `ItemUtils.onContainerDestroyed`
  (ItemUtils.java:50-54) spawn via plain `new ItemEntity` with NO `setDefaultPickUpDelay()`, so
  container/bundle spill has delay 0; only `Block.popResource` (Block.java:483) sets 10. The
  item.rs comment/pattern therefore diverges from this vanilla build. Leaving untouched (outside
  this unit) but flagging.
- 5 despawn before merge (S): ALREADY FIXED in `item.rs:549-566`; merge runs before age increment
  and the 6000-tick discard, matching vanilla `ItemEntity.tick` (ItemEntity.java:202-227). No action.

Net: no source edits this unit; two verification discrepancies (rows 3 and 4) flagged for whoever
owns item.rs. No build run (none required: no .rs changed).


## Result (2026-11-18 re-verification)

Re-checked all 5 STILL OPEN rows against `.vendor/vanilla-decompiled/net/minecraft/` (Paper/Canvas
1.21.x). Every fix lives in `item.rs` / `player.rs`, which are OUTSIDE this unit's editable files
(`allay.rs`, `utils.rs`). No source edits made, no rustfmt run.

- 1 owner pickup protection (M): LEAVE OPEN - fix is in `item.rs:826-838` (outside unit) and is
  already present: return when `item_age <= 60 && thrower != picker && thrower resolves via
  get_player_by_uuid`; target gate at `item.rs:822`. Matches vanilla `ItemEntity.playerTouch` plus
  `Player.take` (ItemEntity.java:423). allay/fox throwers are not players, so they do not block.
- 2 player drops record thrower (M): LEAVE OPEN - fix is in `player.rs:6991` (outside unit) and is
  already present (`item_entity.set_thrower(self.gameprofile.id)`). The editable-file calls
  `allay.rs:205` and `utils.rs:289` match vanilla `BehaviorUtils.throwItem`
  (BehaviorUtils.java:100-105: setThrower + setDefaultPickUpDelay). No edit needed.
- 3 never-despawn merge (S): LEAVE OPEN - `item.rs:250-272` (outside unit). Current `can_merge`
  checks `!removed && !never_despawn && !never_pickup && count < max`, matching vanilla
  `isMergable` (ItemEntity.java:276-278: alive, pickupDelay != 32767, age != -32768,
  age < despawnRate, count < max). Doc premise (never-despawn should merge) was wrong; no divergence
  now. The 2026-11-17 note claiming guards were removed is stale.
- 4 spilled contents pickup delay (S): LEAVE OPEN - `item.rs:461-467` (outside unit). Current code
  keeps `set_pickup_delay(0)`, matching vanilla `Containers.dropItemStack` /
  `ItemUtils.onContainerDestroyed`, which spawn a plain ItemEntity at delay 0 (no
  setDefaultPickUpDelay); only Block.popResource sets 10. The 2026-11-17 note claiming the call was
  removed is stale; doc premise (vanilla default 10) was wrong. No change needed.
- 5 despawn before merge (S): LEAVE OPEN - `item.rs:589-628` (outside unit). Merge runs at 597-599
  before the age increment (603-607) and 6000-tick discard (609), matching vanilla
  `ItemEntity.tick` (ItemEntity.java:203-219). No divergence.

Net: no source edits this unit; all 5 gaps already addressed in item.rs/player.rs by their owners.
The only editable-file check (set_thrower in allay.rs/utils.rs) already matches vanilla.

## Result (2026-11-18 final)

Re-verified all 5 STILL OPEN rows against `.vendor/vanilla-decompiled/net/minecraft/`. Every row's
fix lives in `item.rs`/`player.rs`, OUTSIDE this unit's editable files (`allay.rs`, `utils.rs`);
the only editable-file behaviour (`set_thrower` in `allay.rs:205`, `utils.rs:289`) already matches
vanilla `BehaviorUtils.throwItem` (setThrower + default pickup delay). No .rs file touched; no
rustfmt run.

- 1 owner pickup protection (M): LEAVE OPEN - fix in `item.rs:826-838` (outside unit); returns when
  `item_age <= 60 && thrower != picker && thrower resolves via get_player_by_uuid`.
- 2 player drops record thrower (M): LEAVE OPEN - fix in `player.rs:7030` (outside unit);
  editable-file `set_thrower` calls already match vanilla.
- 3 never-despawn merge (S): LEAVE OPEN - `item.rs:250-272` (outside unit); `can_merge` matches
  vanilla `isMergable` (excludes never-despawn/never-pickup/removed).
- 4 spilled contents pickup delay (S): LEAVE OPEN - `item.rs:466` (outside unit); keeps
  `set_pickup_delay(0)`, matching vanilla container spill at delay 0.
- 5 despawn before merge (S): LEAVE OPEN - `item.rs:589-628` (outside unit); merge runs before
  age increment/discard, matching vanilla `ItemEntity.tick`.

Net: no source edits; all fixes already present outside this unit. No build run (no .rs changed).


## Result (2026-11-18 third pass - final)

Independently re-verified all 5 rows against the decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/`. This pass may edit any source file; no source
change is warranted because every row already matches vanilla. No .rs file touched, so no
rustfmt run and no build required.

- 1 owner pickup protection (M): VERIFIED (matches vanilla) - `ItemEntity.playerTouch`
  (ItemEntity.java:490) gates only on `pickupDelay == 0 && (target == null ||
  target.equals(player.getUUID()))`. The doc's `owner`/`age > 60` rule is old vanilla and
  must NOT be added. Pumpkin `on_player_collision` (item.rs:815,825) checks pickup_delay and
  `target_allows_pickup` (item.rs:196-198, `target.is_none_or(|owner| owner == player)`),
  which matches. Thrower is not a pickup gate in this build (only advancement/NBT). No fix;
  adding the age gate would be a regression.
- 2 player drops record thrower (M): VERIFIED - `player.rs:7046` calls
  `item_entity.set_thrower(self.gameprofile.id)` on the Player.drop path (beside
  `new_with_velocity(..., 40)`), matching vanilla `Player.drop`. Editable-file calls
  `allay.rs:205` and `utils.rs:289` set the thrower and match `BehaviorUtils.java:100-105`
  (setThrower + setDefaultPickUpDelay). No fix needed.
- 3 never-despawn items do not merge (S): VERIFIED - vanilla `isMergable`
  (ItemEntity.java:278) excludes `pickupDelay == 32767` and `age == -32768` from merging, so
  the doc premise (never-despawn/never-pickup should merge) is wrong. Pumpkin `can_merge`
  (item.rs:264-272, `!removed && !never_despawn && !never_pickup && count < max`) correctly
  excludes them; unit test `never_pickup_or_never_despawn_items_never_merge`
  (item.rs:1124-1129) guards it. No fix needed.
- 4 spilled contents pickup delay (S): VERIFIED - vanilla container/bundle spill spawns a
  plain `new ItemEntity` at delay 0 (`Containers.dropItemStack` Containers.java:42,
  `ItemUtils.onContainerDestroyed` ItemUtils.java:50); only `Block.popResource` sets 10.
  Pumpkin `Self::new` defaults to `DEFAULT_PICKUP_DELAY = 10` (item.rs:108,125), so
  `on_destroyed` calling `set_pickup_delay(0)` (item.rs:466) is required to match vanilla 0.
  Doc premise (vanilla default 10) wrong; keep the call. No fix needed.
- 5 despawn before merge (S): VERIFIED - vanilla `ItemEntity.tick` merges at
  ItemEntity.java:203-205 BEFORE `age++` (207-209) and the despawn discard (219). Pumpkin
  `process_age_and_merge` (item.rs:597-609) merges at 597-599 before age increment (603-607)
  and discard (609); matches. No fix needed.

Net: no source edits this pass; all 5 rows VERIFIED against the 1.21.x vanilla decompile.
