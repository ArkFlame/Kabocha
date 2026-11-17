# Poplar Boat (entities) - re-review

Audit date: 2026-11-16. Scope: `crates/pumpkin-core/src/entity/type.rs`,
`crates/pumpkin-core/src/entity/vehicle/boat.rs` (+ `boat/passengers.rs`,
`boat/dismount.rs`), `crates/pumpkin-core/src/item/items/boat.rs`,
`crates/pumpkin-core/src/entity/player.rs`, and the generated boat data in
`crates/pumpkin-data/src/generated/entity_type.rs` / `tag.rs`. No decompiled
vanilla source in this worktree (`.vendor/` absent), so class references are
from the vanilla/Paper call chain, not byte-diffed.

The poplar boat wiring added since the last review is in place and matches the
other wood boats:

- `entity/type.rs:352-353` maps `POPLAR_BOAT` / `POPLAR_CHEST_BOAT` to
  `BoatEntity`.
- `entity/vehicle/boat.rs:53` lists `POPLAR_CHEST_BOAT` in `is_chest_boat`,
  so it gets the 27-slot inventory and one passenger seat.
- `item/items/boat.rs:41-42,91-92` maps the poplar boat items both ways, so
  placement, pick-block and breaking return the right item.
- `generated/entity_type.rs:9181-9243` has the seat data (0.1875 ride height,
  0.15 chest-boat offset) identical to vanilla `AbstractChestBoat`.
- `generated/tag.rs:20187,20277` puts `poplar_boat` in `minecraft:boat` /
  `c:boats` (chest boats are deliberately not in `minecraft:boat`, as in
  vanilla).

## Fixed on this branch

### Boat movement statistic skipped poplar chest boat
- `crates/pumpkin-core/src/entity/player.rs:4365-4386`: `get_movement_statistic`
  decides `BoatOneCm` from the `minecraft:boat` / `c:boats` tags plus an
  explicit list of boat types. The tag only covers plain boats, and the explicit
  list ended at `PALE_OAK_CHEST_BOAT` / `BAMBOO_CHEST_RAFT`.
- Vanilla `ServerPlayer` counts `Statistic.BOAT_ONE_CM` whenever the ridden
  vehicle is an `AbstractBoat`, and `AbstractChestBoat` extends it, so riding a
  poplar chest boat must count.
- Fixed: added `POPLAR_BOAT` and `POPLAR_CHEST_BOAT` to the list.

## Remaining gaps (not fixed)

### 1. Seat offsets ignore the generated `BoatSeats` data (S)
- `entity/vehicle/boat/passengers.rs:67-80` hardcodes `0.15` for chest boats and
  `height / 3.0` otherwise; `entity/vehicle/boat/passengers.rs:53-57` hardcodes
  the `0.2` / `-0.6` multi-rider offsets.
- `entity_type.rs` already carries per-type
  `boat: BoatSeats { ride_height, single_passenger_x_offset }` (poplar: 0.1875 /
  0.15, matching vanilla), so the constants happen to agree for every current
  boat but are not data-driven.
- Vanilla `AbstractBoat#getPassengerAttachmentPoint` / `getPassengersRidingOffset`
  read the entity type attachments. No observable difference today; fix by
  reading `entity_type.boat` when present.

### 2. Boat break drop resolves the item by registry key (S)
- `entity/vehicle/boat.rs:347-357` drops
  `Item::from_registry_key(entity_type.resource_name)`.
- Vanilla `AbstractBoat#getPickResult` returns the item recorded on the entity
  type. Registry key lookup happens to work for every boat including poplar, but
  a renamed or non-matching key would silently drop nothing; using
  `BoatItem::entity_to_item` (already used by pick-block at
  `net/java/play/pick_item.rs:100`) keeps drop and pick-block consistent.

### 3. No automated test for poplar boat coverage (S)
- Nothing pins `POPLAR_BOAT` / `POPLAR_CHEST_BOAT` into `is_chest_boat`,
  `item_to_entity` or `get_movement_statistic`; the same omission class as the
  fixed defect would go unnoticed again. A table test over `BoatItem::ids()`
  asserting `item_to_entity(item).id` round-trips through `entity_to_item`, plus
  a `POPLAR_CHEST_BOAT` `BoatOneCm` assertion, would cover it.
