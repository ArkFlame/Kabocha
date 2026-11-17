# entities / Dark Oak Boat — audit

Sources: crates/pumpkin-core/src/item/items/boat.rs, crates/pumpkin-core/src/entity/vehicle/boat.rs and its block_effects/dismount/passengers/physics submodules, crates/pumpkin-core/src/net/java/play/paddle_boat.rs, crates/pumpkin-core/src/net/java/play/pick_item.rs, crates/pumpkin-core/src/block/blocks/redstone/dispenser.rs.
Compared against vanilla BoatItem (use/clip/entity check), AbstractBoat (interact, tick, floatBoat, move, getStatus, isUnderwater, getGroundFriction, hurt/destroy, getDismountLocationForPassenger), AbstractChestBoat (canAddPassenger, container/destroy), EntityType seat data in pumpkin-data, plus docs/VANILLA_REF.md ("passengers 2 (chest 1), chest 27 slots, drops contents").

## Fixed in this audit
- Chest boats accepted a second passenger (boat.rs interact used the flat MAX_PASSENGERS = 2 for every boat). Vanilla `AbstractChestBoat#canAddPassenger` seats one entity because the second seat is the chest, so right-clicking a chest boat that already has one rider opens the chest instead of boarding. `BoatEntity::interact` now caps chest boats at `CHEST_BOAT_MAX_PASSENGERS = 1` via `max_passengers(entity_type)` and still opens the chest for the second clicker.

## Remaining gaps
| # | Location | Vanilla | Gap | Size |
|---|---|---|---|---|
| 1 | crates/pumpkin-core/src/entity/vehicle/boat.rs:286-311, crates/pumpkin-core/src/entity/vehicle/boat/block_effects.rs:49 | AbstractBoat#tick plays `entity.boat.paddle_water` while a paddle moves and a land baseline sound when a boat hits ground at speed | No boat sound is emitted anywhere in the boat paths (grep for sound in entity/vehicle/boat* and item/items/boat.rs returns nothing), so paddling and beaching are silent server-side; only sends when a vanilla client would predict it too. | S |
| 2 | crates/pumpkin-core/src/entity/vehicle/boat/physics.rs:142-183 | Boat#getGroundFriction returns 0.0F when no block under the boat has collision | `friction / count` divides by zero when count == 0 and yields NaN. Today it is harmless (NaN > 0.0 is false, so the status is InAir and land_friction is never stored), but a guard makes the intent explicit and stops NaN propagating if the caller changes. | S |

## Checked, no change
- BoatItem.use: view-vector entity scan with CAN_HIT filter, 0.3 inflate eye test, MISS early return, yaw copy from the player, item shrink with creative bypass, ITEM_USED stat, custom name/variant via apply_entity_variant (spawn_egg.rs:143-146) all match vanilla; entity_to_item drives middle-click pick result (pick_item.rs:125) and returns the matching chest boat item.
- Placement dimensions/box: generated EntityType dimension + eye height used for the collision box, matching vanilla's Boat construction.
- AbstractBoat.tick ordering (underwater eject at 60 ticks, client-controlled zero velocity, floatBoat/moveBoat, block effects, block collisions) and BoatStatus/isUnderwater/getWaterLevelAbove match vanilla, including the float-then-widen fluid height arithmetic.
- Physics constants: default friction 0.9, underwater 0.45, under-flowing-water vertical speed -7.0e-4, buoyancy 0.01 * (0.04/0.65) * 0.75, gravity 0.04; land friction from block slipperiness (water 0.9 / land 0.6 / ice 0.98 per VANILLA_REF).
- Dismount: horizontal escape vector uses boat width * sqrt(2) against passenger width and passenger yaw, water check on the target column, standing/crouching/swimming poses for players, world border test.
- Damage/destroy: 10x damage threshold of 40, creative instabuild break drops nothing, chest 27-slot inventory unscattered to the world with loot table unpack, entity drops gated on doEntityDrops, custom name carried onto the dropped item.
- Chest boat container: spectator rejection, occupied-by-other rejection, 27 slots, container open game event, NBT inventory read/write.
- Paddle input is read from SPaddleBoat and stored/emitted as metadata (paddle_boat.rs, BoatEntity::set_paddles); boat dispenser behaviour spawns at the vanilla offset with the facing yaw and copies the item custom name.
- Dark oak data: DARK_OAK_BOAT entity registered in entity/type.rs:338 with BoatEntity, boat seats (ride height 0.1875, single-passenger offset 0) from pumpkin-data, item id 987 and recipe from assets; nothing dark-oak-specific differs from the shared boat code.
