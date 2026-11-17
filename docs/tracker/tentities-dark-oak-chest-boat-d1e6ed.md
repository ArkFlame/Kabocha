# Dark Oak Chest Boat re-review (tentities-dark-oak-chest-boat-d1e6ed)

Scope: crates/pumpkin-core/src/entity/vehicle/boat.rs, crates/pumpkin-core/src/item/items/boat.rs.

## Remaining gaps

None found in the chest-boat path for DARK_OAK_CHEST_BOAT.

- item/items/boat.rs:33-34 maps DARK_OAK_BOAT and DARK_OAK_CHEST_BOAT items to their entity types; entity_to_item reverses it via item_to_entity.
- entity/vehicle/boat.rs:99-100 creates the 27-slot MinecartInventory when is_chest_boat is true (name suffix _chest_boat).
- entity/vehicle/boat.rs:115-149 open_custom_inventory_screen matches vanilla AbstractChestBoat (alive, empty or opener-only, spectators rejected); title uses entity.dark_oak_chest_boat translation key.
- entity/vehicle/boat.rs:488-507 existing unit test covers is_chest_boat and max_passengers for every EntityType including dark_oak_chest_boat.

Not checked: client-only rendering of the dark oak boat model and the chest-open sound (client-only, not gated).
