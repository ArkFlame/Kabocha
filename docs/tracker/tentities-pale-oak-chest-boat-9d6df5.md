# Tracker: entities / Pale Oak Chest Boat

Re-review (RE-REVIEW-UNTRACKED). Reviewed source: `crates/pumpkin-core/src/entity/vehicle/boat.rs`
(shared by all 11 chest boats/rafts), `entity/type.rs:332-356`,
`item/items/boat.rs:21-98`, `entity/vehicle/minecart/container.rs`,
generated `pumpkin-data/src/generated/entity_type.rs`. No decompiled tree is
present (`.vendor/` empty), so the comparison is against
`net.minecraft.world.entity.vehicle.AbstractChestBoat` / `AbstractBoat` /
`BoatItem`, not quoted source.

The previous review's fixes are now on master (spectator/occupied open guard,
one-passenger cap, entity-position contents scatter, classification tests).
Two fresh findings this pass: the boat item drop still used the block origin
(fixed), and the remaining M/L gaps below are unchanged.

## Verified parity

| Behaviour | Pumpkin |
|---|---|
| Variant created as a boat entity | `entity/type.rs:332-356` (pale oak listed) |
| Chest detection for all 11 boats/rafts | `boat.rs:43-55` |
| 27-slot container only for chest boats | `boat.rs:37,84-85`; `minecart/container.rs:19-38` |
| One passenger seat (regular boats keep two) | `boat.rs:40,59-65,402` |
| Seat offset/ride height from generated `BoatSeats` | `boat.rs:210-242` |
| Sneak opens container, plain use boards | `boat.rs:390-428` |
| Menu only for non-spectators, alive, not occupied by another entity | `boat.rs:100-138` |
| Title `entity.minecraft.<variant>`, Bedrock chest-boat/raft key, custom name wins | `boat.rs:124-134`, `minecart/container.rs:227` |
| No `GameEvent.CONTAINER_OPEN` for entity containers | `minecart/container.rs:202-233` |
| Break threshold `damage + amount*10 > 40`, creative instabuild | `boat.rs:353-357`; `vehicle.rs:239-266` |
| Contents scatter at entity `pos` via `scatter_inventory_at` | `boat.rs:367-377` |
| Boat item drop at entity `pos` (new `drop_stack_at`) | `boat.rs:379-383` |
| `Items` NBT round trip at container root | `boat.rs:254-264`, `minecart/container.rs:40-81` |
| Item places matching entity, consumes one, increments stat | `item/items/boat.rs:159-211` |

## Fixed this pass

- `boat.rs:379-383` + `world/mod.rs:7239` - vanilla `AbstractBoat.destroy`
  calls `spawnAtLocation`, i.e. the entity position. The drop used
  `block_pos` (floored, x/z offset by 0.5 jitter) while the contents scatter
  already used `pos`. Added `World::drop_stack_at(Vector3<f64>, ItemStack)`
  and used it for the pick item; `drop_stack(BlockPos)` now delegates to it,
  so block drops keep block-centre behaviour.

## Remaining gaps

| # | Location | Vanilla class | Gap | Suggested fix | Size |
|---|---|---|---|---|---|
| 1 | `minecart/container.rs:172-233` | `AbstractContainerMenu.stillValid` / `Container.stillValid` | Entity-backed screen never becomes invalid: no 8-block distance or liveness check, so the menu survives walking away, boat destruction or unload. | Give `MinecartScreenFactory` an optional validity closure (`is_alive() && distance_squared <= 64`) evaluated on each slot click and close the menu when it fails; wire it for chest boats and chest minecarts. | M | STILL OPEN (h6 budget ended, not attempted) |
| 2 | `item/items/boat.rs:184-198` | `BoatItem.use` + `minecraft:container` component | Placing a chest boat item ignores the `minecraft:container` component, so a serialized boat with contents spawns empty. | After `BoatEntity::new`, copy the component stacks slot-for-slot (clamp 27) before `world.spawn_entity`. | M | STILL OPEN (h6 budget ended, not attempted) |
| 3 | `boat.rs:278-293` | `AbstractChestBoat implements Container` (no world hopper support) | `has_container_inventory()` exposes chest boats as a worldly container; if the hopper path inserts through that API a hopper above could fill a boat, which vanilla never does. | Trace `get_container_inventory` consumers; gate insertion per vehicle kind or expose the inventory only for read/drop paths. | M | STILL OPEN (h6 budget ended, not attempted) |
| 4 | `boat.rs:450-476` | `AbstractChestBoat` save/load + open guard | No automated test for the 27-slot `Items` NBT round trip or the occupied-boat open refusal; only classification/passenger tests exist. | Unit test: `BoatEntity` for `PALE_OAK_CHEST_BOAT` writes and reads slot 26 through `MinecartInventory`, and `open_custom_inventory_screen` returns false with a foreign passenger. | S | STILL OPEN (h6 budget ended, not attempted) |

Client-only effects (chest rendering, screen layout, paddle wobble) were not
assessed. No cargo/test run (worker constraint); the orchestrator batch-compiles.
