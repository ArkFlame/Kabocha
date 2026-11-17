# entities / Mangrove Boat re-review (RE-REVIEW-UNTRACKED)

Scope: Rust source for the Mangrove Boat (and its chest variant, shared paths).

Checked paths:
- `crates/pumpkin-core/src/item/items/boat.rs`: `item_to_entity` maps `Item::MANGROVE_BOAT` and `MANGROVE_CHEST_BOAT` to their entity types; both ids are in `ids()`; `entity_to_item` covers the reverse drop mapping.
- `crates/pumpkin-core/src/entity/type.rs:342-343`: both mangrove entity types are built as `BoatEntity`.
- `crates/pumpkin-core/src/entity/vehicle/boat.rs:53`: `MANGROVE_CHEST_BOAT` is in `is_chest_boat`, so it gets the 27-slot inventory and 1 passenger limit; the plain boat gets 2 passengers.
- `crates/pumpkin-core/src/entity/player.rs:4412`: the mangrove boat counts toward the boat movement statistic.

Gaps found: none that are small and certain. The source matches the shared boat behaviour in `docs/VANILLA_REF.md:32`.

Unverified (no `.vendor/` decompiled class was read, and the generated entity dimensions in `crates/pumpkin-data` were not compared to vanilla):
- Mangrove boat entity dimensions and eye height vs vanilla `EntityType` values. Size: S. Fix: compare against decompiled `EntityType.MANGROVE_BOAT` once a reference is available.
- Mangrove-specific sounds and particles, if any differ from other wood types. Size: S.
