# entities / Poplar Chest Boat — re-review

Source: `crates/pumpkin-core/src/entity/vehicle/boat.rs` (shared boat/chest-boat), plus
`entity/type.rs:352-353`, `item/items/boat.rs:41-42,91-92`, `entity/player.rs:4378,4389`,
`entity/mod.rs:1238-1244`. Generated data: `pumpkin-data/src/generated/entity_type.rs:9187-9243`
(`POPLAR_CHEST_BOAT`, id 107, `BoatSeats { ride_height: 0.1875, single_passenger_x_offset: 0.15 }`),
`pumpkin-data/src/generated/tag.rs:24022` (`minecraft:boat` and `c:boats` both contain
`poplar_chest_boat`). Vanilla reference: `AbstractBoat`, `AbstractChestBoat` (`ContainerEntity`
line, 1.21.5+). No decompiled sources in `.vendor/`, so each row names the vanilla member it must
match.

Since the last review the source changed: `97ba2480d` (drop contents on creative break),
`baf497b61` (no `CONTAINER_OPEN` game event on chest-boat open), `0fe61a1fb` (spectator denial),
`570d3ea1f` (boat water physics split into `boat/{physics,block_effects,passengers,dismount}.rs`),
and poplar was added to every hand list. Re-audited against that tree.

## Fixed this pass
Nothing to fix. Every poplar-chest-boat behaviour the vanilla classes define is present and
matches; nothing was changed, so nothing can regress.

## Verified as matching (poplar chest boat)

| Behaviour | Rust | Vanilla member |
|---|---|---|
| `POPLAR_CHEST_BOAT` is a `BoatEntity` | `entity/type.rs:352-353` | `EntityType.CHEST_BOAT` factory |
| Detected as a chest boat (27-slot inventory created) | `boat.rs:51`, `boat.rs:72-73`, `boat.rs:88-90` | `AbstractChestBoat` constructor |
| Single seat only, no mob pickup | `boat.rs:92-94`, `entity/mod.rs:1238-1250` | `AbstractChestBoat` single-seat rule |
| Right-click opens chest when seat full or sneaking, else boards | `boat.rs:359-394` | `AbstractChestBoat.interact` |
| Spectators never get the menu | `boat.rs:97-101` | `AbstractChestBoat.openCustomInventoryScreen` |
| Title is entity display name; bamboo uses raft key, poplar uses boat key | `boat.rs:105-116` | `AbstractChestBoat.getDisplayName` |
| No `CONTAINER_OPEN` game event on open (`false`) | `boat.rs:116`, `minecart/container.rs` `open` | plain `player.openMenu`, no game event |
| `Items`/`LootTable`/`LootTableSeed` save and load | `boat.rs:227-237`, `minecart/container.rs` | `ContainerEntity` + `ContainerHelper.saveAllItems` |
| Loot table accepted and unpacked on first access | `boat.rs:255-266`, `boat.rs:333-337` | `ContainerEntity.unpackChestVehicleLootTable` |
| Chest contents scatter and boat item drops (with custom name) when destroyed, honouring `entity_drops`; creative scatters contents but drops no item | `boat.rs:307-357` | `AbstractBoat.hurtServer`/`destroy`, `ContainerEntity.chestVehicleDestroyed` |
| Rider offset 0.15 from generated data, not a hardcoded branch | `boat/passengers.rs:69-75`, `boat.rs:189-214` | `AbstractBoat.getPassengerAttachmentPoint` |
| Moves `minecraft.custom:minecraft.boat_one_cm` (also covered by `c:boats` now) | `entity/player.rs:4367-4392` | `Player.getMovementStatistic` |
| Place from item, space/entity checks, yaw, stack decrement, `ENTITY_PLACE` | `item/items/boat.rs:107-211` | `BoatItem.use` |
| Pick-block returns `poplar_chest_boat` | `item/items/boat.rs:52-58`, `pick_item.rs:100` | `AbstractBoat.getPickResult` |

## Remaining gaps (shared with the other chest-boat rows)

| # | Where | Vanilla member | Gap | Fix | Size |
|---|-------|----------------|-----|-----|------|
| 1 | `boat.rs:279-290` | `AbstractBoat.tick` float/move | With a player riding, the server zeroes velocity and skips `float_boat`/`move_boat`, so block effects and underwater counters advance but boat-vs-block/piston/entity resolution does not. Player movement is client-authoritative in Pumpkin (`docs/LEDGER.tsv` #1679), so this is a known design item, not a poplar defect | Port the vanilla input-driven float/move path instead of the early return | M |
| 2 | `boat.rs:341-352` | `AbstractBoat.getPickResult` / `AbstractChestBoat.getDropItem` | Destroyed-boat item is resolved with `Item::from_registry_key(resource_name)` while `BoatItem::entity_to_item` already owns the entity-item mapping; two mappings can diverge for a future wood type and silently drop no item. `poplar_chest_boat` resolves today, so there is no current behaviour difference | Use `BoatItem::entity_to_item(...)` and keep the registry lookup only as fallback | S |
| 3 | `boat.rs:41-53`, `item/items/boat.rs:21-98`, `entity/type.rs:332-353`, `entity/mod.rs:1240`, `entity/player.rs:4369-4391` | `AbstractChestBoat` subclasses | Five hand-maintained type lists; adding a wood only works if all five are edited (poplar needed exactly this). `c:boats` now covers both the movement-stat and mob-pickup paths, so the hand lists there are redundant | Derive from one `BoatKind`/flag generated from `resource_name`, then drop the redundant hand checks | M |
| 4 | `boat.rs:307-357` | `ContainerEntity` drops | Contents scatter only through `damage_with_context`; a non-damage destroying removal (`/kill`, `discard`) drops nothing. Matches vanilla if contents are tied to `destroy`, needs the decompiled `AbstractChestBoat.destroy`/`remove` to confirm | Confirm against decompiled source; if vanilla drops on `remove(KILLED)`, add a removal hook to `EntityBase` | M |

## Not gaps (checked)
27 slots, `Items` round trip, `LootTable`/seed handling, single-seat rule and mob pickup block,
interact ordering with sneak/occupied seat, spectator denial, no container-open game event,
bedrock `entity.chest_boat.name`, generated seat offset 0.15, `c:boats`/`minecraft:boat`
membership, `blocksBuilding`, `can_hit`/`is_collidable`/`is_pushable`, item placement
volume/collision checks, pick-block, creative scatter-without-item-drop.
