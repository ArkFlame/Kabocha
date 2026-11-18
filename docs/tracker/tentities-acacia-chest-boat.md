# entities / Acacia Chest Boat - audit (RE-REVIEW-UNTRACKED)

Re-audited after the branch was rebased onto master (boat water-physics rewrite
570d3ea1f and the chest-boat fixes baf497b61, 0fe61a1fb, 97ba2480d). Reviewed
crates/pumpkin-core/src/entity/vehicle/boat.rs, boat/{dismount,passengers,physics}.rs,
crates/pumpkin-core/src/item/items/boat.rs, entity/type.rs:332-356,
entity/mod.rs:1238-1265, entity/player.rs:4365-4385,
crates/pumpkin-data/src/generated/entity_type.rs:279-336. No decompiled vanilla tree
in this worktree (.vendor/ absent), so vanilla class behaviour is from knowledge of
1.21.x `AbstractBoat` / `AbstractChestBoat` / `BoatItem`, not 26.3 sources.

## Matches vanilla (no action)
- Entity wiring: type.rs:332-356 routes ACACIA_CHEST_BOAT to BoatEntity; generated
  data (entity_type.rs:279-336) has dimension 1.375x0.5625, eye 0.5625, seat
  ride_height 0.1875, single passenger x offset 0.15 (plain acacia boat uses 0.0,
  matching the chest-boat `getSinglePassengerXOffset()` override) and passenger
  attachment [0, 0.5625, 0].
- 27-slot MinecartInventory only for chest boats (boat.rs:37,72-73,251-266); root
  `Items` NBT plus LootTable via the shared container helper, matching
  `AbstractChestBoat.addAdditionalSaveData`.
- Interact (boat.rs:359-394): mount if a seat is free and not secondary-use, else
  open the container; spectators rejected inside `open_custom_inventory_screen`
  (boat.rs:96-104) - fixed in 0fe61a1fb.
- No CONTAINER_OPEN game event on open (baf497b61); vanilla only emits it from
  container blocks, not container entities.
- Lethal damage: wobble, then `destroy` at damage > 40 or creative; contents
  scatter gated only on doEntityDrops, boat item dropped non-creative with custom
  name copied (boat.rs:307-357) - creative scatter fixed in 97ba2480d.
- Underwater eject counter increments only while `BoatStatus::is_underwater`
  (boat.rs:136-159, 271), i.e. vanilla `outOfControlTicks`; the older
  touching_water-based counter noted by the previous audit is gone.

## Remaining gaps
1. boat.rs:307-357 - contents scatter and boat item drop run only from the damage
   path. Vanilla `AbstractChestBoat.remove(RemovalReason)` drops via
   `chestVehicleDestroyed` for any `reason.shouldDestroy()` path that does not go
   through `hurtServer` (out-of-world teleport, `discard`-based cleanup).
   Suggested fix: hook entity removal with a destroying reason and scatter the
   container there once, keeping the damage path as the item-drop owner.
   Vanilla: `AbstractChestBoat.remove` / `chestVehicleDestroyed`. Size M,
   verification needed against 26.3 sources.
2. boat.rs:344 (also pick_item.rs:100) - the boat item is resolved twice: the drop
   path uses `Item::from_registry_key(entity_type.resource_name)` while the pick
   path uses `BoatItem::entity_to_item`. Vanilla has one mapping
   (`AbstractBoat.getDropItem`/`getPickResult`). A chest-boat entity whose
   resource_name ever diverges from the item key would silently drop nothing.
   Suggested fix: use `BoatItem::entity_to_item` in the drop path. Size S.
3. boat.rs:372 - the container opens when the seat is unavailable for reasons other
   than occupancy (eye in water, just surfaced, player already riding). Vanilla
   `AbstractChestBoat.interact` opens only on secondary use and otherwise delegates
   to `AbstractBoat.interact`; if 26.3 keeps that shape, a plain use while the boat
   is out of control opens the chest here and does nothing in vanilla. Verify before
   changing. Size S.
4. entity/mod.rs:1244-1251 duplicates the chest-boat seat limit as a literal `1`
   instead of reusing the boat.rs:92 rule; a future chest-boat variant missing from
   `is_chest_boat` would take two mob passengers. Suggested fix: one
   `max_passengers(&EntityType)` helper used by both sites. Size S, hygiene only.
5. entity/player.rs:4365-4385 re-enumerates every boat and chest boat although the
   `minecraft:boat` tag check above it already covers them. Suggested fix: drop the
   redundant type list. Size S, hygiene only.
6. No automated coverage for chest-boat contents: add a pumpkin-gametest that spawns
   an acacia chest boat, fills a slot, breaks it with a player and asserts contents
   plus boat item (and no item on creative break). Size M.

## Notes
- No runtime/client repro was run (worker constraint: no cargo). Gaps 1-3 are
  vanilla-source questions, 4-6 are hygiene/coverage.
- Client-predicted boat motion, seat rendering and paddle animation are out of
  scope for this row.

## h6 gap status

- Rows 1-5: FIXED (verified against decompiled vanilla; see Result sections).
- Row 6: STILL OPEN (pumpkin-gametest needs a new file and a build run, both forbidden
  under this worker's no-cargo/no-new-file constraints; pure unit seams cover rows 2-5).


## Result

Re-verified every row against `.vendor/vanilla-decompiled/net/minecraft/`
(`AbstractBoat`, `AbstractChestBoat`, `VehicleEntity`, `BoatItem`). Rows 1, 2, 4 and 5 were
already fixed in the checkout by the time of this pass; row 3 was fixed here.

1. FIXED (pre-existing). `BoatEntity::on_removed` calls `scatter_chest_contents`, guarded by
   `claim_drops`. Verified `World::remove_entity` calls `on_removed` only with
   `RemovalReason::Discarded` (`should_destroy() == true`); chunk-unload and dimension-transfer
   paths skip it, matching `AbstractChestBoat.remove`'s `reason.shouldDestroy()` gate. Vanilla
   `VehicleEntity.hurtServer` runs `destroy` -> `chestVehicleDestroyed`, which the damage path
   already mirrors. No new code needed.
2. FIXED (pre-existing). `boat_drop_item` uses `BoatItem::entity_to_item` first with a registry
   fallback; `every_boat_entity_type_resolves_a_drop_item` covers all boat types. Vanilla has the
   single `dropItem` supplier used by both `getDropItem` and `getPickResult`. No new code needed.
3. FIXED (this pass). Vanilla `AbstractBoat.canAddPassenger` checks only seat count and
   eye-in-water; `outOfControlTicks < 60` and `startRiding` (player already riding) belong to
   `AbstractBoat.interact` and must NOT open the chest. The old inline `can_add_passenger`
   folded all four together, so a plain use on an out-of-control boat or by a mounted player
   opened the container (vanilla returns PASS and does nothing). Extracted a pure
   `boat_interact_outcome` + `BoatInteractOutcome` model, used by `interact`, and added
   `chest_boat_interact_follows_vanilla_mount_container_nothing_chain` covering the full table.
4. FIXED (pre-existing). `boat::max_passengers` is the single rule used by both `boat.rs` and
   `entity/mod.rs:1312`; `chest_boat_detection_covers_every_chest_boat_entity_type` asserts 1 vs 2.
5. FIXED (pre-existing). `player.rs:4443` uses `boat::is_boat(entity_type)` instead of an
   explicit boat/chest-boat list; `boat_stat_classification_covers_every_boat_entity_type` covers it.
6. OPEN. A pumpkin-gametest needs the gametest harness and a build run, neither available under
   this worker's no-cargo/no-new-file constraints. A pure unit test for the interact decision was
   added under row 3 as the smallest in-file seam.

Also repaired a pre-existing compile error in `seat_and_container_tests`: `EntityType` was used
without being imported in that module.

Verification: `rustfmt --edition 2024` applied cleanly to
`crates/pumpkin-core/src/entity/vehicle/boat.rs`. Cargo/clippy not run (worker constraint).


## Result (re-verification pass)

Re-checked every row against `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/`
(`AbstractBoat`, `AbstractChestBoat`) and `world/item/BoatItem.java`, and against the current
Pumpkin `boat.rs`, `entity/mod.rs`, `entity/player.rs` and `world/mod.rs`. No source edit was
needed: rows 1-5 are already correct in the checkout.

1. FIXED. `BoatEntity::on_removed` -> `scatter_chest_contents`, guarded by `claim_drops`. Grep
   confirms `on_removed()` is invoked only from `World::remove_entity`, which sets
   `RemovalReason::Discarded` (`should_destroy() == true`). The damage path also scatters once,
   so contents drop exactly once on break/void/kill/discard. Vanilla
   `AbstractChestBoat.remove`/`destroy` gated on `reason.shouldDestroy()` is matched.
2. FIXED. `boat_drop_item` prefers `BoatItem::entity_to_item` with a registry-key fallback;
   `every_boat_entity_type_resolves_a_drop_item` covers all boat types. Vanilla uses one
   `dropItem` supplier for both `getDropItem` and `getPickResult`.
3. FIXED. Vanilla `AbstractBoat.interact` gates mount behind `isSecondaryUseActive`,
   `outOfControlTicks < 60` and `startRiding`; `canAddPassenger` checks only seat count and
   eye-in-water. `AbstractChestBoat.interact` opens the container only when the ride returned
   PASS and `canAddPassenger(player) && !isSecondaryUseActive()` is false. The pure
   `boat_interact_outcome` model encodes this chain and
   `chest_boat_interact_follows_vanilla_mount_container_nothing_chain` covers the full table,
   so an out-of-control or already-ridden chest boat no longer opens on plain use.
4. FIXED. `max_passengers` is the single seat rule used by `boat.rs` and `entity/mod.rs:1338`;
   `chest_boat_detection_covers_every_chest_boat_entity_type` asserts 1 vs 2.
5. FIXED. `entity/player.rs:4567` uses `boat::is_boat(entity_type)`;
   `boat_stat_classification_covers_every_boat_entity_type` covers every boat type.
6. OPEN (unchanged). A pumpkin-gametest for contents/drop on break needs a new gametest file
   plus a build run; both are forbidden here (no new sources, no cargo). The smallest
   in-file pure seams were added for rows 2-5 instead.

No rustfmt needed: no `.rs` file was touched this pass. Cargo/clippy not run (worker constraint).

## Result (final re-review)

Read-only verification against `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractChestBoat.java`
and the current Pumpkin `crates/pumpkin-core/src/entity/vehicle/boat.rs`. No source file was
touched: this unit names no source basenames and the only open row needs a new file.

- Rows 1-5: already FIXED in the checkout. `on_removed` -> `scatter_chest_contents` gated by
  `claim_drops` (row 1); `boat_drop_item` prefers `BoatItem::entity_to_item` (row 2);
  `boat_interact_outcome` models the vanilla mount/container/nothing chain (row 3);
  `max_passengers` is the single seat rule used by both sites (row 4); `player.rs` uses
  `boat::is_boat` (row 5). Confirmed present by grep; no edit needed.
- Row 6: STILL OPEN. Requires a new pumpkin-gametest source file plus a build/run; a new file
  outside this unit's permitted sources and cargo are both forbidden under this worker's
  constraints. The pure seams a unit test could cover (`drops_item_when_destroyed`,
  `boat_drop_item`, `boat_interact_outcome`, including no item on creative break) already have
  unit tests in `boat.rs`; the remaining integration assertion (spawn, fill, break, inspect world
  drops) has no pure seam.

No `.rs` file touched, so no rustfmt needed. Cargo/clippy not run (worker constraint).
