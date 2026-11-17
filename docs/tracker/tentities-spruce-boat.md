# entities / Spruce Boat
Reviewed: `crates/pumpkin-core/src/entity/vehicle/boat.rs`, `vehicle/boat/{physics,block_effects,dismount,passengers}.rs`, `vehicle/vehicle.rs`, `net/java/play/paddle_boat.rs` and `item/items/boat.rs` against vanilla `net.minecraft.world.entity.vehicle.AbstractBoat` / `VehicleEntity` / `BoatItem`. `SPRUCE_BOAT` and `SPRUCE_CHEST_BOAT` share this generic path; the only per-variant data is `BoatSeats { ride_height: 0.1875, single_passenger_x_offset: 0.0 }` in `crates/pumpkin-data/src/generated/entity_type.rs:10910`, which matches vanilla `dimensions.height() / 3.0`.
Verdict: the port is structurally complete and unusually faithful (status machine, buoyancy/friction, client-authoritative movement, block-effect sweep with 16-step/100-movement cutoffs, dismount escape vector, chest-boat inventory, wobble metadata). One small placement gap was fixed; two open items remain.

## Fixed in this audit
- `crates/pumpkin-core/src/item/items/boat.rs:143`: vanilla `BoatItem.use` rejects placement when `entity.getBoundingBox().inflate(0.3).contains(player.getEyePosition())`; Rust tested the raw bounding box against a degenerate eye box, so a boat could be placed with another entity within 0.3 blocks of the player's eye. Now uses `expand_all(0.3).contains(player_eye_pos)` (`BoundingBox::contains` already mirrors vanilla `AABB.contains` half-open bounds). Commit 5e4651707.

## Open gaps

| file:line | vanilla | gap | fix | size |
|---|---|---|---|---|
| `entity/vehicle/boat.rs:276` (`tick`) | `AbstractBoat#tickBubbleColumn`, `getBubbleTime`/`setBubbleTime`, `DATA_ID_BUBBLE_TIME` | Bubble columns are not implemented for boats: `DATA_ID_BUBBLE_TIME` (`crates/pumpkin-data/src/generated/tracked_data.rs:4103`, id 13) is never defined or sent, and the tick path never applies the upward push of a bubble column (so a boat sits in a bubble column instead of rising / wobbling). | Port `tickBubbleColumn`: count `BUBBLE_COLUMN` blocks inside the boat box, store/sync bubble time, apply the drag+vertical push, and decrement client-side. | M |
| `entity/vehicle/boat.rs:179` (`can_push`) | `AbstractBoat#push` / `Entity#push` | Rust pushes boats with `is_boat ? other_bottom < bounds.max.y : other_bottom <= bounds.min.y`. The asymmetric boat-vs-boat and boat-vs-mob cases were written without a decompiled reference; needs a side-by-side check of `AbstractBoat#push` before it can be called parity. | Re-derive both branches from decompiled `AbstractBoat`/`Entity#push` and right the comparison if it differs. | S |

## Notes (not gaps)
- `boat.rs:365` (`interact`) opens the chest-boat inventory when the player sneaks or the boat is full, and otherwise refuses to mount while any passenger eye is in water or the 60-tick out-of-control timer is running; that matches `AbstractBoat#interact` + `AbstractChestBoat#interact` ordering.
- `vehicle/vehicle.rs:167` already syncs `ID_HURT`, `ID_HURTDIR` and `ID_DAMAGE` together, matching `VehicleEntity#defineSynchedData`; the boat only had to add the paddle pair in `boat.rs:127`.
- `boat.rs:332`: creative destroys drop nothing and survival destroys drop the picked item plus scattered chest contents, matching vanilla `destroy`/`getPickResult`. Chest contents use the generic inventory path, not a `container` item component, which is the engine-wide minecart/chest choice.
- Paddle motion (`SPaddleBoat` -> `set_paddle_state`) is a pure metadata mirror of the client input; no server-side prediction exists in vanilla either.
- Boat drops for spruce resolve through `Item::from_registry_key("spruce_boat")`, so the variant item is correct without a per-wood table.
