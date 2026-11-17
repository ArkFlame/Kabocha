# Poplar Boat (entities) - re-review

Audit date: 2026-11-17. Scope: crates/pumpkin-core/src/entity/type.rs,
crates/pumpkin-core/src/entity/vehicle/boat.rs (+ boat/passengers.rs,
boat/dismount.rs, boat/physics.rs), crates/pumpkin-core/src/item/items/boat.rs,
crates/pumpkin-core/src/entity/player.rs, and the generated boat data in
crates/pumpkin-data/src/generated/entity_type.rs / tag.rs.
No decompiled vanilla source in this worktree (.vendor absent), so class
references are from the vanilla/Paper call chain, not byte-diffed.

Source changed since the 2026-11-16 review: cc2e20b55 moved seat offsets to
the generated BoatSeats data and 0c22e1aee changed the placement
entity-blocking test to inflate(0.3). Both changes were re-checked against
vanilla AbstractBoat#getPassengerAttachmentPoint / BoatItem#use and are
consistent with the poplar entries.

## Wiring verified (no gap)

- entity/type.rs:352-353 maps POPLAR_BOAT / POPLAR_CHEST_BOAT to BoatEntity.
- entity/vehicle/boat.rs:41-53 lists POPLAR_CHEST_BOAT in is_chest_boat, so it
  gets the 27-slot inventory and one passenger seat (boat.rs:92-94); plain
  POPLAR_BOAT keeps two seats.
- entity/vehicle/boat.rs:183-215 and boat/passengers.rs:44-83 both read
  entity_type.boat for ride height and single-passenger offset; poplar values
  (generated/entity_type.rs:9183-9186, 9239-9242) are 0.1875 / 0.0 and
  0.1875 / 0.15, matching vanilla AbstractBoat / AbstractChestBoat.
- item/items/boat.rs:41-42,91-92 maps the poplar boat items both ways, so
  placement, pick-block and breaking return the right item.
- entity/player.rs:4378,4389 counts BoatOneCm for both poplar boat types.
- generated/tag.rs:20187,20277 puts poplar_boat in c:boats / minecraft:boat;
  24011-24259 covers the broader boat and chest-boat tags with poplar_boat /
  poplar_chest_boat.
- generated/item.rs poplar boat items carry the expected components
  (MaxStackSize 1, CookingFuel time_boats, item model), identical to the other
  wood boats.

## Fixed on this branch

### 1. No regression test pinned the boat item/entity mapping (S)
- 05ce1b3b5 adds every_boat_item_round_trips_through_its_entity_type and
  poplar_boats_are_wired in item/items/boat.rs. The first iterates
  BoatItem::ids(), so a future wood omitted from item_to_entity, entity_to_item
  or is_chest_boat now fails the test; the second pins the poplar rows
  explicitly.

### 2. Boat movement statistic skipped poplar chest boat (S) - earlier review
- entity/player.rs:4365-4391: BoatOneCm now covers POPLAR_BOAT and
  POPLAR_CHEST_BOAT, matching vanilla where AbstractChestBoat extends
  AbstractBoat.

## Remaining gaps (not fixed)

### 1. Multi-passenger seat offsets are duplicated magic numbers (S)
- entity/vehicle/boat.rs:196-208 and boat/passengers.rs:53-57 both hardcode
  0.2 / -0.6 (plus ANIMAL_SEAT_SHIFT only in the former).
- Vanilla AbstractBoat#getPassengerAttachmentPoint hardcodes the same
  constants, and the generated BoatSeats data does not carry them, so there is
  no observable difference today. Fix by moving the offset choice into one
  helper used by both paths; risk is future drift between the two code paths,
  not current parity. Size S.

### 2. Boat break drop resolves the item by registry key (S)
- entity/vehicle/boat.rs:341-352 drops
  Item::from_registry_key(entity_type.resource_name).
- Vanilla AbstractBoat#getPickResult returns the item recorded on the entity
  type. Registry key lookup happens to work for every boat including poplar,
  but a renamed key would silently drop nothing; BoatItem::entity_to_item
  already backs pick-block (net/java/play/pick_item.rs) and the new test, so
  the drop could share it for consistency. No behaviour change for poplar.
  Size S.

### 3. is_chest_boat is an id allow-list (S)
- entity/vehicle/boat.rs:41-53 enumerates 11 chest-boat ids. Vanilla derives
  the distinction from the entity class (ChestBoat), so a new wood must be
  added in three places (type.rs, boat.rs, item/items/boat.rs).
- The new test flags a missed is_chest_boat entry, so the residual risk is
  reduced but not removed. Size S.
