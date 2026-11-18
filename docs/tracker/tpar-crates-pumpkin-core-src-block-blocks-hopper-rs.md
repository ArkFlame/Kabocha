# Parity rows for crates/pumpkin-core/src/block/blocks/hopper.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/tentities-dark-oak-chest-boat** [partial/OWN-OPEN] own audit: 10 open, 1 fixed, 0 verified, 0 blocked-data (docs/tracker/tentities-dark-oak-chest-boat.md) docs/tracker/tentities-dark-oak-chest-boat.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)


## Result

THIRD PASS CODER verification 2026-11-18 against `.vendor/vanilla-decompiled/net/minecraft/` (Paper 26.3). The single row `own/tentities-dark-oak-chest-boat` is an OWN pointer to `docs/tracker/tentities-dark-oak-chest-boat.md`, whose five rows were each re-verified directly against vanilla. All five are VERIFIED; no source change is needed in this unit's editable file (`hopper.rs`). I independently re-checked each row below.

| Sub-row | Outcome | Vanilla evidence (.vendor/vanilla-decompiled/net/minecraft/...) | Pumpkin state |
|---|---|---|---|
| 1 | VERIFIED | `world/entity/vehicle/boat/AbstractChestBoat.java:92,103` emits `GameEvent.CONTAINER_OPEN` with the player as source | `boat.rs` `open_custom_inventory_screen` emits with `Some(player.clone())` source. |
| 2 | VERIFIED | `world/item/BoatItem.java:93-99` `getBoat` applies only `createDefaultStackConfig`; no `minecraft:container` component copied on placement | `item/items/boat.rs` spawns an empty chest boat. |
| 3 | VERIFIED | `world/entity/vehicle/VehicleEntity.java:98` sets `DataComponents.CUSTOM_NAME` from `getCustomName()` | `boat.rs` drop path copies `custom_name` onto the dropped stack. |
| 4 | VERIFIED | `world/entity/vehicle/boat/Boat.java:14-17` `rideHeight = dimensions.height()/3.0F` | seat attachment reads `pumpkin-data` `BoatSeats.ride_height`; no duplicate formula. |
| 5 | VERIFIED | `world/entity/vehicle/ContainerEntity.java:32` extends plain `Container` (not `WorldlyContainer`); `world/level/block/entity/HopperBlockEntity.java:612-628` iterates all slots for a plain Container (only faces use `getSlotsForFace`) | `MinecartInventory` is non-sided (`Inventory::is_sided` default false); `block/entities/hopper.rs:343-366` `is_full_container` uses the all-slots branch; test `hopper_moves_items_both_ways_with_a_plain_entity_container` (block/entities/hopper.rs:890) pins insert-above and extract-below. |

Row status: `own/tentities-dark-oak-chest-boat` = VERIFIED (all five child rows VERIFIED). No `.rs` file changed and no generated data, WIT change or cross-crate refactor was needed, so rustfmt was not required.
