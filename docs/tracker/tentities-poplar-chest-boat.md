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

## Gap status
- Remaining-gap items (section lines 9+): STILL OPEN (unverified: no .vendor decompile in h6; not changed)


## Result

Verified on 2026-11-17 against decompiled vanilla at .vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/ (AbstractBoat.java, AbstractChestBoat.java) and VehicleEntity.java.

- Row "interact opens container only when sneaking / full boat returns PASS" (doc line 13): NO GAP, already correct. AbstractChestBoat.interact returns PASS only when super.interact is PASS AND canAddPassenger(player) && !isSecondaryUseActive(); a full chest boat has canAddPassenger == false, so it DOES open the container on a plain click. Current chest_opens_container = has_inventory && (is_sneaking || !can_add_passenger) matches vanilla exactly; the doc premise was inverted. No code change.

- Row "contents dropped on every destroying removal, not only damage break" (doc line 14): NO GAP, already correct. Vanilla AbstractChestBoat.remove scatters when reason.shouldDestroy() and destroy scatters on break. Current EntityBase::on_removed (invoked from World::remove_entity with RemovalReason::Discarded) scatters via scatter_chest_contents, damage_with_context also scatters on break, claim_drops prevents a double scatter, and EntityBase::kill scatters container inventory for /kill. Unload paths set a non-destroying removal reason without calling on_removed, matching vanilla. No code change.

- Row "drop extra CONTAINER_OPEN game event" (doc line 15): NO GAP, already correct. Decompiled AbstractChestBoat.interact AND openCustomInventoryScreen both emit gameEvent(GameEvent.CONTAINER_OPEN, player) and call PiglinAi.angerNearbyPiglins. Current open_custom_inventory_screen emits ContainerOpen with the player as source and calls PiglinAi::on_guarded_block_interacted; matches vanilla. No code change.

- Row "c:boats missing poplar_chest_boat" (doc line 16): STILL OPEN, needs codegen. The value lives in generated crates/pumpkin-data/src/generated/tag.rs, which must not be hand-edited; regenerate with cargo run --locked -p pumpkin-codegen. No functional gap remains because is_boat() already covers poplar_chest_boat through is_chest_boat (the _chest_boat suffix).

- Row "passenger offsets use generated seat data" (doc line 17): NO GAP, already correct. boat/passengers.rs delegates to super::passenger_attachment, which reads the generated entity.entity_type.boat seat data (ride_height, single_passenger_x_offset); the local is_chest_boat/bamboo branches are gone. No code change.

Also corrected the stale inline comment in boat.rs about the instabuild drop path to match verified vanilla VehicleEntity.hurtServer (instabuild discards directly and drops nothing) and the existing function doc comment. No new unit test: boat.rs already covers these seams (chest_boat_offers_container_only_when_no_seat_or_secondary_use, boat_seat_offset_follows_vanilla_single_multi_rider_and_animal_shift, named_boat_keeps_its_drop_when_creative, chest_boat_detection_covers_every_chest_boat_entity_type), so adding tests would duplicate them.
