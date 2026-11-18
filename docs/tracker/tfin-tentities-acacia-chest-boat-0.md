# Final rows from docs/tracker/tentities-acacia-chest-boat.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files:
All six rows are closed below. Verified against decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractChestBoat.java`
(and `AbstractBoat.java`, `VehicleEntity.java` in the same folder, and
`.vendor/vanilla-decompiled/net/minecraft/world/item/BoatItem.java`).

## Result (final pass)

No source edit was needed this pass: every fix and its test coverage is already present in the
checkout and matches the vanilla source above.

1. FIXED, VERIFIED. Vanilla `AbstractChestBoat.remove` scatters the 27 slots on any
   `reason.shouldDestroy()` removal (`AbstractChestBoat.java:71-77`) and `destroy` runs
   `chestVehicleDestroyed` (`AbstractChestBoat.java:65-68`). Pumpkin `BoatEntity::on_removed`
   calls `scatter_chest_contents` gated by `removal_reason.should_destroy()` and
   `MinecartInventory::claim_drops` (`boat.rs:270-288`, `490-505`); the damage path scatters too
   (`boat.rs:738-762`). `chest_contents_scatter_claim_is_single_use` covers single-scatter.
2. FIXED, VERIFIED. Vanilla uses one `dropItem` supplier for `getDropItem`/`getPickResult`
   (`AbstractChestBoat.java:38-40`). Pumpkin `boat_drop_item` prefers
   `BoatItem::entity_to_item` with a registry fallback (`boat.rs:103-109`);
   `every_boat_entity_type_resolves_a_drop_item` covers every boat type.
3. FIXED, VERIFIED. Vanilla `AbstractBoat.interact` gates mounting on secondary use,
   out-of-control and existing vehicle; `AbstractChestBoat.interact` opens only when the super
   result is PASS and NOT (`canAddPassenger(player) && !isSecondaryUseActive()`)
   (`AbstractChestBoat.java:80-97`). Pumpkin `boat_interact_outcome` + `chest_opens_container`
   mirror the chain (`boat.rs:133-177`, `812-835`), covered by
   `chest_boat_interact_follows_vanilla_mount_container_nothing_chain`.
4. FIXED, VERIFIED. Vanilla `AbstractChestBoat.getMaxPassengers` is 1
   (`AbstractChestBoat.java:48-50`) vs `AbstractBoat` 2. Pumpkin `boat::max_passengers` is the
   single seat rule used by `boat.rs:822` and `entity/mod.rs:1354`;
   `chest_boat_detection_covers_every_chest_boat_entity_type` asserts 1 vs 2.
5. FIXED, VERIFIED. Pumpkin `entity/player.rs:4582` uses `boat::is_boat(entity_type)`;
   `boat_stat_classification_covers_every_boat_entity_type` covers every boat/chest-boat type.
6. FIXED, VERIFIED. The pumpkin-gametest `pumpkin:acacia_chest_boat_contents_and_item_drop`
   exists at `crates/pumpkin-core/src/server/builtin_game_tests/boat_block_effects.rs:284-335`:
   it spawns an acacia chest boat, fills three slots via summon NBT, breaks it with `/damage`,
   then asserts entity removal, three scattered diamonds and the dropped acacia chest boat item.
   It is listed in `BUILTIN_TEST_IDS` (`mod.rs:279`) and registered in
   `ensure_builtin_game_tests_registered` (`mod.rs:1862-1866`). The creative-break half is covered
   by the pure `creative_boat_drops_no_item_even_when_named` unit test (`boat.rs:1137`).

No `.rs` file touched this pass, so no rustfmt needed; cargo/clippy not run (worker constraint).
