# entities / Dark Oak Chest Boat — re-review
Source: `crates/pumpkin-core/src/entity/vehicle/boat.rs` (+ `boat/{passengers,dismount,physics,block_effects}.rs`, `item/items/boat.rs`).
Reference: vanilla `net.minecraft.world.entity.vehicle.boat.AbstractChestBoat` / `Boat` / `BoatItem`. No decompiled sources in `.vendor/`, so each row names the vanilla member it must match.

## Verified correct this pass (no change needed)
| Where | Vanilla member | Note |
|---|---|---|
| boat.rs:43-55 `is_chest_boat` | `Boat.isChestBoat` | All 11 chest boat/raft entity types listed (incl. PALE_OAK, POPLAR). |
| boat.rs:72-88, data `BoatSeats` | `AbstractChestBoat` ctor | 27-slot container created only for chest boats; `dark_oak_chest_boat` data has `ride_height: 0.1875`, `single_passenger_x_offset: 0.15`. |
| boat.rs:94-108, passengers.rs:67-69 | `Boat.canAddPassenger` / `getSinglePassengerXOffset` | Chest boat seats 1, offset 0.15; normal boat seats 2 with 0.2 / -0.6. |
| boat.rs:191-221 | `Boat.getPassengerAttachmentPoint` | Seat offsets and animal shift match (0.2 front, -0.6 back, +0.2 animal). |
| boat.rs:225-245 | `AbstractChestBoat.addAdditionalSaveData/readAdditionalSaveData` | Container round-trips through `Items` via `MinecartInventory`; loot table keys not written for boats. |
| boat.rs:315-362 | `Boat.hurt` + `AbstractChestBoat` drops | Break threshold `damage + amount*10 > 40`, creative instabuild, `entity_drops` gating, contents scattered before the boat item. |
| boat.rs:364-395 | `AbstractChestBoat.interact` | Board when a seat is free and not sneaking/underwater, otherwise open the menu; sneaking on a non-chest boat falls through to `false`. |
| dismount.rs:16-72 | `Boat.getDismountLocationForPassenger` | Horizontal escape vector, water check, standing/crouching/swimming poses. |
| entity/mod.rs:1232-1256 | `Boat` mob boarding | Chest boats cap at 1 passenger, normal boats at 2. |

## Fixed this pass
| # | Where | Vanilla member | Gap | Fix |
|---|-------|----------------|-----|-----|
| 1 | boat.rs:113-121 (removed) | `AbstractChestBoat.interact` | Opening a chest boat emitted `GameEvent.ContainerOpen` (plus a `source_entity` context) at the entity position. Vanilla only emits that event from block containers (`ChestBlock`, `BarrelBlock`, shulker box); entity containers — chest boat, chest minecart (same `container::open` path) — emit nothing, so this produced phantom sculk-sensor vibrations that vanilla does not have. | Dropped the emit and the now-unused `GameEvent`/`GameEventContext` imports; `open_custom_inventory_screen` just returns `container::open(..)`. |

## Remaining gaps
| # | Where | Vanilla member | Gap | Fix | Size |
|---|-------|----------------|-----|-----|------|
| 2 | `item/items/boat.rs:179-191` | `BoatItem.use` + `AbstractChestBoat` container component | Placing a chest boat item never reads the `minecraft:container` component, so a chest boat serialized with contents spawns empty. Vanilla boat items carry their contents in that component since 1.21.2. | After `BoatEntity::new`, copy `ContainerImpl` stacks into the chest boat inventory (slot-for-slot, clamp to 27) before `world.spawn_entity`. Confirm against the 26.3 `BoatItem`/`EntityType.create` path first — if vanilla drops the component on place, document that instead and close the row. | M |
| 3 | `boat.rs:337-352` | `Boat.destroy` (`spawnAtLocation(getDropItem())`) | The dropped chest boat item gets the entity's `CustomName` copied onto it; vanilla drops the plain pick item, so a named boat loses its name on break. Small behavioural difference either way — confirm 26.3 `getPickResult` before changing. | Drop the `CustomNameImpl` `set_data_component` call (or keep and record as an intentional, documented deviation). | S |
| 4 | `boat/passengers.rs:71-79` | `Boat.getRideHeight` | Ride height is recomputed from `height()/3.0` (and `height()*0.8888889` for rafts) while `pumpkin-data` already ships `BoatSeats.ride_height` (0.1875 for `dark_oak_chest_boat`, consumed by `boat.rs:191`). Two sources of truth for the same seat point; a data change desynchronises them. | Read `entity_type.boat.map(\|s\| s.ride_height)` and fall back to the current formula only if the data is absent. | S |
| 5 | `boat.rs:259-274` | `AbstractChestBoat implements Container` | Chest boat inventory is a `MinecartInventory`, which is also exposed as a `WorldlyContainer`-style target. Vanilla chest boats only implement `Container`, so a hopper above a chest boat cannot insert, while a hopper beneath can extract. Verify the hopper/worldly-container path (`has_container_inventory`, `crates/pumpkin-core/src/block/blocks/hopper*`) treats chest boats like minecarts before changing anything. | Gate insertion on the vehicle kind if the hopper path currently allows it. | M |
