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

## h6 status
- Rider yaw clamp (AbstractBoat.clampRotation, +/-105 deg, body and head sync): FIXED bd8b9bf97 in entity/vehicle/boat/passengers.rs. Unit test: rider_yaw_is_clamped_to_105_degrees_around_boat_yaw. Delta-rotation accumulation not added: UNVERIFIED, STILL OPEN.

## Result

Re-verified against `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java` on the main checkout. Every gap in this unit is ALREADY FIXED; no source edits were needed this pass.

| # | Row | Status | Evidence |
|---|-----|--------|----------|
| 1 | canAddPassenger / eye-in-water boarding gate (S) | FIXED | boat.rs:621-622 checks ticks_underwater < TIME_TO_EJECT && !is_eye_in_water(...); is_eye_in_water at boat.rs:100-108 samples the eye block with fluid height. Matches vanilla AbstractBoat.canAddPassenger (AbstractBoat.java:836-837). Pushed-mob path also gated at entity/mod.rs:1320. |
| 2 | positionRider + clampRotation + deltaRotation (M) | FIXED | boat.rs:666-676 (on_rider_positioned) applies rider_yaw_with_delta then stores body/head yaw; helpers clamp_rider_yaw / rider_yaw_with_delta at boat.rs:710-723; per-tick delta derived at boat.rs:433-439 (last_yaw/yaw_delta). Pure unit tests rider_yaw_clamps_to_boat_heading_window and rider_yaw_follows_boat_yaw_delta_then_clamps at boat.rs:734-752. |
| 3 | Bubble-time metadata (M) | FIXED | ID_BUBBLE_TIME written on arming (boat.rs:490-493) and decremented/written each tick (boat.rs:319-340); impulse + BOAT_SINK eject at count zero. Test bubble_column_impulse_matches_vanilla_boat_effect at boat.rs:755-758. |
| 4 | Leashable (M) | FIXED | Shared entity leash path: boat tick() calls entity.tick_leash(caller) (boat.rs:450-451) and support_quad_leash returns true (boat.rs:682-685); leashed_to / tick_leash exist in entity/mod.rs:1829,5708. |
| 5 | hasEnoughSpaceFor width gate (S) | FIXED | entity/mod.rs:1330-1334 checks other.width < self.width plus CANNOT_BE_PUSHED_ONTO_BOATS tag before boarding. Matches AbstractBoat.java:691-693. |
| 6 | deltaRotation decay / server-side prediction (L) | NO ACTION (by design) | Client-authoritative engine difference; the doc already recommends no action. |

The h6 status note is stale: delta-rotation accumulation and the +/-105 rider yaw clamp are both implemented in boat.rs (not passengers.rs) under the names rider_yaw_with_delta / clamp_rider_yaw, with unit tests. Marking them resolved.

No .rs files touched, so rustfmt was not run.

## Result

Re-re-verified against decompiled vanilla AbstractBoat.java in .vendor/vanilla-decompiled on the main checkout. All rows are ALREADY FIXED in the current source; no source edits were needed. This corrects the stale line numbers in the earlier Result block.

| # | Row | Status | Evidence (current lines) |
|---|-----|--------|--------------------------|
| 1 | canAddPassenger / eye-in-water boarding gate (S) | FIXED | boat.rs is_eye_in_water at 123-131; gate passed to boat_interact_outcome at 815-816; pushed-mob path entity/mod.rs:1345. Matches AbstractBoat.java:836-837. |
| 2 | positionRider + clampRotation + deltaRotation (M) | FIXED | delta derived in tick boat.rs:571-577 (wrap_degrees of consecutive yaw); on_rider_positioned boat.rs:846-889 applies rider_body_yaw + yaw_delta, clamp_rider_yaw (105 deg) and mob head clamp; on_passenger_turned 891-939; helpers 972-1009. Matches AbstractBoat.java:696-705,740-765. |
| 3 | Bubble-time metadata (M) | FIXED | on_above_bubble_column boat.rs:631-682 arms BUBBLE_COLUMN_DELAY=60 and writes ID_BUBBLE_TIME; tick_bubble_column 429-454 decrements/writes and impulses at zero; helper 966-968. Matches AbstractBoat.java:165-170,351-387,399-408. |
| 4 | Leashable (M) | FIXED | tick calls entity.tick_leash boat.rs:593; support_quad_leash true at 946-948; shared leash path in entity/mod.rs. |
| 5 | hasEnoughSpaceFor width gate (S) | FIXED | entity/mod.rs:1353-1359 checks other.width < self.width plus CANNOT_BE_PUSHED_ONTO_BOATS before add_passenger. Matches AbstractBoat.java:691-693. |
| 6 | deltaRotation decay / server prediction (L) | NO ACTION (by design) | Client-authoritative engine difference; doc recommends no action. |

The h6 status note is stale: delta-rotation accumulation is implemented at boat.rs:571-577 and applied to riders at 873-888 (mob and non-mob), with pure unit tests rider_yaw_clamps_to_boat_heading_window / rider_yaw_follows_boat_yaw_delta_then_clamps at boat.rs:1027-1042. Marking it resolved.

No .rs files touched, so rustfmt was not run.
