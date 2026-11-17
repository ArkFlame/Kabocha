# Dark Oak Chest Boat re-review (tentities-dark-oak-chest-boat-d1e6ed)

Scope: crates/pumpkin-core/src/entity/vehicle/boat.rs, crates/pumpkin-core/src/item/items/boat.rs.

## Remaining gaps

None found in the chest-boat path for DARK_OAK_CHEST_BOAT.

- item/items/boat.rs:33-34 maps DARK_OAK_BOAT and DARK_OAK_CHEST_BOAT items to their entity types; entity_to_item reverses it via item_to_entity.
- entity/vehicle/boat.rs:99-100 creates the 27-slot MinecartInventory when is_chest_boat is true (name suffix _chest_boat).
- entity/vehicle/boat.rs:115-149 open_custom_inventory_screen matches vanilla AbstractChestBoat (alive, empty or opener-only, spectators rejected); title uses entity.dark_oak_chest_boat translation key.
- entity/vehicle/boat.rs:488-507 existing unit test covers is_chest_boat and max_passengers for every EntityType including dark_oak_chest_boat.

Not checked: client-only rendering of the dark oak boat model and the chest-open sound (client-only, not gated).

## Gap status
- Remaining-gap items (prose, lines 5-13, not enumerated in this sweep): STILL OPEN (unverified: no .vendor decompile in h6; not changed)

## Result

Re-review against .vendor/vanilla-decompiled (this sweep has a decompile; the h6 note no longer applies). All four remaining-gap rows are VERIFIED; no dark-oak-specific gap exists, so no code change was needed.

- Row 1 (item/items/boat.rs:33-34): VERIFIED. item_to_entity maps DARK_OAK_BOAT -> EntityType::DARK_OAK_BOAT and DARK_OAK_CHEST_BOAT -> EntityType::DARK_OAK_CHEST_BOAT; entity_to_item (lines 53-58) reverses it via item_to_entity over BoatItem::ids(). Vanilla BoatItem.java holds one EntityType per item and drops the same entity, so the mapping is correct. No change.
- Row 2 (entity/vehicle/boat.rs, now 110-111): VERIFIED. is_chest_boat includes DARK_OAK_CHEST_BOAT (line 62); new() builds MinecartInventory::new(CHEST_BOAT_CONTAINER_SIZE) with CHEST_BOAT_CONTAINER_SIZE = 27 (line 41). Vanilla AbstractChestBoat.CONTAINER_SIZE = 27 and getContainerSize() returns 27. No change.
- Row 3 (entity/vehicle/boat.rs, now 146-193): VERIFIED. open_custom_inventory_screen rejects spectators and dead boats, refuses a boat occupied by someone other than the opener, opens the container, then emits CONTAINER_OPEN and angers nearby piglins on a successful open. This mirrors AbstractChestBoat.interact/createMenu plus ContainerEntity.interactWithContainerVehicle. Title key resolves to entity.minecraft.dark_oak_chest_boat (entity_type.resource_name = "dark_oak_chest_boat"; generated translation ENTITY_MINECRAFT_DARK_OAK_CHEST_BOAT matches). No change.
  Note (general chest-boat, not dark-oak-specific, left unchanged): this exact decompile's ContainerEntity.interactWithContainerVehicle no longer checks spectators/occupancy and Player.interactOn lets spectators open MenuProviders, while Pumpkin keeps older-vanilla spectator/occupancy gating. That affects every chest boat equally and is outside this dark-oak re-review; not fixed here.
- Row 4 (entity/vehicle/boat.rs, now 572-592): VERIFIED. chest_boat_detection_covers_every_chest_boat_entity_type iterates EntityType::ALL (includes DARK_OAK_CHEST_BOAT), asserts is_chest_boat on the _chest_boat / bamboo_chest_raft suffix and max_passengers 1 vs 2. The item round-trip is also covered in item/items/boat.rs:every_boat_item_round_trips_through_its_entity_type. Existing coverage is sufficient; no duplicate test added.

Not checked (unchanged): client-only dark oak boat model rendering and chest-open sound, which are client-only and not gated.

No .rs files were modified, so rustfmt was not needed.
