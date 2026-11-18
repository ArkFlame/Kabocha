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
| 3 | `boat.rs:208` | `AbstractBoat.tick` -> `tickBubbleColumn` | Upward bubble columns do not lift the boat, downward ones do not sink it | Read `BubbleColumnBlock` state under the boat and set vertical velocity, then flag `velocity_dirty` | M, FIXED / VERIFIED (bubble column impulse; AbstractBoat.java:165-184,370-407) |
| 4 | `boat.rs:256` | `ChestBoat.destroy` / `ContainerHelper.dropContents` | Contents are dropped only when `doEntityDrops` is on and never on a creative break; vanilla drops the 27 slots regardless of the gamerule (only the boat item is gamerule-gated). No `.vendor` copy to confirm ordering | Unconditional `scatter_inventory`, keep the item drop gated | S, VERIFIED (contents scatter ungated; AbstractChestBoat.java:71-77) |
| 5 | `boat.rs:270` + `vehicle.rs:190` | `Boat.destroy` (hardcoded drop) | `kill_and_drop_self` also rolls `minecraft:entities/<type>` and `boat.rs` drops the item from code: a datapack table makes the boat drop twice | Skip the loot-table roll for boats (`is_chest_boat`/boat types) or drop only from one place | S, FIXED / VERIFIED (single boat drop; AbstractBoat.java:862, AbstractChestBoat.java:65-68) |
| 6 | `boat.rs:215` | `Entity.updateFluidInteraction` push | Fluid current push deliberately skipped for boats (client predicted); vanilla boats are pushed by flowing water/in water columns | Only if server-side divergence is observed: apply the push when no controlling passenger | M, VERIFIED (fluid push addressed; client-predicted, no divergence) |

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


## Result (2026-11-18 third CODER pass)

Unit scope: only `net/java/play/paddle_boat.rs` is editable; no source change needed.

- Paddle handler (`paddle_boat.rs`): re-verified line-by-line against decompiled vanilla
  `ServerGamePacketListenerImpl.handlePaddleBoat` (line 2464). Vanilla gates on
  `player.getControlledVehicle() instanceof AbstractBoat`, where `Entity.getControlledVehicle`
  (line 6661) requires the immediate vehicle whose `getControllingPassenger() == this`, and
  `AbstractBoat.getControllingPassenger` (line 845) = first passenger if Player else super (null).
  Pumpkin checks the immediate vehicle (`get_vehicle`), that its controlling passenger is the
  sender by unique entity id, downcasts to `BoatEntity` (all boat/chest-boat types), then calls
  `set_paddle_state` -> `set_paddles`. Vanilla `setPaddleState` (line 418) is a plain metadata set.
  `get_controlling_passenger` (boat.rs:521) = first passenger that is a player else None, matching
  vanilla. Exact parity, no gap, no pure-logic seam for a unit test in this network glue. No change.
- #3 (bubble column): targets `boat.rs` (outside this unit); already fixed there
  (`tick_bubble_column`, `on_above_bubble_column`, `on_inside_bubble_column`, called from `tick`).
  No change.
- #4 (chest contents vs gamerule): targets `boat.rs` (outside this unit); `scatter_chest_contents`
  runs unconditionally before the `entity_drops` gate. No change.
- #5 (double boat drop): targets `vehicle.rs` (outside this unit); `kill_and_drop_self` gates the
  loot roll on `has_entity_loot_table` and boat types return false, so only the hardcoded
  `boat.rs` drop remains. No change.
- #6 (fluid current push): LEFT OPEN. Needs `base_tick_without_fluid_push` in `entity/mod.rs`,
  outside this unit, and is authorised only on observed server-side divergence; boat motion is
  client-predicted. No change.


## Result (2026-11-18 second-pass verification, expanded scope)

All four STILL OPEN rows verified against decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/`; no source change needed.

- #3 (bubble column, boat.rs): FIXED and present. `tick_bubble_column` (line 429) runs from
  `tick` (line 624); `on_above_bubble_column`/`on_inside_bubble_column` (lines 642/695) arm the
  60-tick launch/sink countdown. Vanilla `AbstractBoat.handleBubbleColumnEffect` (line 399) sets
  up -> (x, 0.6, z), down -> +(-0.7) on y gated on `canSimulateMovement()`, matching
  `bubble_column_vertical_velocity` (line 971) and the eject-riders branch (line 456). Pure unit
  test at line 1071. No change.
- #4 (chest contents vs gamerule, boat.rs): FIXED. `on_removed` scatters on any `should_destroy`
  removal (lines 488-500) and `hurt` calls `scatter_chest_contents()` before the `entity_drops`
  gate (lines 741-742); `claim_drops()` makes it run once. Vanilla `AbstractChestBoat.remove`
  scatters the 27 slots regardless of `doEntityDrops`, only the item is gamerule-gated. No change.
- #5 (double boat drop, vehicle.rs): FIXED. `has_entity_loot_table` returns false for all boat
  types (lines 27-28) and gates the `minecraft:entities/<type>` loot roll (line 201), leaving the
  single hardcoded item drop in `boat.rs` as the only boat drop. Unit test at line 335. No change.
- #6 (fluid current push): ADDRESSED in current source; the tracker note is stale. `tick`
  (boat.rs:594-598) now applies `base_tick` (with fluid push) to empty and mob-ridden boats and
  only skips the push (`base_tick_without_fluid_push`) for player-steered boats. That matches
  vanilla `Entity.baseTick`, which pushes every boat, while the skip is invisible because a
  steered boat's position is client-authoritative and its velocity is zeroed (lines 606-613).
  `is_client_controlled` (line 421) = a player controlling passenger, matching vanilla
  `AbstractBoat.getControllingPassenger` (line 845: first passenger only when it is a Player).
  The row is authorised only on observed server-side divergence, which is absent, so no change.
- Paddle handler (`paddle_boat.rs`): exact parity with vanilla
  `ServerGamePacketListenerImpl.handlePaddleBoat` (line 2464): gate on the immediate controlled
  vehicle being an `AbstractBoat`, then `setPaddleState`. Pumpkin checks the immediate vehicle,
  matches the controlling passenger by unique entity id, downcasts to `BoatEntity` (all boat and
  chest-boat types) and sets the plain metadata state. No pure-logic seam for a unit test.

No source edits made: every STILL OPEN row is already fixed/addressed in the current source and
`paddle_boat.rs` needs no change. No generated pumpkin-data fields or missing subsystem involved.

## Result (2026-11-18 fourth pass, source edits permitted)

Each STILL OPEN row re-verified against `.vendor/vanilla-decompiled/net/minecraft/` and the
current source. No source edit was required: the earlier fixes are present and correct.

- #3 (bubble column) FIXED. `tick_bubble_column` (boat.rs:429) runs from `tick` (boat.rs:581).
  `on_above_bubble_column` (boat.rs:647) arms the 60-tick launch countdown,
  `on_inside_bubble_column` (boat.rs:700) applies the capped sink push, and
  `bubble_column_vertical_velocity` (boat.rs:976) matches `AbstractBoat.handleBubbleColumnEffect`
  (AbstractBoat.java:399-407). Unit test `bubble_column_impulse_matches_vanilla_boat_effect`
  covers the caps, and the eject-riders branch is present.
- #4 (chest contents vs gamerule) FIXED. `on_removed` (boat.rs:488) scatters on every
  `should_destroy` removal reason, and `damage_with_context` (boat.rs:707) scatters before the
  `entity_drops` gate, with `claim_drops()` guaranteeing a single run. Matches
  `AbstractChestBoat.remove` (AbstractChestBoat.java:71-77), where only the boat item is
  gamerule-gated. Unit test `chest_contents_scatter_claim_is_single_use`.
- #5 (double boat drop) FIXED. `has_entity_loot_table` (vehicle.rs:27) returns
  `entity_type.boat.is_none()` and gates the loot roll in `kill_and_drop_self` (vehicle.rs:201),
  so boats keep only the hardcoded drop. Unit test at vehicle.rs:337-341.
- #6 (fluid current push) ADDRESSED, the row is stale. `tick` (boat.rs:599-603) applies
  `base_tick` (fluid push) to empty and mob-ridden boats and `base_tick_without_fluid_push` only
  to player-controlled boats, whose velocity is then zeroed (boat.rs:612-618). Vanilla
  `Entity.baseTick` pushes every boat, and the skip is unobservable because a steered boat's
  position is client-authoritative (`isLocalInstanceAuthoritative`). Authorised only on observed
  server-side divergence, absent.
- Paddle handler (`net/java/play/paddle_boat.rs`) parity confirmed with
  `ServerGamePacketListenerImpl.handlePaddleBoat` (lines 2464-2469) through
  `Entity.getControlledVehicle` (Entity.java:6661) and `AbstractBoat.getControllingPassenger`
  (AbstractBoat.java:845). Network glue, no pure-logic seam for a unit test.

No generated `pumpkin-data` field or missing subsystem was involved. No source edits: zero new
fixes this pass, zero rows left open.

## Result (2026-11-18 fifth pass, terminal verification)

All four previously STILL OPEN rows re-verified against decompiled vanilla at
.vendor/vanilla-decompiled/net/minecraft/ and the current source. Every row now
carries a terminal status; none remain STILL OPEN. No source edit was required:
each fix is present and correct in the current tree, and no generated
pumpkin-data field or missing subsystem is involved.

- #3 (bubble column) FIXED / VERIFIED. boat.rs tick_bubble_column (line 429)
  runs from tick (line 629); on_above_bubble_column (line 647) arms the 60-tick
  countdown and ejects riders on the sink; bubble_column_vertical_velocity
  (line 976) sets updraft y=0.6 / whirlpool y-0.7. Matches vanilla
  AbstractBoat.onAboveBubbleColumn (AbstractBoat.java:165-184), the tick
  countdown (AbstractBoat.java:370-385) and handleBubbleColumnEffect
  (AbstractBoat.java:399-407). Unit test
  bubble_column_impulse_matches_vanilla_boat_effect at boat.rs:1076.
- #4 (chest contents vs gamerule) VERIFIED. boat.rs on_removed (line 488) scatters
  on every should_destroy removal and damage_with_context (line 707) scatters
  before the entity_drops gate via scatter_chest_contents (line 272), with
  claim_drops() making it single-use; only the boat item is gamerule-gated.
  Matches vanilla AbstractChestBoat.remove (AbstractChestBoat.java:71-77), which
  calls Containers.dropContents regardless of doEntityDrops.
- #5 (double boat drop) FIXED / VERIFIED. vehicle.rs has_entity_loot_table
  (line 27) returns entity_type.boat.is_none(), gating the loot roll in
  kill_and_drop_self (line 201), so boats keep only the hardcoded item drop in
  boat.rs damage_with_context. Vanilla drops boats from code via
  AbstractBoat.getDropItem (AbstractBoat.java:862) and AbstractChestBoat.destroy
  (AbstractChestBoat.java:65-68), with no minecraft:entities/type loot roll.
  Unit test at vehicle.rs:337.
- #6 (fluid current push) VERIFIED (authorised-on-divergence row; no divergence
  observed). boat.rs tick (lines 599-603) applies base_tick (with fluid push) to
  empty and mob-ridden boats and base_tick_without_fluid_push only to
  player-steered boats, whose velocity is then zeroed (lines 612-618). Matches
  vanilla Entity.baseTick pushing all boats, with the server push unobservable
  for a client-authoritative boat. No observed server-side divergence, no change.
- Paddle handler (net/java/play/paddle_boat.rs) exact parity with vanilla
  ServerGamePacketListenerImpl.handlePaddleBoat
  (ServerGamePacketListenerImpl.java:2464-2469): gate on the immediate controlled
  vehicle being an AbstractBoat, then setPaddleState. Pumpkin checks the
  immediate vehicle, matches its controlling passenger by unique entity id,
  downcasts to BoatEntity (all boat and chest-boat types) and sets the plain
  metadata state. Network glue, no pure-logic seam for a unit test.

Status: PASS fixed=0 open=0 files=docs/tracker/tentities-bamboo-chest-raft.md
