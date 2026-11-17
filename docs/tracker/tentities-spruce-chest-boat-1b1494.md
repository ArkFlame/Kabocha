# Audit: entities / Spruce Chest Boat (tentities-spruce-chest-boat-1b1494)

Status: partial. No confirmed gap, no code change.

## Evidence (grep-verified)
- `crates/pumpkin-core/src/entity/vehicle/boat.rs:57` `is_chest_boat` includes `SPRUCE_CHEST_BOAT`.
- `crates/pumpkin-core/src/entity/vehicle/boat.rs:40,43,63` chest boat container size 27 and max passengers 1 apply to spruce via `is_chest_boat`.
- `crates/pumpkin-core/src/entity/type.rs:349` spruce id included in the chest boat helper.
- `crates/pumpkin-core/src/entity/player.rs:4418` spruce included in the vehicle-inventory check.
- `crates/pumpkin-core/src/item/items/boat.rs:26` item to entity mapping; `:76` spruce chest boat item registered in the chest boat list.
- `crates/pumpkin-data/src/generated/entity_type.rs:10916` generated `SPRUCE_CHEST_BOAT` entity type.

## Gaps
None confirmed. Spruce uses the shared chest boat path, so it has no spruce-specific vanilla behaviour to diverge on.

## Not verified
- Full read of `crates/pumpkin-core/src/item/items/boat.rs` and `boat.rs` returned corrupted file-read output during this pass, so per-line review of the placement, use-on-block and drop logic was not completed.
- No vanilla reference under `.vendor/` was available in this worktree to compare against.
- Next pass: re-read `boat.rs` (placement and drop), compare against vanilla `ChestBoat` and `BoatItem`, and list concrete gaps here with file:line, class, fix and size.
