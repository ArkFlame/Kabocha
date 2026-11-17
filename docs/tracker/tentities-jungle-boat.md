# entities / Jungle Boat (RE-REVIEW)

Reviewed: `crates/pumpkin-core/src/entity/vehicle/boat.rs`, `boat/physics.rs`, `boat/passengers.rs`, `boat/dismount.rs`, `boat/block_effects.rs`, `entity/vehicle/vehicle.rs`, `item/items/boat.rs`.

Jungle boat is a plain wood variant: `EntityType::JUNGLE_BOAT` / `JUNGLE_CHEST_BOAT` share the whole `BoatEntity` path (`is_chest_boat` at boat.rs:43-55), the item map keeps `JUNGLE_BOAT`/`JUNGLE_CHEST_BOAT` separate from oak (item/items/boat.rs:29-30), and seat geometry comes from generated `BoatSeats` (`ride_height` 0.1875, `single_passenger_x_offset` 0.0/0.15).

No decompiled reference is available in this worktree (`.vendor/` absent), so `AbstractBoat`/`Boat`/`ChestBoat` could not be diffed line by line. The status, float, move, dismount and collision paths all exist and `block_effects.rs` already carries recorded-vanilla traversal tests, so no vanilla gap could be shown from source alone.

Fixed: `boat/passengers.rs` recomputed the seat values the generated data already holds (chest 0.15 vs 0.0 offset, `height/3` with a bamboo `0.8888889` special case) while `boat.rs::passenger_attachment_point` read `EntityType.boat`. Both paths agreed numerically, so the literals were a drift hazard rather than a live bug. Both now read `BoatSeats`.

## Remaining gaps

| file:line | vanilla | suggested fix | size |
|---|---|---|---|
| boat/passengers.rs:12-16 | `AbstractBoat.positionRider` places every rider through the attachment point; here `rider_position` returns `None` for non-players, so a mob passenger in a jungle boat is only moved by the generic `EntityBase::passenger_attachment_point` path | check a mob rider's position on the client after boarding and route mobs through the same attachment point | M (verify) |
| boat.rs:365-366 | `Boat.interact` only gates boarding on `outOfControlTicks < 60.0F` and secondary use | confirm the extra `!is_eye_in_water()` gate against decompiled `Boat.interact`; it can refuse boarding a boat in shallow water | S (verify) |
| boat.rs:144-167 | vanilla `ejectPassengers` when `outOfControlTicks >= 60` runs alongside the normal entity tick with no extra event | confirm reverse-order `remove_passenger` matches expected dismount results for a full jungle boat | S (verify) |
| boat.rs:327-357 | `AbstractBoat.destroy` drops chest contents and the boat item under `entity_drops` | verify scatter-then-drop ordering and offsets against decompiled `destroy`; no test covers the break path | S (verify) |
| boat.rs:94-96 | chest boats seat one passenger, boats two | none needed beyond a test: no automated test pins the seat count or the offsets | S |
| boat.rs:43-55 | chest boat set must match the generated chest-boat entity types | grep generated data for any missing chest variant (new wood types) before the next port | S (verify) |

Test gap: nothing exercises `passenger_attachment_point` for the 0.2/-0.6 two-seat offsets or the `+0.2` animal shift (boat.rs:191-223); a unit test on `BoatEntity` seat selection would be the cheapest gate.
