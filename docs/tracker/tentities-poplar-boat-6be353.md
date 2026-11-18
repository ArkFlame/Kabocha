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

## Gap status
- 1 seat offset magic numbers (S): FIXED (see Result; single shared helper, vanilla constants verified against .vendor decompile)
- 2 break drop resolves by registry key (S): FIXED (see Result; entity_to_item mapping with registry-key fallback, vanilla getDropItem verified)
- 3 is_chest_boat id allow-list (S): FIXED (see Result; derived from resource-name suffix, vanilla class distinction verified)

## Result

Verified against .vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/
(AbstractBoat.java, AbstractChestBoat.java). All three open rows are fixed in
crates/pumpkin-core/src/entity/vehicle/boat.rs and boat/passengers.rs.

- 1 seat offset magic numbers (S): FIXED. A single 'passenger_seat_offset' helper plus
  a shared 'passenger_attachment' now drive both 'entity::passenger_attachment_point'
  and the mount-positioning path in passengers.rs. Vanilla constants confirmed: lone
  rider uses the type offset (0.0 boat / 0.15 chest boat), two riders use +0.2 / -0.6,
  animal rider adds +0.2. Pure unit test
  'boat_seat_offset_follows_vanilla_single_multi_rider_and_animal_shift' pins the
  offsets; no magic numbers remain duplicated.
- 2 break drop resolves by registry key (S): FIXED. 'boat_drop_item' resolves through
  'BoatItem::entity_to_item' first (the same mapping as pick-block) with a registry-key
  fallback, so a renamed key no longer silently drops nothing.
  'every_boat_entity_type_resolves_a_drop_item' and
  'named_boat_keeps_its_drop_when_creative' cover it. Vanilla 'getPickResult' returns
  'new ItemStack(dropItem.get())', i.e. the item recorded per entity type.
- 3 is_chest_boat id allow-list (S): FIXED. 'is_chest_boat' now derives the ChestBoat
  distinction from the generated entity resource-name suffix ('_chest_boat' /
  'bamboo_chest_raft') instead of enumerating 11 ids, matching vanilla's class-based
  distinction. A new wood needs no extra entry here. 'max_passengers' is no longer const
  (str::ends_with is not const); it has no const callers. Existing coverage tests
  'chest_boat_detection_covers_every_chest_boat_entity_type' and
  'boat_stat_classification_covers_every_boat_entity_type' verify all entity types.

No generated data, WIT, or cross-crate changes were needed. Rustfmt run on boat.rs.

Re-verified against .vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/:
AbstractBoat.getPassengerAttachmentPoint confirms single-passenger offset, +0.2/-0.6 for two
riders and +0.2 for an Animal rider; AbstractBoat.getDropItem/getPickResult return the per-type
dropItem; AbstractChestBoat.getMaxPassengers=1 and getSinglePassengerXOffset=0.15. All three rows
are FIXED and the code matches vanilla. Compile-pitfall review clean: entity_to_item is pub(crate),
Item::from_registry_key exists, helpers are module-level free functions, no const callers of the
now non-const max_passengers/is_chest_boat, no unwrap/expect/panic in new code.
