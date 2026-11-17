# entities / Cherry Boat - audit (2026-11-17, re-review)

Tracker row: `entities | Cherry Boat | RE-REVIEW-UNTRACKED`.
The shared `BoatEntity` source changed since the 2026-11-16 review: physics, block effects and
dismount handling now live in `crates/pumpkin-core/src/entity/vehicle/boat/{physics,block_effects,dismount,passengers}.rs`.
Cherry boats use that shared code plus the wiring below.

## Wiring (complete)

| Concern | Rust |
|---|---|
| item -> entity | `crates/pumpkin-core/src/item/items/boat.rs:37`, id list `:87` |
| entity -> item | `item/items/boat.rs:53` (`entity_to_item`, `Boat.getPickResult`) |
| entity construction | `entity/type.rs:350-351` (cherry), `:352-353` (poplar) |
| dimensions / seats | generated `pumpkin-data/entity_type.rs:2136`, `boat` seats used in `passengers.rs` |
| chest inventory | `boat.rs:41 is_chest_boat`, `:37` 27 slots, `:227-237` NBT |
| boat_one_cm stat | `entity/player.rs:4376`, `:4386` (chest) |
| buoyancy / status / friction | `boat/physics.rs:215-273 float_boat`, reset/eject `boat.rs:136-159` |
| block collision effects | `boat/block_effects.rs:49 apply_effects_from_blocks` |
| dismount search | `boat/dismount.rs:18-75` |

Resolved since the last review: server-side `floatBoat` (water gravity/buoyancy, land friction,
`Boat.Status` transitions) and chest-content scatter on the damage destruction path (`boat.rs:307-357`).

## Fixed in this pass

- `boat.rs:92-94` - chest boats were capped at 1 passenger (`get_max_passengers` returned 1).
  Vanilla `AbstractBoat.canAddPassenger` returns `passengers.size() < 2` for every boat; `ChestBoat.interact`
  only opens the container when sneaking or when no seat is free. Fixed: constant `MAX_PASSENGERS = 2`
  (`boat.rs:39`, used at `boat.rs:363`).

## Remaining gaps (shared `BoatEntity`, visible on a cherry boat)

1. `boat/passengers.rs:12-16, 34-65` - rider rotation is never clamped to the boat. Vanilla
   `Boat.positionRider` does `passenger.setYHeadRot(passenger.getYHeadRot() + this.deltaRotation)` and then
   `clampRotation(passenger)` (body yaw = boat yaw, head yaw wrapped to +-105 deg relative to the boat).
   Pumpkin has no `delta_rotation` or `clamp_rotation` (grep: no hits under `crates/pumpkin-core/src`), so a
   cherry-boat rider can look backwards and the rider's head does not follow the hull while it turns.
   Fix: track `deltaRotation` in `BoatEntity` and add `EntityBase::clamp_rotation` called from both
   `position_rider` and `passenger_attachment_point`. Size M.

2. `boat.rs:359-362` - `interact` refuses mounting while `ticks_underwater >= TIME_TO_EJECT` or the boat's
   eye is in water. Vanilla `Boat.interact` only rejects secondary use and then relies on
   `startRiding` (`canAddPassenger`, player not already riding, boarding cooldown); the underwater eject
   lives in `Boat.tick` (`outOfControlTicks >= 60`). A submerged cherry boat accepts riders in vanilla.
   Fix: drop `ticks_underwater < TIME_TO_EJECT` and `!self.is_eye_in_water()` from `can_add_passenger`
   (then delete the now-unused `physics.rs:45-72 is_eye_in_water`). Size S.

3. `boat.rs:136-142 tick_underwater` - ejection counter increments for `UnderWater` and
   `UnderFlowingWater` (`BoatStatus::is_underwater`, `physics.rs:24-27`). Vanilla `AbstractBoat.floatBoat`
   only increments `outOfControlTicks` in the `UNDER_WATER` case; flowing water does not count. Fix:
   increment only for `BoatStatus::UnderWater`. Size S.

4. `boat.rs:307-357` + `entity/vehicle/vehicle.rs:190-214` - chest contents scatter only inside the boat
   damage path. `Entity::remove` (`entity/mod.rs:4281`) carries no `RemovalReason`, so `/kill`, plugin-driven
   removal or any other destroy path calls `kill_and_drop_self`, which drops only the loot table and never
   `scatter_inventory`. Vanilla `ChestBoat.remove(RemovalReason)` scatters the 27 slots whenever
   `reason.shouldDestroy()`. Fix: add a destroy hook (or pass `RemovalReason`) so `ChestBoat` scatters once
   per destroying removal. Size M.
