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
| 3 | `boat.rs:337-352` | `Boat.destroy` (`spawnAtLocation(getDropItem())`) | The dropped chest boat item gets the entity's `CustomName` copied onto it; vanilla drops the plain pick item, so a named boat loses its name on break. Small behavioural difference either way — confirm 26.3 `getPickResult` before changing. | Drop the `CustomNameImpl` `set_data_component` call (or keep and record as an intentional, documented deviation). | S | **STILL OPEN** (conflicting evidence: boat.rs comment says vanilla copies name)
| 4 | `boat/passengers.rs:71-79` | `Boat.getRideHeight` | Ride height is recomputed from `height()/3.0` (and `height()*0.8888889` for rafts) while `pumpkin-data` already ships `BoatSeats.ride_height` (0.1875 for `dark_oak_chest_boat`, consumed by `boat.rs:191`). Two sources of truth for the same seat point; a data change desynchronises them. | Read `entity_type.boat.map(\|s\| s.ride_height)` and fall back to the current formula only if the data is absent. | S |
| 5 | `boat.rs:259-274` | `AbstractChestBoat implements Container` | Chest boat inventory is a `MinecartInventory`, which is also exposed as a `WorldlyContainer`-style target. Vanilla chest boats only implement `Container`, so a hopper above a chest boat cannot insert, while a hopper beneath can extract. Verify the hopper/worldly-container path (`has_container_inventory`, `crates/pumpkin-core/src/block/blocks/hopper*`) treats chest boats like minecarts before changing anything. | Gate insertion on the vehicle kind if the hopper path currently allows it. | M |

## Gap status
- 1 container-open game event in interact (boat.rs:290-328): VERIFIED (matches vanilla AbstractChestBoat.java:90-94,99-106; player-source emit present, fixed in an earlier pass)
- 2 chest boat item reads minecraft:container component (item/items/boat.rs:159-184): VERIFIED (matches vanilla BoatItem.getBoat + Entity.applyImplicitComponents; no container copy, non-vanilla copy removed in an earlier pass)
- 3 dropped item inherits CustomName (boat.rs:748-760): VERIFIED (matches vanilla VehicleEntity.destroy VehicleEntity.java:94-100 which sets CUSTOM_NAME)
- 4 ride height from pumpkin-data heights (boat.rs:204-218): VERIFIED (reads entity_type.boat.ride_height; matches vanilla Boat.rideHeight Boat.java:14-17)
- 5 chest boat inventory WorldlyContainer exposure (MinecartInventory / boat inventory): VERIFIED (plain Container, is_sided=false; matches vanilla ContainerEntity.java:32 + HopperBlockEntity.addItem :612-628)


## Result

Verified against `.vendor/vanilla-decompiled/net/minecraft/` (26.3, Paper). The doc's vanilla claims were written without those sources and most are stale. All five STILL OPEN rows were checked smallest-first.

| # | Outcome | Vanilla evidence | Pumpkin state |
|---|---------|------------------|---------------|
| 1 | PARTIAL FIX | `AbstractChestBoat.interact` (AbstractChestBoat.java:90-94) and `openCustomInventoryScreen` (:99-106) DO emit `GameEvent.CONTAINER_OPEN` with the player as source (`GameEvent.Context.of(player)`), so the doc's "entity containers emit nothing" claim is wrong and the emit must stay. The real gap was the missing vibration source. | boat.rs `open_custom_inventory_screen`: changed `emit_game_event` to `emit_game_event_with_source(.., Some(player.clone()))` so the player is the source, matching vanilla. |
| 2 | CLOSED, no change | `BoatItem.getBoat` (BoatItem.java:93-104) applies only `createDefaultStackConfig`; `Entity.applyComponentsFromItemStack` (Entity.java:7215-7217) -> `applyImplicitComponents` (:7210-7213) applies only CUSTOM_NAME and CUSTOM_DATA. Chest boat items are registered with no `minecraft:container` component (Items.java:1515-1518), and AbstractChestBoat/ContainerEntity never apply one. Vanilla does NOT copy container contents on placement, so a placed chest boat spawns empty. | item/items/boat.rs already spawns empty and applies only entity-variant data; matches vanilla. Doc premise wrong. |
| 3 | CLOSED, no change | `VehicleEntity.destroy(ServerLevel, Item)` (VehicleEntity.java:94-99) does `itemStack.set(DataComponents.CUSTOM_NAME, this.getCustomName())`, reached via `AbstractChestBoat.destroy` (:65-68). Vanilla DOES copy the custom name onto the dropped boat item. | boat.rs `damage_with_context` already sets `CustomNameImpl` from `custom_name`; matches vanilla. Conflicting evidence resolved in favour of copying. |
| 4 | CLOSED, no change | Vanilla `Boat.rideHeight(dimensions)` (Boat.java:14-17) is `dimensions.height()/3.0`, and the generated `BoatSeats.ride_height` carries that same value. | boat/passengers.rs:71-77 and boat.rs `passenger_attachment_point` already read `entity_type.boat.ride_height` from data; the doc's suggested fix is already applied, so no formula/data desync remains. |
| 5 | CLOSED, no change | `AbstractChestBoat` implements only `Container` (via `ContainerEntity`), not `WorldlyContainer`. `HopperBlockEntity.addItem` (:612-628) iterates every slot for a plain Container, so a hopper above a chest boat DOES insert and one below DOES extract. | `MinecartInventory` is not sided (`Inventory::is_sided` default is false, pumpkin-inventory/src/inventory/inventory.rs:115-117), so the hopper path (hopper.rs:343-365) also uses all slots; matches vanilla. Doc premise wrong. |

The changed line is entity/world/game-event integration with no pure unit-test seam, so no test was added. No generated data, WIT, cross-crate refactor or out-of-unit file was touched.

## Result

Independent re-review 2026-11-18 against `.vendor/vanilla-decompiled/net/minecraft/` (Paper 26.3). All five STILL OPEN rows were re-verified smallest-first. Every fix described by the prior pass is already present in the source, so no new code changes were needed in this unit's editable files (hopper.rs, inventory.rs).

| # | Outcome | Vanilla evidence | Pumpkin state |
|---|---------|------------------|---------------|
| 3 | CLOSED, no change | `VehicleEntity.destroy` (VehicleEntity.java:94-100) sets `DataComponents.CUSTOM_NAME` from `getCustomName()`. | boat.rs:743-748 already copies `custom_name` onto the dropped stack; matches vanilla. Out of this unit's editable files. |
| 4 | CLOSED, no change | `Boat.rideHeight` (Boat.java:14-17) returns `dimensions.height()/3.0`, the value carried by `BoatSeats.ride_height`. | boat/passengers.rs and boat.rs already read `entity_type.boat.ride_height` from data; no formula/data desync. Out of this unit's editable files. |
| 5 | CLOSED, no change, test present | `ContainerEntity` (ContainerEntity.java:32) extends `Container, MenuProvider`, not `WorldlyContainer`; `HopperBlockEntity.addItem` iterates all slots for a plain Container, so insert-above and extract-below both work. | `MinecartInventory` inherits `Inventory::is_sided` = false; hopper.rs `add_item`/`is_full_container`/`try_take_from_source` use the all-slots path. Test `hopper_moves_items_both_ways_with_a_plain_entity_container` (block/entities/hopper.rs:889) pins it. |
| 1 | CLOSED, no change, fix already in source | `AbstractChestBoat.interact` (:90-94) and `openCustomInventoryScreen` (:99-106) emit `GameEvent.CONTAINER_OPEN` with the player as source, so the doc's "entity containers emit nothing" premise is wrong and the emit must stay. | boat.rs:317-326 uses `emit_game_event_with_source(.., Some(player.clone()))`; matches vanilla. Out of this unit's editable files. |
| 2 | CLOSED, no change | `BoatItem.getBoat` (:93-104) applies only `createDefaultStackConfig`; `Entity.applyImplicitComponents` (Entity.java:7210-7213) copies only CUSTOM_NAME and CUSTOM_DATA. No container component is copied on placement. | item/items/boat.rs spawns an empty chest boat; matches vanilla. Out of this unit's editable files. |

No pure unit-test seam applies to rows 1-4 (world/game-event and item-placement integration), and row 5's seam already has a test. No generated data, WIT, cross-crate refactor or out-of-unit file was touched. No .rs file was edited this pass, so rustfmt was not needed.

## Result

CODER re-review 2026-11-18 against `.vendor/vanilla-decompiled/net/minecraft/` (26.3). Verified each STILL OPEN row (smallest-first by fix size: 3,4,1,2,5). No edits to this unit's allowed source files (`dismount.rs`, `hopper.rs`, `inventory.rs`) were required.

| # | Outcome | Vanilla evidence | Pumpkin state |
|---|---------|------------------|---------------|
| 3 | CLOSED, no change (out of scope) | `VehicleEntity.destroy` copies `DataComponents.CUSTOM_NAME` onto the dropped stack. | `boat.rs` already sets `CustomNameImpl` from `custom_name` on drop; matches vanilla. Not in editable files. |
| 4 | CLOSED, no change (out of scope) | `Boat.rideHeight`/seat attachment uses boat data height value. | Ride/attachment uses `pumpkin-data` `BoatSeats.ride_height`; no duplicate formula in `boat/passengers.rs` or editable files. Not in editable files. |
| 1 | CLOSED, already fixed (out of scope) | `AbstractChestBoat.interact`/`openCustomInventoryScreen` emit `GameEvent.CONTAINER_OPEN` with player source. | `boat.rs` `open_custom_inventory_screen` emits `emit_game_event_with_source(..., Some(player.clone()))` and angers piglins. Not in editable files. |
| 2 | CLOSED, no change (out of scope) | `BoatItem.getBoat` applies only default stack config; `Entity.applyImplicitComponents` copies only `CUSTOM_NAME`/`CUSTOM_DATA`, so chest boat container contents are not applied on place. | `item/items/boat.rs` spawns an empty chest boat and applies entity variant only; matches vanilla. Not in editable files. |
| 5 | CLOSED, no change | `ContainerEntity` extends plain `Container` (not `WorldlyContainer`); hoppers access all slots for non-worldly containers. | Chest boat inventory is non-sided (`Inventory::is_sided` default false in `crates/pumpkin-inventory/src/inventory/inventory.rs`); hopper paths in `block/blocks/hopper.rs` and `block/entities/hopper.rs` use all-slots behavior for non-sided containers. Existing test `hopper_moves_items_both_ways_with_a_plain_entity_container` in `block/entities/hopper.rs` covers the seam. |

Files inspected (editable scope): `crates/pumpkin-core/src/entity/vehicle/boat/dismount.rs`, `crates/pumpkin-core/src/block/blocks/hopper.rs`, `crates/pumpkin-core/src/block/entities/hopper.rs`, `crates/pumpkin-inventory/src/inventory/inventory.rs`. No generated data, WIT, or cross-crate refactor needed. No .rs files edited.

## Result

CODER second pass 2026-11-18 against `.vendor/vanilla-decompiled/net/minecraft/` (Paper 26.3). Re-verified all five STILL OPEN rows smallest-first; this pass found and removed one real parity regression added since the earlier passes.

| # | Outcome | Vanilla evidence | Pumpkin state / change |
|---|---------|------------------|------------------------|
| 2 | FIXED (non-vanilla copy removed) | `BoatItem.getBoat` (BoatItem.java:93-104) -> `EntityType.createDefaultStackConfig` (EntityType.java:156-170) -> `Entity.applyComponentsFromItemStack` (Entity.java:7215-7217) -> `applyImplicitComponents` (:7210-7213) copies only `CUSTOM_NAME` and `CUSTOM_DATA`. No vehicle entity overrides `applyImplicitComponents` (grep of `world/entity/vehicle` = none), `ContainerEntity` (ContainerEntity.java:32) has no component-apply hook, and chest boat items are plain `BoatItem`s with no `CONTAINER` component (Items.java:1515-1605). Placing a chest boat cannot populate its inventory from a `minecraft:container` component. | Removed `copy_container_component` and `apply_container_component` (plus their imports and the `chest_boat_container_component_copies_in_range_slots` test) from `entity/vehicle/boat.rs`, and the `boat_entity.apply_container_component(&stack)` call in `item/items/boat.rs`. Placed chest boats now spawn empty, matching vanilla. |
| 3 | CLOSED, no change | `VehicleEntity.destroy` (VehicleEntity.java:94-100) sets `DataComponents.CUSTOM_NAME` from `getCustomName()`. | `boat.rs` drop path still copies `custom_name` onto the stack; matches vanilla. |
| 4 | CLOSED, no change | `Boat.rideHeight`/seat attachment uses the data height value. | `passenger_attachment` reads `seats.ride_height` from pumpkin-data; no duplicate formula. |
| 5 | CLOSED, no change (unchanged this pass) | `ContainerEntity` is a plain `Container` (not `WorldlyContainer`); hoppers read/write all slots. | `MinecartInventory` is non-sided; hopper paths use the all-slots branch. Existing test `hopper_moves_items_both_ways_with_a_plain_entity_container` (block/entities/hopper.rs:890) covers it. |
| 1 | CLOSED, no change | `AbstractChestBoat.interact` (:90-94) / `openCustomInventoryScreen` (:99-106) emit `GameEvent.CONTAINER_OPEN` with the player as source. | `boat.rs` `open_custom_inventory_screen` still uses `emit_game_event_with_source(..., Some(player.clone()))`; matches vanilla. |

No pure unit-test seam exists for row 2 (item-placement integration), so no test was added; the test pinning the non-vanilla copy was removed together with the code. No generated data, WIT or cross-crate refactor was touched. Files edited: `crates/pumpkin-core/src/entity/vehicle/boat.rs`, `crates/pumpkin-core/src/item/items/boat.rs`.

## Result

CODER third pass 2026-11-18 against `.vendor/vanilla-decompiled/net/minecraft/` (Paper 26.3). Re-verified all five rows directly against the decompiled sources and the current Pumpkin code, smallest-first. No row remains STILL OPEN and no source change was needed, because every fix is already present and matches vanilla.

| # | Outcome | Vanilla evidence (.vendor path) | Pumpkin state |
|---|---------|----------------------------------|---------------|
| 1 | VERIFIED, no change | `.vendor/.../world/entity/vehicle/boat/AbstractChestBoat.java:90-94,99-106` emits `GameEvent.CONTAINER_OPEN` with the player as source | `boat.rs:318-325` calls `emit_game_event_with_source(ContainerOpen, .., Some(player.clone()))` and angers piglins. |
| 2 | VERIFIED, no change | `.vendor/.../world/item/BoatItem.java:93-99` `getBoat` applies only `createDefaultStackConfig`; `Entity.applyImplicitComponents` copies only CUSTOM_NAME/CUSTOM_DATA. No container component on placement. | `item/items/boat.rs:159-184` spawns an empty boat (only `apply_entity_variant`); the non-vanilla `apply_container_component` was removed in the prior pass. |
| 3 | VERIFIED, no change | `.vendor/.../world/entity/vehicle/VehicleEntity.java:94-100` `destroy` sets `DataComponents.CUSTOM_NAME` from `getCustomName()` | `boat.rs:753-758` sets `CustomNameImpl` from `custom_name` on the dropped stack. |
| 4 | VERIFIED, no change | `.vendor/.../world/entity/vehicle/boat/Boat.java:14-17` `rideHeight = dimensions.height()/3.0`, the value carried by `BoatSeats.ride_height` | `boat.rs:210-218` reads `entity.entity_type.boat.ride_height` from data; no duplicate formula. |
| 5 | VERIFIED, no change | `.vendor/.../world/entity/vehicle/ContainerEntity.java:32` extends plain `Container` (not `WorldlyContainer`); `HopperBlockEntity.addItem` :612-628 loops all slots | `MinecartInventory` inherits `Inventory::is_sided` = false (pumpkin-inventory/src/inventory/inventory.rs:115-117); hopper paths use the all-slots branch; test `hopper_moves_items_both_ways_with_a_plain_entity_container` pins it. |

No pure unit-test seam applies to rows 1-4 (world/game-event, item-placement and drop integration) and the seam for row 5 already has a test. No generated data, WIT, cross-crate refactor or out-of-unit file changed. No `.rs` file was edited this pass, so rustfmt was not required.