# entities / Poplar Chest Boat (RE-REVIEW)

Reviewed: `crates/pumpkin-core/src/entity/vehicle/boat.rs`, `boat/passengers.rs`, `boat/dismount.rs`, `item/items/boat.rs`, `entity/type.rs`, `entity/player.rs:4361`, `pumpkin-data` generated data for `poplar_boat` / `poplar_chest_boat`.

Already covered (no gap): chest-boat detection includes `POPLAR_CHEST_BOAT` (`boat.rs:53`), boat entity construction (`entity/type.rs:352-353`), item to entity mapping and item ids (`item/items/boat.rs:41-42,91-92`), container size 27 and NBT persistence (`boat.rs:75,235-245`), drop of boat item and contents on damage break (`boat.rs:332-357`), pickup via `getPickResult` (`net/java/play/pick_item.rs:100`, `item/items/boat.rs:53-58`), and generated seat data (`entity_type.rs:9238`, `boat: Some { ride_height 0.1875, single_passenger_x_offset 0.15 }`).

Fixed: `Player::get_movement_statistic` listed every boat wood type except poplar, so riding a poplar boat or poplar chest boat did not increment `BoatOneCm` (`player.rs:4361`). Added `POPLAR_BOAT` / `POPLAR_CHEST_BOAT`. Commit: fix(entity): count poplar boats in boat movement stat.

## Remaining gaps

| file:line | vanilla | suggested fix | size |
|---|---|---|---|
| boat.rs:377 | `AbstractChestBoat.interact`: opens the container only when `player.isSecondaryUseActive()` (sneak); a full/occupied chest boat returns PASS and does not open | open the inventory only when the player is sneaking, and treat the not-sneaking case as "cannot mount" instead of opening | S (verify) |
| boat.rs:332-357 | `AbstractChestBoat.destroy`: contents are dropped whenever the chest boat is destroyed (void, `/kill`, discard), not only on the damage break path | move the `claim_drops`/`scatter_inventory` block into a destroy/removal hook that runs for every removal | M (verify) |
| boat.rs:113-122 | vanilla `AbstractChestBoat` opens the menu without a `GameEvent.CONTAINER_OPEN` emission | drop the extra `ContainerOpen` game event once confirmed against decompiled source | S (verify) |
| pumpkin-data `tag.rs:20175` (`C_BOATS`) | `c:boats` should contain `poplar_chest_boat` (it is missing while `minecraft:boats` has it) | regenerate tags from `assets/` (`cargo run --locked -p pumpkin-codegen`); do not hand-edit generated data | S (needs codegen) |
| boat/passengers.rs:67-69, 71-80 | passenger offsets/ride height come from `AbstractChestBoat`/`Boat` entity attachments and are data driven | replace the local `is_chest_boat()` / bamboo branches with the generated `entity_type.boat` seat data used at `boat.rs:197` | S |
