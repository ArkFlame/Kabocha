# Audit: entities / Bamboo Raft (RE-REVIEW-UNTRACKED)

Source re-reviewed on branch `kab/tentities-bamboo-raft-302792` after the boat rework
(`0d03a0263 fix(entity): align boat boarding and control with vanilla`):
- `crates/pumpkin-core/src/entity/vehicle/boat.rs` (dispatcher, metadata, damage/drop, interact)
- `crates/pumpkin-core/src/entity/vehicle/boat/{physics,passengers,dismount,block_effects}.rs`
- `crates/pumpkin-core/src/item/items/boat.rs`
- `crates/pumpkin-core/src/entity/type.rs`, `crates/pumpkin-core/src/entity/player.rs`

Vanilla reference (decompiled, verified this pass):
`/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft/world/entity/vehicle/boat/`
(`AbstractBoat.java`, `AbstractChestBoat.java`, `Raft.java`, `ChestRaft.java`). The package
was renamed from `entity.vehicle` to `entity.vehicle.boat`, and rafts are `Raft`/`ChestRaft`.

## Verdict

Raft has no separate Rust entity and does not need one. Vanilla `Raft`/`ChestRaft` extend
`AbstractBoat`/`AbstractChestBoat` and override exactly one thing:

- `Raft.rideHeight(dimensions) = dimensions.height() * 0.8888889` and the same on
  `ChestRaft`. Pumpkin reads the seat height from generated data
  (`entity_type.rs:908,965`, `ride_height: 0.5` for `bamboo_raft`/`bamboo_chest_raft`,
  dimensions height 0.5625), so the override is already reproduced without code.
- Chest raft single-rider offset 0.15 (`AbstractChestBoat.getSinglePassengerXOffset`) is
  also in generated data (`entity_type.rs:965`).

Everything else is shared `AbstractBoat` behaviour and is wired per raft:

- placement: `Item::BAMBOO_RAFT`/`BAMBOO_CHEST_RAFT` map to the matching entity types
  (`item/items/boat.rs:43-44`, ids `:93-94`).
- construction: both raft types build `BoatEntity` (`entity/type.rs:350-351,357`).
- chest inventory: 27 slots (`boat.rs:39,86-87`), title key
  `entity.minecraft.bamboo_chest_raft` with the Bedrock chest-raft fallback
  (`boat.rs:126-136`), spectator gate (`boat.rs:103-105`), `CONTAINER_OPEN` on open,
  matches `AbstractChestBoat.openCustomInventoryScreen`.
- drop: `Item::from_registry_key` on the entity registry name (`boat.rs:370-378`) equals
  `Raft.getDropItem()`; chest contents scatter on destroy (`boat.rs:359-369`).
- statistics: `entity/player.rs:4376,4387` cover `BAMBOO_RAFT`/`BAMBOO_CHEST_RAFT`.

## Fixed in this pass

**Chest rafts/chest boats accepted two passengers.** Vanilla
`AbstractChestBoat.getMaxPassengers() = 1`; `BoatEntity::interact` used a flat
`MAX_PASSENGERS = 2` for every variant, so a second player could board a
`bamboo_chest_raft` (and the pushed-mob path in `entity/mod.rs` disagreed with it).
Now `boat::max_passengers(&EntityType)` (`boat.rs:58-68`) returns 1 for chest
boats/rafts and is used by both boarding paths (`boat.rs:398`, `entity/mod.rs:1248-1249`);
`boat.rs:451-469` asserts it for every boat entity type.

## Remaining gaps (shared `AbstractBoat`, not raft-specific)

| # | file:line | Vanilla class | Gap | Suggested fix | Size |
|---|---|---|---|---|---|
| 1 | `boat.rs:386-405` | `AbstractBoat.canAddPassenger` | Boarding only checks `ticks_underwater < 60` (out-of-control ticks). Vanilla also refuses when `isEyeInFluid(WATER)`, so a player can board a raft whose eye is under water while it has been submerged for less than 60 ticks. | Add an `isEyeInFluid(WATER)`-equivalent check (fluid state at the boat eye block) to `can_add` in `interact`. | S |
| 2 | `boat/passengers.rs:12-42` | `AbstractBoat.positionRider` + `clampRotation` + `deltaRotation` | Rider position is set but vanilla also applies `deltaRotation` to yaw/yHeadRot for passengers not in `CAN_TURN_IN_BOATS` and clamps the rider's body rotation to the boat heading (`calculatePassengerBodyYRot`). | Port the rotation part of `positionRider`: add `delta_rotation`, the `CAN_TURN_IN_BOATS` tag check and `clamp_rotation`; apply in `position_rider`/`move_from_client`. | M |
| 3 | `boat.rs:154-161` | `AbstractBoat.DATA_ID_BUBBLE_TIME` / `tickBubbleColumn` | Bubble-time metadata (`tracked_data::bamboo_raft::ID_BUBBLE_TIME`) is never written, so the client shows no bubble effect while a raft is dragged down a bubble column. | Track submerged ticks and `set_bubble_time` on changes, plus the bubble-column direction handling. | M |
| 4 | `entity/vehicle/vehicle.rs` (none) | `AbstractBoat implements Leashable` | No leash support anywhere under `entity/vehicle/`, so a raft cannot be tied to a fence. | Implement the leash path on `VehicleEntity` (shared with minecarts), not a raft special case. | M |
| 5 | `entity/mod.rs:1254-1275` | `AbstractBoat.hasEnoughSpaceFor` | Pushed mobs board a raft regardless of width; vanilla requires `entity.getBbWidth() < boat.getBbWidth()`. | Add the width comparison before `add_passenger` in the boat pickup loop. | S |
| 6 | `boat.rs:297-322` | `AbstractBoat.tick` | Vanilla boats decay `deltaRotation` and reapply `outOfControlTicks`; Pumpkin keeps velocity for client-controlled boats (documented engine-wide client-authority difference), so server-side raft prediction is absent. | No action unless server-side raft prediction is wanted. | L |

Notes that are **not** gaps (checked against the decompile this pass):

- Chest raft/chest boat loot tables are vanilla-supported in this version
  (`AbstractChestBoat` keeps `lootTable`/`unpackChestVehicleLootTable`), so the earlier
  "skip loot table for chest boats" suggestion in `tentities-cherry-chest-boat.md` is
  wrong for the target version.
- `CONTAINER_OPEN` on a chest raft is emitted by vanilla too, so the emission is correct.
- Raft translations, item components, recipe and tags exist in generated data; no
  raft-specific paddle sound exists (vanilla `Raft` overrides none).
