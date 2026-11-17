# entities / Jungle Boat - re-audit (2026-11-17)

Tracker row: `entities | Jungle Boat | RE-REVIEW-UNTRACKED`.
Source re-reviewed: `crates/pumpkin-core/src/entity/vehicle/boat.rs`, `boat/physics.rs`, `boat/passengers.rs`, `boat/dismount.rs`, `boat/block_effects.rs`, `crates/pumpkin-core/src/entity/vehicle/vehicle.rs`, `crates/pumpkin-core/src/item/items/boat.rs`, `crates/pumpkin-core/src/entity/type.rs`, `crates/pumpkin-core/src/entity/player.rs`.
Vanilla: `net.minecraft.world.entity.vehicle.AbstractBoat` / `Boat` / `BoatItem` (26.3). No decompiled source is present in this worktree, so parity is against the vanilla class semantics only.

No code change was made in this pass: every jungle-specific wire is present and the previously reported live gaps (missing `floatBoat`, missing block-effect sweep) are now implemented. The remaining items are shared-boat behaviours that a jungle boat also shows, plus one local drift.

## Verified present (vanilla reference)

| Concern | Rust | Vanilla |
|---|---|---|
| item -> entity | `item/items/boat.rs:29` (`JUNGLE_BOAT`), `:79` id list | `BoatItem.getType()` |
| entity -> item (pick/drop) | `item/items/boat.rs:53-58` via resource name | `AbstractBoat.getPickResult()` / `getDropItem()` |
| entity construction | `entity/type.rs:338-339,355` | `EntityType.JUNGLE_BOAT.create()` |
| dimensions / eye height | generated `entity_type.rs` `[1.375, 0.5625]` | `EntityType` boat dimensions |
| seat offsets / ride height | `boat.rs:183-215`, `passengers.rs:67-83` from generated `BoatSeats`; two seats `0.2` / `-0.6` plus `0.2` for animals (`boat.rs:34-36,197-208`) | `AbstractBoat.positionRider`, `getSinglePassengerXOffset` |
| boat_one_cm statistic | `player.rs:4372`, `:4383` (chest variant) | `ServerPlayer` vehicle statistic |
| status / float / drag | `physics.rs:74-142` (`get_status`, `check_in_water`), `:215-273` (`float_boat` with buoyancy, underwater 0.45 friction, land friction, surface jump) | `AbstractBoat.getStatus`, `floatBoat` |
| movement authority | `boat.rs:280-292` leaves a ridden boat to the client, `physics.rs:297-323` validates `move_from_client` and rewinds on collision | client-authoritative (documented Pumpkin difference) |
| underwater ejection | `boat.rs:38,136-159` 60-tick count and reverse-order removal | `outOfControlTicks >= 60` -> `ejectPassengers` |
| paddles | `boat.rs:119-130`, cleared on server tick `:283-287` | `setPaddleState` + `DATA_ID_PADDLE_*` |
| damage / wobble / break | `boat.rs:307-357`, `vehicle.rs:167-190,216-261` (damage > 40, creative instabuild, `entity_drops`, chest spill, item drop) | `AbstractBoat.hurt`, `destroy` |
| interact gating | `boat.rs:359-394` (secondary use / no free seat opens the chest, `ticks_underwater < 60`, `!is_eye_in_water`, capacity, sneak) | `Boat.interact` out-of-control gate, `canAddPassenger`, `ChestBoat` open path |
| dismount | `dismount.rs:18-113` (water fallback, collision escape vector, standing/crouching/swimming poses) | `AbstractBoat.getDismountLocationForPassenger` |
| block effects sweep | `block_effects.rs:49-103` plus recorded-vanilla traversal tests `:389-494` | `Entity.checkInsideBlocks` / `addMovementThisTick` |

## Remaining gaps (shared `BoatEntity`, visible on a jungle boat)

| file:line | vanilla class / behaviour | suggested fix | size |
|---|---|---|---|
| `boat.rs:182-215`, `passengers.rs:34-65` | `AbstractBoat.positionRider` does `passenger.setYHeadRot(passenger.getYHeadRot() + this.deltaRotation)` then `Entity.clampRotation(passenger)` (body yaw follows boat, yaw clamped to +-105 deg). There is no `delta_rotation` or `clamp_rotation` anywhere in `crates/pumpkin-core/src/entity/vehicle` (grep: no hits), so a rider can look straight backwards and the rider does not follow the boat while turning. | Track per-tick yaw delta in `BoatEntity`, store it, and clamp the first passenger's yaw/head yaw from the attachment path in `passengers.rs::rider_position`. | M |
| `boat.rs:119-130`, `physics.rs:215-273` | `AbstractBoat` publishes `DATA_ID_BUBBLE_TIME` (generated id 13) while `UNDER_WATER`, driving the client bubble particles; Pumpkin never sends `tracked_data::boat::ID_BUBBLE_TIME` (grep: no hits in `crates/pumpkin-core/src`). | Add a `bubble_time` cell, set 60 on entering `UNDER_WATER`, decrement each tick and on `!isUnderwater` reset to 0, publish as metadata together with the paddle update. | M |
| `passengers.rs:53-57` | `AbstractBoat.positionRider` adds `0.2` to the seat offset for an `Animal` in the two-seat branch; `boat.rs:202-208` applies it but `passengers.rs` does not, so the two rider-positioning paths disagree for an animal in a two-seat boat. Currently only players reach `rider_position`, so this is latent, not live. | Extract one seat-offset helper (index, passenger kind, seat count) used by both `passenger_attachment_point` and `rider_position`. | S |
| `boat.rs:341-352` | `AbstractBoat.destroy` drops `getDropItem()` without copying the entity custom name; Pumpkin sets `CustomNameImpl` on the dropped boat item. Needs a decompiled check (shared with the chest-boat row). | If confirmed, drop the custom-name copy from the item drop. | S (verify) |
| `boat.rs:92-94` | `AbstractBoat.canAddPassenger` allows two passengers for every boat type; Pumpkin caps chest boats at one before the chest-open branch. Plain jungle boat is unaffected (2). Cross-reference: jungle-chest-boat row. | Confirm chest-boat capacity against decompiled `AbstractChestBoat`, then align `get_max_passengers`. | S (verify) |

## Test gap

Nothing exercises `BoatEntity::passenger_attachment_point` for the two-seat `0.2` / `-0.6` offsets or the `0.2` animal shift (`boat.rs:183-215`), and nothing pins that `passengers.rs::rider_position` and the generic attachment path agree. A unit test on seat-offset selection is the cheapest gate and would have caught the drift above. Size S.
