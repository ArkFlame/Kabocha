# entities / Acacia Chest Boat - audit (RE-REVIEW-UNTRACKED)

Re-audited after the source changed since the previous assessment. Reviewed
`crates/pumpkin-core/src/entity/vehicle/boat.rs`, `boat/{passengers,dismount,physics,block_effects}.rs`,
`crates/pumpkin-core/src/item/items/boat.rs`, `entity/type.rs:333`, `entity/mod.rs:1238-1265`,
`entity/player.rs:4360-4390`, `block/entities/hopper.rs:342`,
`crates/pumpkin-data/src/generated/entity_type.rs:279-336`, `tag.rs:20265`. No decompiled
vanilla tree in this worktree (`.vendor/` absent), so each row names the vanilla member it
must match; 1.21.x `AbstractBoat` / `AbstractChestBoat` / `BoatItem` behaviour from knowledge.

## Verified correct (no change needed)
| Where | Vanilla member | Note |
|---|---|---|
| boat.rs:40-52 `is_chest_boat` | `Boat.isChestBoat` | All 11 chest boat/raft types, incl. `ACACIA_CHEST_BOAT`, `PALE_OAK`, `POPLAR`; unit test covers every `EntityType`. |
| boat.rs:71, generated `entity_type.rs:279-336` | `AbstractChestBoat` ctor / `Boat` data | 27-slot container only for chest boats; acacia chest boat data has `ride_height 0.1875`, `single_passenger_x_offset 0.15`, `[0,0.5625,0]` attachment. |
| boat.rs:91-93, entity/mod.rs:1238-1245 | `Boat.getMaxPassengers` | Chest boats seat 1, plain boats 2. |
| boat.rs:95-116 | `AbstractChestBoat.openCustomInventoryScreen` | Spectator denied, custom name honoured, 3-row chest title, no `CONTAINER_OPEN` game event (block containers only). |
| boat.rs:208-213, passengers.rs:62-82 | `Boat.getPassengerAttachmentPoint` / `getRideHeight` | Both paths read `BoatSeats.ride_height`/`single_passenger_x_offset`; 0.2/-0.6 and animal +0.2 for two-seat boats. |
| boat.rs:226-253 | `AbstractChestBoat.addAdditionalSaveData/readAdditionalSaveData` | Container round-trips through `Items` via `MinecartInventory`; loot table unpacked on access. |
| boat.rs:306-347 | `Boat.hurtServer` + `AbstractChestBoat` drops | Threshold `damage + amount*10 > 40`, creative guest, `doEntityDrops` gate, contents scattered at the entity position before the item. |
| boat.rs:354-396 | `AbstractChestBoat.interact` | Secondary use opens, plain use boards a free seat or does nothing (fixed this pass). |
| boat/passengers.rs, dismount.rs | `Boat.getDismountLocationForPassenger` | Escape vector, water check and pose variants match. |

## Fixed this pass
| # | Where | Vanilla member | Gap | Fix |
|---|-------|----------------|-----|-----|
| 1 | boat.rs:354-360 | `AbstractChestBoat.interact` | A plain (non-secondary) use opened the container whenever no seat was available (occupied, out of control, eye in water, rider already mounted). Vanilla only opens on secondary use and otherwise delegates to `AbstractBoat.interact`, so an occupied chest boat cannot be looted with a plain right-click. | Open only when the player is sneaking. |
| 2 | boat.rs:341-347, imports | `AbstractBoat.destroy` (`spawnAtLocation(getDropItem())`) | The dropped chest boat item had the entity `CustomName` copied onto it; vanilla drops the plain pick item, so a renamed boat keeps its name only in Pumpkin. | Drop `ItemStack::new(1, BoatItem::entity_to_item(..))`, removed the `CustomNameImpl` copy; also replaced the separate `Item::from_registry_key` lookup with the shared `BoatItem::entity_to_item` mapping used by `getPickResult` (`pick_item.rs:100`). |

## Remaining gaps
| # | Where | Vanilla member | Gap | Fix | Size |
|---|-------|----------------|-----|-----|------|
| 1 | boat.rs:306-347 | `AbstractChestBoat.remove(RemovalReason)` / `chestVehicleDestroyed` | Contents and the boat item are only dropped from the damage path. A destroying removal that does not go through `hurtServer` (out-of-world teleport, command `discard`, chunk cleanup) removes the boat silently, losing the container. | Hook `Entity::remove` when `removal_reason.should_destroy()` and scatter once, keeping the item drop owned by the damage path. | M |
| 2 | item/items/boat.rs:179-200 (`BoatItem::use`) | `BoatItem.use` + `minecraft:container` component | Placing a chest boat item never reads the `minecraft:container` component, so a chest boat serialized with contents spawns empty (1.21.2+ items carry contents in that component). | After `BoatEntity::new`, copy `ContainerImpl` stacks into the 27 slots (clamped) before `world.spawn_entity`; confirm the 26.3 place path first. | M |
| 3 | entity/player.rs:7503, 7660 | Bedrock `ContainerOpen` packet | Entity-backed containers are sent with `target_entity_id: -1` and `BlockPos::ZERO`, so a chest boat UI is not tied to the vehicle. | Thread an optional `target_entity_id` through `Player::open_handled_screen` / `container::open` and pass the boat `entity_id`. | M |
| 4 | entity/mod.rs:1238-1245 | `Boat.getMaxPassengers` | The mob-boarding path repeats the chest-boat seat limit as a literal `1`/`2` instead of reusing `BoatEntity::get_max_passengers`, so a new chest-boat variant missing from `is_chest_boat` would take two mob passengers here while players see one. | Export a `pub const fn max_passengers(&EntityType)` from `vehicle/boat.rs` and call it from both sites. | S |
| 5 | block/entities/hopper.rs:342 (`has_container_inventory`) | `HopperBlockEntity.getContainerAt` | Chest boats expose `MinecartInventory` to the hopper path with no `WorldlyContainer` face rules. Vanilla hoppers do accept a plain `Container`, so this may already match; needs a check that hopper insert/extract on a chest boat matches vanilla before any change. | Verify; gate only if a face-restricted path is reached. | S |
| 6 | tests | - | No automated coverage for chest-boat contents: spawn an acacia chest boat, fill a slot, break it with a player, assert contents plus one boat item and no name; assert nothing on a creative break. | `pumpkin-gametest` scenario. | M |

## Notes
- No cargo/test run and no runtime repro (worker constraint); both fixes are source-level parity changes.
- Gap 2 of the previous assessment (non-damage removal) is now gap 1 above and is still open.
- Client-predicted boat motion, seat rendering and paddle animation are out of scope for this row.
