# entities / Pale Oak Boat — re-review (2026-11-17)

Source: `crates/pumpkin-core/src/item/items/boat.rs`, `crates/pumpkin-core/src/entity/vehicle/boat.rs`
and submodules `physics.rs`, `passengers.rs`, `dismount.rs`, `block_effects.rs`.
Prior review: `docs/tracker/tentities-pale-oak-boat.md`. Since then the source moved with
0c22e1aee (boat placement entity hitbox inflate 0.3), 97ba2480d (chest contents on creative break),
cc2e20b55 (seat offsets from generated entity data), baf497b61, 0fe61a1fb, add8c3c16.
No decompiled sources in `.vendor/`, so each row names the vanilla member it must match
(`AbstractBoat` / `BoatEntity` / `AbstractChestBoat` / `BoatItem`).

Pale oak is not special-cased: it rides the shared boat entity, shared `BoatItem`, and the
generated `EntityType.boat` seat data (`entity_type.rs` PALE_OAK_BOAT = 95). The audit therefore
covers (a) every registration a pale-oak boat needs and (b) the shared behaviour the source changed.

## Wiring re-verified (no gaps)

| Where | Check |
|---|---|
| item/items/boat.rs:39-40, 89-90 | `PALE_OAK_BOAT`/`PALE_OAK_CHEST_BOAT` map to their entity types and are registered |
| item/items/boat.rs:53-58 | `entity_to_item` resolves pale oak (pick block, boat break drop) |
| entity/type.rs:344 | both pale oak types construct `BoatEntity` |
| entity/vehicle/boat.rs:41-53 | `is_chest_boat` includes `PALE_OAK_CHEST_BOAT` (27 slots, 1 passenger) |
| entity/vehicle/boat.rs:92-94 | non-chest pale oak keeps 2 passengers |
| entity/vehicle/boat/passengers.rs:44-83 | seat offsets/ride height read from generated `EntityType.boat` data (matches vanilla `AbstractBoat.getPassengerAttachmentPoint`/getRideHeight) |
| item/items/boat.rs:150-157 | `entity.getBoundingBox().inflate(0.3).contains(eye)` matches vanilla `BoatItem.use` CAN_HIT loop |
| item/items/boat.rs:164-182 | placement space + occupying-entity checks match vanilla `noCollision` + `isSpaceEmpty` |
| entity/vehicle/boat.rs:326-354 | chest contents scatter on any destroying removal, item drop creative-gated (`ChestBoat.remove`) |

## Prior gaps: resolved or disproved

1. (old #1) chest-boat open result discarded — fixed: entity/vehicle/boat.rs:372-374 returns `open_custom_inventory_screen`.
2. (old #2) `!is_eye_in_water()` mount gate — **not a gap**: that is vanilla `BoatEntity.canAddPassenger == getPassengers().size() < 2 && !isUnderWater()`, and `Entity.isUnderWater()` is the boat's eye-in-water test. Keep it.
3. (old #4) `friction / count` NaN — **not a gap**: vanilla `getGroundFriction` also returns `f / j` with `j == 0`, and `get_status` tests `friction > 0.0`. Rust `NaN > 0.0` is false, same outcome. Do not guard.
4. (old #5) stale `last_yd` on the client-controlled path — fixed in this commit: `move_from_client` stored `entity.velocity.load().y`, which `tick` has just zeroed for client-controlled boats, so `get_water_level_above` always used 0 instead of the accepted fall distance.

## Remaining gaps

| # | Where | Vanilla member | Gap | Suggested fix | Size |
|---|-------|----------------|-----|---------------|------|
| 1 | entity/vehicle/boat.rs:359-394 `interact` | `AbstractChestBoat.interact` | When the container cannot open (spectator filtered earlier, out-of-range or plugin-denied open) the method returns `false` and never falls through to mounting; vanilla runs the ride attempt and only then offers the container, so a failed open still consumes the click as a ride attempt when a seat is free. Needs the decompiled `AbstractChestBoat.interact`/openMenu ordering to confirm which branch wins | If a failed open should fall through, retry the mount path when `open_custom_inventory_screen` is `false` and a seat is free | S |
| 2 | entity/vehicle/boat.rs:319-321 `damage_with_context` | `VehicleEntity.damage` / `ChestBoat.remove` | Break detection is predicted before damage (`will_break`) using the same `damage + amount*10 > 40` rule as vehicle.rs:243; it only stays correct because the two thresholds are kept in sync by hand, and the `is_removed()` guard hides any drift | Have `VehicleEntity::damage_with_context` return the destroy decision, or drop the prediction and test `is_removed()` after the call | S |
| 3 | entity/vehicle/boat.rs:268-293 `tick` | `BoatEntity.tick` | No paddle audio (`SoundEvents.BOAT_PADDLE_WATER` while paddling in water, `BOAT_PADDLE_LAND` on land) and no underwater bubble particles when `status` is `UnderWater`. Only server-sent effect packets, no gameplay impact | Emit the level event/sound and particle packets from the boat tick for both paddle states | M |
| 4 | entity/vehicle/boat.rs:183-215 vs boat/passengers.rs:34-65 | `AbstractBoat.getPassengerAttachmentPoint` | Two independent rider-position implementations: `EntityBase::passenger_attachment_point` adds `ANIMAL_SEAT_SHIFT` for animals, `passengers.rs::rider_position` (players only) does not, and each recomputes the yaw rotation differently | Share one helper that returns `(offset, ride_height)` per passenger and let both call it | S |

## Not gaps (checked)

Pale oak tag membership (`minecraft:boat`, `c_boats`) and `boat_one_cm` statistics, chest boat
inventory size 27, `TIME_TO_EJECT` 60-tick underwater ejection, custom-name preservation on
placement and break drop, wobble metadata pairing, two-passenger seat indexes 0.2/-0.6, dismount
spot search over Standing/Crouching/Swimming, `blocks_building = true`, dispenser spawning,
lily-pad entity collision and the 100-movement/16-step vanilla travel cut-offs in `block_effects.rs`.
