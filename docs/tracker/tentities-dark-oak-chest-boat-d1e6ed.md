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
- Remaining-gap items (prose, lines 5-13): VERIFIED across three re-reviews against .vendor/vanilla-decompiled; no dark-oak-specific gap; no code change needed.

## Result

Re-review against .vendor/vanilla-decompiled (this sweep has a decompile; the h6 note no longer applies). All four remaining-gap rows are VERIFIED; no dark-oak-specific gap exists, so no code change was needed.

- Row 1 (item/items/boat.rs:33-34): VERIFIED. item_to_entity maps DARK_OAK_BOAT -> EntityType::DARK_OAK_BOAT and DARK_OAK_CHEST_BOAT -> EntityType::DARK_OAK_CHEST_BOAT; entity_to_item (lines 53-58) reverses it via item_to_entity over BoatItem::ids(). Vanilla BoatItem.java holds one EntityType per item and drops the same entity, so the mapping is correct. No change.
- Row 2 (entity/vehicle/boat.rs, now 110-111): VERIFIED. is_chest_boat includes DARK_OAK_CHEST_BOAT (line 62); new() builds MinecartInventory::new(CHEST_BOAT_CONTAINER_SIZE) with CHEST_BOAT_CONTAINER_SIZE = 27 (line 41). Vanilla AbstractChestBoat.CONTAINER_SIZE = 27 and getContainerSize() returns 27. No change.
- Row 3 (entity/vehicle/boat.rs, now 146-193): VERIFIED. open_custom_inventory_screen rejects spectators and dead boats, refuses a boat occupied by someone other than the opener, opens the container, then emits CONTAINER_OPEN and angers nearby piglins on a successful open. This mirrors AbstractChestBoat.interact/createMenu plus ContainerEntity.interactWithContainerVehicle. Title key resolves to entity.minecraft.dark_oak_chest_boat (entity_type.resource_name = "dark_oak_chest_boat"; generated translation ENTITY_MINECRAFT_DARK_OAK_CHEST_BOAT matches). No change.
  Note (general chest-boat, not dark-oak-specific, left unchanged): this exact decompile's ContainerEntity.interactWithContainerVehicle no longer checks spectators/occupancy and Player.interactOn lets spectators open MenuProviders, while Pumpkin keeps older-vanilla spectator/occupancy gating. That affects every chest boat equally and is outside this dark-oak re-review; not fixed here.
- Row 4 (entity/vehicle/boat.rs, now 572-592): VERIFIED. chest_boat_detection_covers_every_chest_boat_entity_type iterates EntityType::ALL (includes DARK_OAK_CHEST_BOAT), asserts is_chest_boat on the _chest_boat / bamboo_chest_raft suffix and max_passengers 1 vs 2. The item round-trip is also covered in item/items/boat.rs:every_boat_item_round_trips_through_its_entity_type. Existing coverage is sufficient; no duplicate test added.

Not checked (unchanged): client-only dark oak boat model rendering and chest-open sound, which are client-only and not gated.

No .rs files were modified, so rustfmt was not needed.


## Result (re-review 2026-11-18)

Re-verified every STILL OPEN row against .vendor/vanilla-decompiled (BoatItem.java, AbstractBoat.java, AbstractChestBoat.java). All four rows are VERIFIED; no dark-oak-specific gap exists and no code change was needed.

- Row 1 (item/items/boat.rs:21-58): VERIFIED. item_to_entity maps DARK_OAK_BOAT -> EntityType::DARK_OAK_BOAT and DARK_OAK_CHEST_BOAT -> EntityType::DARK_OAK_CHEST_BOAT; entity_to_item reverses it via item_to_entity over BoatItem::ids(). Vanilla BoatItem.java stores one EntityType per item (field entityType, created via entityType.create) and drops that same type, so the 1:1 mapping is correct.
- Row 2 (entity/vehicle/boat.rs:44,76-86,245-268): VERIFIED. is_chest_boat matches the _chest_boat / bamboo_chest_raft resource-name suffix, which includes dark_oak_chest_boat. boat_container_size returns CHEST_BOAT_CONTAINER_SIZE = 27 and new() allocates MinecartInventory::new(size) only for chest boats. Vanilla AbstractChestBoat.CONTAINER_SIZE = 27 and getContainerSize() returns 27; AbstractBoat has no container.
- Row 3 (entity/vehicle/boat.rs:133-177,290-328): VERIFIED. boat_interact_outcome mirrors AbstractBoat.interact + AbstractChestBoat.interact (super result first, then canAddPassenger && !secondaryUse -> PASS, else interactWithContainerVehicle). On a successful open open_custom_inventory_screen emits CONTAINER_OPEN with the player as source and angers nearby piglins, matching openCustomInventoryScreen/interact. The earlier spectator/occupancy gating flagged in the prior Result has since been removed: vanilla openCustomInventoryScreen itself has no spectator or rider check (only createMenu guards spectators when a loot table is present, a Paper LootTable API behaviour), and current Pumpkin matches that. Title resolves to entity.minecraft.dark_oak_chest_boat.
- Row 4 (entity/vehicle/boat.rs:95-101,1104-1124): VERIFIED. Vanilla AbstractBoat.getMaxPassengers() returns 2 and AbstractChestBoat overrides it with 1; Pumpkin max_passengers returns 1 for chest boats, 2 otherwise. chest_boat_detection_covers_every_chest_boat_entity_type iterates EntityType::ALL (includes DARK_OAK_CHEST_BOAT) and asserts is_chest_boat plus max_passengers 1 vs 2. Existing coverage is sufficient; no duplicate test added.

Not checked (unchanged): client-only dark oak boat model rendering and chest-open sound, which are client-only and not gated. No .rs files were modified, so rustfmt was not needed.


## Result (re-review 2026-11-18, third pass)

The single STILL OPEN row (line 17, the prose gaps at lines 5-13) was re-verified against .vendor/vanilla-decompiled (BoatItem.java, AbstractChestBoat.java, plus the current Pumpkin sources). All four sub-rows are VERIFIED; no dark-oak-specific gap exists and no source file needed editing. The stale STILL OPEN flag has been changed to VERIFIED so this unit stops being re-queued.

- Row 1 (item/items/boat.rs:21-58): VERIFIED. item_to_entity maps DARK_OAK_BOAT -> EntityType::DARK_OAK_BOAT and DARK_OAK_CHEST_BOAT -> EntityType::DARK_OAK_CHEST_BOAT; entity_to_item reverses it via item_to_entity over BoatItem::ids(). Vanilla BoatItem.java declares a single `private final EntityType<? extends AbstractBoat> entityType` field and spawns exactly that type (getBoat -> entityType.create), so the 1:1 mapping is correct.
- Row 2 (entity/vehicle/boat.rs:44,76-86,245-268): VERIFIED. CHEST_BOAT_CONTAINER_SIZE = 27; is_chest_boat matches the `_chest_boat` / `bamboo_chest_raft` resource-name suffix (includes dark_oak_chest_boat); boat_container_size returns Some(27) only for chest boats and new() allocates MinecartInventory::new(size) on that basis. Vanilla AbstractChestBoat.CONTAINER_SIZE = 27, getContainerSize() = 27, and plain AbstractBoat has no container.
- Row 3 (entity/vehicle/boat.rs:133-177,290-328): VERIFIED. boat_interact_outcome mirrors AbstractChestBoat.interact (super result first, then canAddPassenger && !secondaryUse -> ride, else container). open_custom_inventory_screen has no spectator/rider gate and emits CONTAINER_OPEN with the player as source plus PiglinAi anger on a successful open, matching AbstractChestBoat.openCustomInventoryScreen/interact; vanilla only guards spectators in createMenu when a loot table exists. Title key resolves to entity.minecraft.dark_oak_chest_boat.
- Row 4 (entity/vehicle/boat.rs:95-101,1098-1118): VERIFIED. max_passengers returns 1 for chest boats and 2 otherwise; vanilla AbstractBoat.getMaxPassengers = 2, AbstractChestBoat overrides to 1. chest_boat_detection_covers_every_chest_boat_entity_type iterates EntityType::ALL (includes DARK_OAK_CHEST_BOAT), asserting is_chest_boat plus max_passengers 1 vs 2; item round-trip is covered in item/items/boat.rs::every_boat_item_round_trips_through_its_entity_type. Coverage is sufficient; no duplicate test added.

Only the tracker doc was edited (STILL OPEN flag + this Result). No .rs files changed; rustfmt not needed; no compile run (workers cannot compile).


## Result (session resume check 2026-11-18)

Resumed after server restart; no STILL OPEN rows remain for this unit. The `## Gap status` line was already flipped to VERIFIED in the third pass, and all four prose sub-rows (lines 5-13) were previously verified against .vendor/vanilla-decompiled (BoatItem.java, AbstractBoat.java, AbstractChestBoat.java): item<->entity 1:1 mapping for DARK_OAK_CHEST_BOAT, 27-slot chest boat inventory, boat_interact_outcome / open_custom_inventory_screen parity, and max_passengers 1 (chest) vs 2 (plain) with full EntityType::ALL test coverage. Unit names no source-file basenames to edit and no dark-oak-specific gap was found, so no .rs file was changed. Not repeating the completed re-reviews. No compile (workers cannot compile); no rustfmt (no .rs change).
