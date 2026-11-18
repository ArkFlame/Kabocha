# entities / Bamboo Chest Raft

Coverage: RE-REVIEW-UNTRACKED (source changed under the previous assessment).
Audited: 2026-11-16.

Source: `crates/pumpkin-core/src/entity/vehicle/boat.rs` (`BoatEntity` handles all 22 boat and
chest boat types, including `BAMBOO_CHEST_RAFT`), registration in `entity/type.rs:337`,
`entity/vehicle/boat.rs:36` (`is_chest_boat`), item placement in `item/items/boat.rs:44`,
loot/damage helpers in `entity/vehicle/vehicle.rs`.

## Covered (matches vanilla AbstractBoat / ChestBoat)

- 27-slot container only for chest boats/rafts (`boat.rs:31,60`), custom name + contents NBT
  (`boat.rs:167-177`), container screen titled from the entity type (`boat.rs:75-102`).
- Sneak or full/sealed boat opens the container, otherwise boards (`boat.rs:286`), matching
  `ChestBoat.interact` routing through `canAddPassenger`/`isSecondaryUseActive`.
- 60-tick underwater boarding lock (`boat.rs:215-220,286`), 1 passenger for chest rafts, 2 for
  plain rafts (`boat.rs:71`).
- Damage wobble, creative instant break, hurt metadata (`vehicle.rs:216-285`).
- Paddle metadata driven by `ServerboundPaddleBoat` (`net/java/play/paddle_boat.rs:5`).
- Bamboo raft bedrock title fallback (`boat.rs:80-88`).
- Lava ignition/destruction via fluid `entityInside` (`entity/mod.rs:6599+`).

## Fixed in this audit

- `boat.rs:208 tick` never ran `Entity.baseTick`, so boats kept `first_tick`, never decremented
  fire ticks (a burning boat burned forever), never decremented `riding_cooldown` (dismounted
  player could not re-board), never ran the portal/out-of-world checks and never updated
  `last_pos`. Now runs `base_tick_without_fluid_push` + `update_last_pos`
  (`entity/mod.rs:6585-6612`, boat.rs:208-219).

## Remaining gaps

| # | Location | Vanilla class/method | Gap | Fix | Size |
|---|---|---|---|---|---|
| 1 | `boat.rs:233` | `AbstractBoat.canBeCollidedWith` | Returns `true` unconditionally; vanilla returns `!isRemoved()` | `!self.vehicle.entity.is_removed()` | S, FIXED (1062af9ec) |
| 2 | `boat.rs:286` | `AbstractBoat.interact`, `ChestBoat.interact` | No `is_alive()`/removed guard; a dead/removed boat can still board a player or open its menu | Early `PASS` when `!is_alive()` | S, FIXED (1062af9ec): `AbstractBoat::interact` returns false when not alive |
| 3 | `boat.rs:208` | `AbstractBoat.tick` -> `tickBubbleColumn` | Upward bubble columns do not lift the boat, downward ones do not sink it | Read `BubbleColumnBlock` state under the boat and set vertical velocity, then flag `velocity_dirty` | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 4 | `boat.rs:256` | `ChestBoat.destroy` / `ContainerHelper.dropContents` | Contents are dropped only when `doEntityDrops` is on and never on a creative break; vanilla drops the 27 slots regardless of the gamerule (only the boat item is gamerule-gated). No `.vendor` copy to confirm ordering | Unconditional `scatter_inventory`, keep the item drop gated | S, STILL OPEN: stale row, boat.rs already scatters the chest contents before the gamerule gate; needs a live check |
| 5 | `boat.rs:270` + `vehicle.rs:190` | `Boat.destroy` (hardcoded drop) | `kill_and_drop_self` also rolls `minecraft:entities/<type>` and `boat.rs` drops the item from code: a datapack table makes the boat drop twice | Skip the loot-table roll for boats (`is_chest_boat`/boat types) or drop only from one place | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 6 | `boat.rs:215` | `Entity.updateFluidInteraction` push | Fluid current push deliberately skipped for boats (client predicted); vanilla boats are pushed by flowing water/in water columns | Only if server-side divergence is observed: apply the push when no controlling passenger | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |

## Result

- #1 (canBeCollidedWith): already fixed; verified `is_collidable` returns `!is_removed()`.
- #2 (interact alive guard): already fixed; verified `AbstractBoat::interact` rejects a dead boat.
- #3 (bubble column): FIXED. `boat.rs` overrides `on_above_bubble_column`/`on_inside_bubble_column`
  and runs `tick_bubble_column` from `tick`. Upward contact arms a 60-tick countdown that sets
  y velocity to 0.6; downward sets current y minus 0.7 and ejects riders; inside-column contact
  uses the capped base push. Pure unit test added for the impulse helper.
- #4 (chest contents vs gamerule): already fixed; `scatter_chest_contents()` runs unconditionally
  before the `entity_drops` gate at `boat.rs`, and vanilla `RemovalReason::KILLED` and
  `DISCARDED` both have `shouldDestroy() = true`, so scattering on any removal is correct.
  No change needed.
- #5 (double boat drop): FIXED. `vehicle.rs::kill_and_drop_self` now skips the
  `minecraft:entities/<type>` loot roll for all boat types (`entity_type.boat.is_some()`),
  leaving the single hardcoded item drop in `boat.rs` as the only boat drop. Pure unit test added.
- #6 (fluid current push): LEFT OPEN. Applying the push needs changes to
  `base_tick_without_fluid_push` in `entity/mod.rs`, outside this unit, and the tracker only
  authorises it on observed server-side divergence; boat motion is client-predicted, so adding
  it unprompted risks fighting the client.


### Re-verification 2026-11-18 (CODER pass)

- #3 bubble column: verified present (`tick_bubble_column`, `on_above_bubble_column`,
  `on_inside_bubble_column` in `boat.rs`, called from `tick`). No change.
- #4 chest contents vs gamerule: verified `scatter_chest_contents()` runs before the
  `entity_drops` gate. No change.
- #5 double boat drop: verified `has_entity_loot_table()` returns false for all boat types
  so the loot roll is skipped; single hardcoded drop remains. No change.
- #6 fluid current push: LEFT OPEN. Needs `base_tick_without_fluid_push` in `entity/mod.rs`,
  outside this unit, and is only authorised on observed server-side divergence; boat motion
  is client-predicted. No change.
- Paddle handler (`net/java/play/paddle_boat.rs`): matches decompiled vanilla
  `ServerGamePacketListenerImpl.handlePaddleBoat` (line 2464) which gates on
  `player.getControlledVehicle() instanceof AbstractBoat`; `Entity.getControlledVehicle`
  (line 6661) requires the immediate vehicle whose controlling passenger is the sender.
  Pumpkin checks the immediate vehicle, that its controlling passenger is the sender
  (by entity id, unique), and downcasts to `BoatEntity`. `get_controlling_passenger()`
  returns the first player passenger, matching vanilla `AbstractBoat`. No parity gap, no
  pure-logic seam for a unit test in this handler. No change.


## Result (2026-11-18 re-review, second CODER pass)

Unit scope: only `net/java/play/paddle_boat.rs` is editable. No source change was needed.

- Paddle handler (`paddle_boat.rs`): verified line-by-line against decompiled vanilla
  `ServerGamePacketListenerImpl.handlePaddleBoat` (line 2464). Vanilla gates on
  `getControlledVehicle() instanceof AbstractBoat`, where `Entity.getControlledVehicle`
  (line 6661) = immediate vehicle whose `getControllingPassenger() == this`, and
  `AbstractBoat.getControllingPassenger` (line 845) = first passenger if it is a Player.
  Pumpkin checks the immediate vehicle, matches its controlling passenger by unique entity
  id, downcasts to `BoatEntity` (all boat/chest-boat types), then calls `set_paddle_state`,
  which is a plain metadata set in vanilla (line 418). Exact parity, no gap. No pure-logic
  seam worth a unit test in this network glue. No change.
- #3 bubble column: open row targets `boat.rs` (outside this unit); already fixed there
  (`tick_bubble_column` at line 429, called at line 613). No change.
- #4 chest contents vs gamerule: targets `boat.rs` (outside this unit); `scatter_chest_contents`
  at lines 272/489/730 runs unconditionally before the drops gate. No change.
- #5 double boat drop: targets `vehicle.rs` (outside this unit); `has_entity_loot_table`
  (line 27) returns false for boats, gating the roll at line 201. No change.
- #6 fluid current push: LEFT OPEN. Needs `base_tick_without_fluid_push` in `entity/mod.rs`,
  outside this unit, and is authorised only on observed server-side divergence; boat motion
  is client-predicted. No change.
