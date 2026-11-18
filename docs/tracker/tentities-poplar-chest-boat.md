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
- Row 'interact opens container only when sneaking / full boat returns PASS' (line 13): CLOSED (NO GAP; re-verified 2026-11-18)
- Row 'contents dropped on every destroying removal, not only damage break' (line 14): CLOSED (NO GAP; re-verified 2026-11-18)
- Row 'drop extra CONTAINER_OPEN game event' (line 15): CLOSED (NO GAP; re-verified 2026-11-18)
- Row 'c:boats missing poplar_chest_boat' (line 16): OPEN (needs pumpkin-codegen regeneration; generated data out of scope)
- Row 'passenger offsets use generated seat data' (line 17): CLOSED (NO GAP; re-verified 2026-11-18)


## Result

Verified on 2026-11-17 against decompiled vanilla at .vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/ (AbstractBoat.java, AbstractChestBoat.java) and VehicleEntity.java.

- Row "interact opens container only when sneaking / full boat returns PASS" (doc line 13): NO GAP, already correct. AbstractChestBoat.interact returns PASS only when super.interact is PASS AND canAddPassenger(player) && !isSecondaryUseActive(); a full chest boat has canAddPassenger == false, so it DOES open the container on a plain click. Current chest_opens_container = has_inventory && (is_sneaking || !can_add_passenger) matches vanilla exactly; the doc premise was inverted. No code change.

- Row "contents dropped on every destroying removal, not only damage break" (doc line 14): NO GAP, already correct. Vanilla AbstractChestBoat.remove scatters when reason.shouldDestroy() and destroy scatters on break. Current EntityBase::on_removed (invoked from World::remove_entity with RemovalReason::Discarded) scatters via scatter_chest_contents, damage_with_context also scatters on break, claim_drops prevents a double scatter, and EntityBase::kill scatters container inventory for /kill. Unload paths set a non-destroying removal reason without calling on_removed, matching vanilla. No code change.

- Row "drop extra CONTAINER_OPEN game event" (doc line 15): NO GAP, already correct. Decompiled AbstractChestBoat.interact AND openCustomInventoryScreen both emit gameEvent(GameEvent.CONTAINER_OPEN, player) and call PiglinAi.angerNearbyPiglins. Current open_custom_inventory_screen emits ContainerOpen with the player as source and calls PiglinAi::on_guarded_block_interacted; matches vanilla. No code change.

- Row "c:boats missing poplar_chest_boat" (doc line 16): STILL OPEN, needs codegen. The value lives in generated crates/pumpkin-data/src/generated/tag.rs, which must not be hand-edited; regenerate with cargo run --locked -p pumpkin-codegen. No functional gap remains because is_boat() already covers poplar_chest_boat through is_chest_boat (the _chest_boat suffix).

- Row "passenger offsets use generated seat data" (doc line 17): NO GAP, already correct. boat/passengers.rs delegates to super::passenger_attachment, which reads the generated entity.entity_type.boat seat data (ride_height, single_passenger_x_offset); the local is_chest_boat/bamboo branches are gone. No code change.

Also corrected the stale inline comment in boat.rs about the instabuild drop path to match verified vanilla VehicleEntity.hurtServer (instabuild discards directly and drops nothing) and the existing function doc comment. No new unit test: boat.rs already covers these seams (chest_boat_offers_container_only_when_no_seat_or_secondary_use, boat_seat_offset_follows_vanilla_single_multi_rider_and_animal_shift, named_boat_keeps_its_drop_when_creative, chest_boat_detection_covers_every_chest_boat_entity_type), so adding tests would duplicate them.


## Result (re-verified 2026-11-18)

Re-checked every STILL OPEN row against decompiled vanilla at .vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractChestBoat.java and the current Pumpkin source. The 2026-11-17 Result section still holds; no code changes were needed.

- Line 13 interact: CLOSED, NO GAP. AbstractChestBoat.interact (Java 80-97) opens the container iff super.interact returned PASS and !(canAddPassenger && !isSecondaryUseActive). boat.rs chest_opens_container (135-141) = has_inventory && (is_sneaking || !can_add_passenger), wired through boat_interact_outcome (161-177) from interact (813-825). Matches for seat free, full boat, sneaking, out-of-control and already-riding cases. Existing unit tests boat_interact_outcome / chest_opens_container (1191-1200) cover this seam.

- Line 14 drops: CLOSED, NO GAP. AbstractChestBoat.remove (Java 71-77) scatters on any reason.shouldDestroy() and destroy (Java 65-68) scatters on break. boat.rs on_removed (488-490) calls scatter_chest_contents (272-288) for destroying removals, damage_with_context (691-755) scatters on break, and claim_drops (276) prevents a double scatter. Matches.

- Line 15 CONTAINER_OPEN: CLOSED, NO GAP. AbstractChestBoat.interact (Java 91-93) and openCustomInventoryScreen (Java 100-106) both emit CONTAINER_OPEN with the player and anger piglins. boat.rs open_custom_inventory_screen (290-328) emits ContainerOpen with the player as source and calls PiglinAi::on_guarded_block_interacted. Matches.

- Line 16 c:boats: STILL OPEN, out of scope. poplar_chest_boat is missing from the generated c:boats tag in crates/pumpkin-data/src/generated/tag.rs; generated data must be regenerated with `cargo run --locked -p pumpkin-codegen`, not hand-edited. No functional gap: boat.rs is_boat (line 89-91) is `has_tag(C_BOATS) || is_chest_boat(...)`, and is_chest_boat (76-79) matches the _chest_boat suffix, so poplar_chest_boat is already recognised.

- Line 17 passenger offsets: CLOSED, NO GAP. boat.rs passenger_attachment (204-222) reads the generated entity.entity_type.boat seat data (ride_height, single_passenger_x_offset) and passenger_seat_offset (181-200) mirrors AbstractBoat.getPassengerAttachmentPoint. No local chest/bamboo branch remains.

No new unit test: the pure helpers boat_interact_outcome, chest_opens_container and passenger_seat_offset already have direct tests (1191-1200); adding more would duplicate them. No .rs file touched, so rustfmt is not applicable.

## Result (dismount.rs re-verification 2026-11-18)

Scope for this unit is `crates/pumpkin-core/src/entity/vehicle/boat/dismount.rs` only. No STILL OPEN row maps to dismount.rs: the only open row (line 16, c:boats missing poplar_chest_boat) needs pumpkin-codegen regeneration of the generated tag.rs and is explicitly out of scope (generated data, no hand-edits). Left open with that one-line reason.

Re-verified dismount.rs against decompiled vanilla at .vendor/vanilla-decompiled/net/minecraft/world/entity/:
- AbstractBoat.getDismountLocationForPassenger (Java 709-738): escape vector from getBbWidth()*SQRT_OF_TWO and passenger yaw; target BlockPos built with boat maxY; water-at-below early return; two floor targets gated by isBlockFloorValid (floor.is_finite() && floor < 1.0); pose order Standing/Crouching/Swimming for players, Standing only for non-players; canDismountTo checks collisions + world border; setPose before returning. dismount.rs:18-74 matches all of these.
- Entity.getCollisionHorizontalEscapeVector (Java 6645-6651): distance=(collider+colliding+1e-5)/2, x=-sin, z=cos, scale=max(|x|,|z|). dismount.rs:77-92 matches exactly (DISMOUNT_EPSILON=1e-5).
- Base fallback Entity.getDismountLocationForPassenger (Java 6653-6655) returns (getX, maxY, getZ); dismount.rs fallback = (pos.x, top, pos.z) at lines 26 and 73 matches.

No gap found, no source edit made. The extra `pose.load() != pose` recheck at line 66 is stricter than vanilla (retries the next pose if a setPose is rejected) but harmless. No new unit test: no in-scope row was fixed and the pure helpers are unchanged; a test would duplicate existing coverage.

## Result (2026-11-18, dismount unit)

Reworked `crates/pumpkin-core/src/entity/vehicle/boat/dismount.rs` against
`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java:709-738`,
`Entity.java:6645-6655` and `DismountHelper.java:38-54`.

- FIXED: the dismount search only used `get_dimensions(pose)` for players and fell back to
  `entity_dimension.load()` (the rider's current, riding pose box) for mobs. Vanilla
  `DismountHelper.canDismountTo` always tests `passenger.getLocalBoundsForPose(pose)` ->
  `LivingEntity.getDimensions(pose)`, so every rider now tests the pose box.
- FIXED: the pose list is now the vanilla `getDismountPoses()` shape (`LivingEntity` =
  `[Standing]`, `Player` = `[Standing, Crouching, Swimming]`) via a pure `dismount_poses`
  helper instead of the `take(if is_player { 3 } else { 1 })` count hack.
- Targets are collected once into a `Vec` so the pose/target loops mirror vanilla's
  `targets` list ordering (above block first, then the block below).
- Added two pure unit tests: `only_players_offer_the_crouching_and_swimming_dismount_poses`
  and `escape_vector_moves_half_the_two_widths_plus_epsilon_along_the_heading`
  (vanilla `getCollisionHorizontalEscapeVector` distances for yaw 0 and 90).
- Already correct, unchanged: water short-circuit uses `minecraft:water` on
  `target_pos.down()`, `getBlockFloorHeight` validity is `is_finite() && < 1.0`,
  escape vector scale division, world-border epsilon check.
- Row "c:boats missing poplar_chest_boat" (line 16): STILL OPEN, needs
  `cargo run --locked -p pumpkin-codegen`; generated data must not be hand-edited.
- Not changed on purpose: vanilla calls `passenger.setPose(dismountPose)` for every rider,
  but Pumpkin's `set_pose` also refreshes dimensions and fires a cancellable pose event, so
  the set stays player-only; a mob rider is already `Standing`, so behaviour is the same.
