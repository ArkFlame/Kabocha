# entities / Mangrove Chest Boat (RE-REVIEW)

Reviewed: `crates/pumpkin-core/src/entity/vehicle/boat.rs`, `crates/pumpkin-core/src/item/items/boat.rs`,
`crates/pumpkin-core/src/entity/vehicle/boat/passengers.rs`, `crates/pumpkin-core/src/entity/mod.rs:1236-1250`.
No decompiled vanilla tree present (no `.vendor/`); parity judged against vanilla
`net.minecraft.world.entity.vehicle.AbstractBoat` / `AbstractChestBoat` / `ChestBoat` / `BoatItem`.

## Matches vanilla (no action)

- `is_chest_boat` (boat.rs:43-55) lists all 11 chest boats/rafts in `pumpkin-data`
  (`entity_type.rs` resource names match `item.rs` registry keys exactly), 27-slot container
  (`CHEST_BOAT_CONTAINER_SIZE`), one passenger (`get_max_passengers`).
- `BoatItem::item_to_entity`/`ids` (item/items/boat.rs:36,86) map `minecraft:mangrove_chest_boat`
  to `MANGROVE_CHEST_BOAT`; `entity_to_item` gives the pick-block result; placement copies the
  custom name and consumes one item (non-creative).
- Seat data: `boat.rs:191-223` and passengers.rs:67-69 use ride height 0.1875 and single-passenger
  x offset 0.15, matching generated `BoatSeats` for `mangrove_chest_boat`.
- Interact order (mount unless secondary use/full, else open container; no loot tables on boats),
  wobble, destroy at damage > 40 or instabuild, item drop with `CustomName`, container NBT at
  `Items` root (`MinecartInventory::write_nbt`), `CONTAINER_OPEN` game event, 9x3 menu.
- `tick_underwater` uses `BoatStatus::UNDER_WATER` (boat.rs:144-167), not raw water overlap.

## Fixed in this commit

- Creative break did not scatter the chest contents: the whole drop block was gated on `!creative`.
  Vanilla `ChestBoat.remove` (destroying reason) calls `chestVehicleDestroyed` gated only by
  `RULE_DOENTITYDROPS`; only the boat item drop is instabuild-suppressed. boat.rs:332-360 now scatters
  contents for instabuild breaks too. Commit: fix(entity): drop chest boat contents on creative break.

## Remaining gaps

| file:line | vanilla | suggested fix | size |
|---|---|---|---|
| boat.rs:315-362 | `ChestBoat.remove(Reason)` scatters contents for any destroying removal (`/kill`, out-of-world, `discard`), not only the damage path | add a removal hook on `BoatEntity` (or `EntityBase`) that claims the container on destroying removals and scatters it | M |
| boat.rs:98-125 | `AbstractChestBoat.openCustomInventoryScreen` requires `(!isVehicle() \|\| hasPassenger(player)) && isAlive()` | reject opening when the boat is ridden by another entity or is dead | S |
| boat.rs:115-123 | `openCustomInventoryScreen` awards `Stats.OPEN_CHEST` | award the open-chest statistic; blocked on a custom-stat id registry (pumpkin-data has only item-based categories) | M |
| entity/mod.rs:1238-1243 | one source of truth for `getMaxPassengers()` | reuse a `BoatEntity::max_passengers` helper instead of the duplicated chest-boat literal; a new chest boat type missing from `is_chest_boat` would seat two mobs | S |
| boat.rs:365-400 | `AbstractChestBoat.interact` also returns a deciding result when the mount fails (out of control / full) so the held item is not used | confirm chest-boat interact always returns true once it owns the click; currently returns false when sneaking on a non-chest boat only | S |

## Notes

- Runtime behaviour of the remaining gaps was not observed (no client/world repro); they come from
  reading the vanilla call chain. Generic boat physics/motion is covered by the Mangrove Boat row.

## Gap status
- 1 ChestBoat.remove scatter on any destroying removal (boat.rs:488-505): FIXED, VERIFIED against `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractChestBoat.java:71-77` (remove scatters on every `reason.shouldDestroy()`); `on_removed` (boat.rs:488) calls `scatter_chest_contents` (boat.rs:272-288), guarded by `claim_drops` (minecart/container.rs:39).
- 2 shared getMaxPassengers helper (entity/mod.rs:1350-1357): FIXED, VERIFIED against `AbstractChestBoat.java:47-50`; `boat.rs::max_passengers` (boat.rs:95-101) is reused by the mob-push mounting path. No behaviour change.
- 3 interact deciding result on failed mount (boat.rs:767-851): FIXED, VERIFIED against `AbstractChestBoat.java:80-97` and `AbstractBoat.java:779-788`; pure `chest_opens_container`/`boat_interact_outcome` seam (boat.rs:135-177) with unit tests in the boat test module.


## Result

Re-reviewed against the decompiled vanilla tree at
`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/`.

- Gap 1 (chest contents scatter on any destroying removal): VERIFIED, no new edit needed.
  `AbstractChestBoat.remove` (AbstractChestBoat.java:71-77) drops contents whenever
  `reason.shouldDestroy()`, and `ContainerEntity.chestVehicleDestroyed` (:84-91) gates only the
  damage path on `doEntityDrops`. Pumpkin covers this through `BoatEntity::on_removed`
  (boat.rs:266-268) calling `scatter_chest_contents` (`:183-201`, `claim_drops` guards against a
  second scatter), plus the explicit scatter in `damage_with_context` (`:487-491`). `World::remove_entity`
  (world/mod.rs:6410-6423) invokes `on_removed` for kill, void and discard, so the removal hook fires
  on every destroying path.
- Gap 2 (shared `getMaxPassengers` helper): LEFT OPEN. `boat.rs::max_passengers` (boat.rs:74) is
  already the single source of truth used by the boat, but the duplicated literal this row targets
  lives in `entity/mod.rs:1238-1243`, which is outside this unit's editable files. No behaviour
  change identified.
- Gap 3 (interact deciding result on a failed mount): FIXED. `AbstractChestBoat.interact`
  (AbstractChestBoat.java:80-97) plus `AbstractBoat.interact` (:779-788) and `canAddPassenger`
  (:836-838) open the container when the ride attempt did not consume the click. The new pure seam
  `chest_opens_container` (boat.rs:96) drives `interact` (`:639-643`) and is covered by a pure unit test in the boat test module.
  Also removed the rider/occupied
  rejection from `open_custom_inventory_screen`: the decompiled
  `AbstractChestBoat.openCustomInventoryScreen` has no such check (the
  `(!isVehicle() || hasPassenger(player))` rule belongs to `AbstractHorse`), and `player.openMenu`
  succeeds while another player rides, so a full chest boat must still offer its container.

## Result (re-review 2)

Re-verified every STILL OPEN row against `.vendor/vanilla-decompiled/net/minecraft/`
and the current Pumpkin source. All three are already fixed in the tree; no source
edits were needed this pass (this unit had no editable basenames).

- 1 chest contents scatter on any destroying removal: FIXED and verified.
  `AbstractChestBoat.remove` (AbstractChestBoat.java:71-77) drops contents for every
  `reason.shouldDestroy()`. Pumpkin `BoatEntity::on_removed` (boat.rs:488) calls
  `scatter_chest_contents` (boat.rs:272), guarded by `MinecartInventory::claim_drops`
  (minecart/container.rs:39) so the scatter fires at most once; `World::remove_entity`
  (world/mod.rs:6649) invokes `on_removed` for kill, void and discard. The damage path
  scatters explicitly too, matching `AbstractChestBoat.destroy` (:65-68) and
  `ContainerEntity.chestVehicleDestroyed` (:84-91), which gates the damage drop on
  `doEntityDrops`.
- 2 shared `getMaxPassengers` helper: FIXED (corrects the prior "LEFT OPEN" note).
  `boat.rs::max_passengers` (boat.rs:95) is now the single source of truth and is reused
  by the mob-push mounting path in `entity/mod.rs:1337-1338`, replacing the duplicated
  chest-boat literal. Matches vanilla `AbstractBoat.getMaxPassengers` = 2 (:840-842)
  and `AbstractChestBoat.getMaxPassengers` = 1 (:47-50). No behaviour change.
- 3 interact deciding result on a failed mount: FIXED and verified.
  `AbstractBoat.interact` (:779-788), `canAddPassenger` (:836-838) and
  `AbstractChestBoat.interact` (:80-97) offer the container whenever the ride did not
  consume the click. The pure `chest_opens_container` (boat.rs:135) drives
  `boat_interact_outcome` (boat.rs:161) used by `interact` (boat.rs:807), so an occupied
  boat still opens its container; covered by the pure unit test
  `chest_boat_offers_container_only_when_no_seat_or_secondary_use` (boat.rs:1189).
  `open_custom_inventory_screen` (boat.rs:290) keeps only the alive guard, matching
  vanilla's lack of a rider/spectator check.

## Result (re-review 3, 2026-11-18)

Re-verified the three STILL OPEN rows against
`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractChestBoat.java`
and the current Pumpkin source via read/grep only. This unit declares no editable source
basenames, so no code edits were possible; every row is already fixed in the tree. No
cargo run was performed (workers may not run cargo).

- 1 chest contents scatter on any destroying removal: ALREADY FIXED, verified.
  `AbstractChestBoat.remove` (AbstractChestBoat.java:71-77) drops contents for every
  `reason.shouldDestroy()`. `BoatEntity::on_removed` (boat.rs:488) calls
  `scatter_chest_contents` (boat.rs:272), with `claim_drops` guarding a second scatter; the
  damage path scatters too (boat.rs:730). Matches vanilla `destroy` (:65-68) and
  `ContainerEntity.chestVehicleDestroyed`.
- 2 shared `getMaxPassengers` helper: ALREADY FIXED, verified.
  `boat.rs::max_passengers` (boat.rs:95) is the single source of truth
  (`CHEST_BOAT_MAX_PASSENGERS` = 1, `BOAT_MAX_PASSENGERS` = 2) and is reused by the mob-push
  mounting path (entity/mod.rs:1337-1338). Matches `AbstractChestBoat.getMaxPassengers`
  (:47-50). No behaviour change.
- 3 interact deciding result on a failed mount: ALREADY FIXED, verified.
  `AbstractChestBoat.interact` (:80-97) offers the container when the ride did not consume
  the click. The pure `chest_opens_container` (boat.rs:135) feeds `boat_interact_outcome`
  (boat.rs:161), used by `interact` (boat.rs:807), with pure unit tests in the boat test
  module (boat.rs:1189-1258). `open_custom_inventory_screen` keeps only the alive guard,
  matching vanilla (:100-106), which has no rider/spectator check.

Status: PASS fixed=0 open=0 files=none (no editable basenames; all STILL OPEN rows already fixed; no cargo run allowed for workers)


## Result (re-review 4, 2026-11-18)

Re-verified the three STILL OPEN rows against the decompiled vanilla tree
(`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractChestBoat.java`)
and the current Pumpkin source via read/grep only. This unit declares no editable source
basenames, so no code edits were possible; every row is already fixed in the tree. No
cargo run was performed (workers may not run cargo).

- 1 chest contents scatter on any destroying removal: ALREADY FIXED, verified.
  `AbstractChestBoat.remove` (AbstractChestBoat.java:71-77) drops contents for every
  `reason.shouldDestroy()`. Pumpkin `BoatEntity::on_removed` (boat.rs:488-490) calls
  `scatter_chest_contents` (boat.rs:272-288), guarded by `claim_drops`
  (minecart/container.rs:39) so the scatter fires at most once; the damage path scatters
  too (boat.rs:730). `World::remove_entity` (world/mod.rs:6649) invokes `on_removed` for
  kill, void and discard. Matches vanilla `destroy` (:65-68) and
  `ContainerEntity.chestVehicleDestroyed`.
- 2 shared `getMaxPassengers` helper: ALREADY FIXED, verified.
  `boat.rs::max_passengers` (boat.rs:95-101) is the single source of truth
  (`CHEST_BOAT_MAX_PASSENGERS` = 1, `BOAT_MAX_PASSENGERS` = 2) and is reused by the
  mob-push mounting path (entity/mod.rs:1337-1344). Matches
  `AbstractChestBoat.getMaxPassengers` (:47-50). No behaviour change.
- 3 interact deciding result on a failed mount: ALREADY FIXED, verified.
  `AbstractChestBoat.interact` (:80-97) offers the container when the ride did not consume
  the click. The pure `chest_opens_container` (boat.rs:135-141) feeds `boat_interact_outcome`
  (boat.rs:161-177), used by `interact` (boat.rs:807-816), with pure unit tests in the boat
  test module (boat.rs:1189-1258). `open_custom_inventory_screen` (boat.rs:290) keeps only
  the alive guard, matching vanilla (:100-106), which has no rider/spectator check.

Status: PASS fixed=0 open=0 files=none

## Result (third pass, 2026-11-18)

Re-verified every remaining STILL OPEN row against the decompiled vanilla tree at
`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractChestBoat.java`
and the current Pumpkin source via read/grep only. All three rows are already fixed in the
tree; no source edits were needed. No cargo run was performed (workers may not run cargo).

- 1 chest contents scatter on any destroying removal: FIXED, VERIFIED.
  `AbstractChestBoat.remove` (AbstractChestBoat.java:71-77) drops contents for every
  `reason.shouldDestroy()`. Pumpkin `BoatEntity::on_removed` (boat.rs:488-505) checks
  `removal_reason.should_destroy()` and calls `scatter_chest_contents` (boat.rs:272-288),
  with `claim_drops` (minecart/container.rs:39) making the scatter run at most once; the damage
  path scatters too (matching `destroy` :65-68).
- 2 shared `getMaxPassengers` helper: FIXED, VERIFIED.
  `AbstractChestBoat.getMaxPassengers` returns 1 (:47-50). `boat.rs::max_passengers`
  (boat.rs:95-101) is the single source of truth and is reused by the mob-push mounting path
  (entity/mod.rs:1350-1357). No behaviour change.
- 3 interact deciding result on a failed mount: FIXED, VERIFIED.
  `AbstractChestBoat.interact` (:80-97) offers the container when the ride did not consume the
  click. The pure `chest_opens_container` (boat.rs:135-141) feeds `boat_interact_outcome`
  (boat.rs:161-177), used by `interact` (boat.rs:823-835); covered by pure unit tests in the
  boat test module. `open_custom_inventory_screen` (boat.rs:290) keeps only the alive guard,
  matching vanilla (:100-106), which has no rider/spectator check.

Updated the stale Gap status table so no row remains STILL OPEN.

Status: PASS fixed=0 open=0 files=docs/tracker/tentities-mangrove-chest-boat.md (all rows already fixed; doc status synced; no cargo run allowed for workers)


## Result (final verification, 2026-11-18)

Re-read the three rows against the current decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractChestBoat.java`
and the current Pumpkin source. This unit declares no editable source basenames; every row
is already FIXED in the tree, so no source edits were required. No cargo/git/shell run was
performed (workers may not run them).

- 1 chest contents scatter on any destroying removal: FIXED, VERIFIED.
  `AbstractChestBoat.remove` (AbstractChestBoat.java:71-77) drops contents for every
  `reason.shouldDestroy()`. Pumpkin `BoatEntity::on_removed` (boat.rs:488-506) checks
  `removal_reason.should_destroy()` and calls `scatter_chest_contents` (boat.rs:272-288),
  guarded by `claim_drops` so the scatter runs at most once; the damage path scatters too
  (boat.rs:746), matching `destroy` (:65-68) and `ContainerEntity.chestVehicleDestroyed`
  (:84-91), which gates only the boat-item drop on `doEntityDrops`.
- 2 shared `getMaxPassengers` helper: FIXED, VERIFIED.
  `AbstractChestBoat.getMaxPassengers` returns 1 (:47-50). `boat.rs::max_passengers`
  (boat.rs:95-101) is the single source of truth (`CHEST_BOAT_MAX_PASSENGERS` = 1,
  `BOAT_MAX_PASSENGERS` = 2) and is reused by the mob-push mounting path
  (entity/mod.rs:1350-1357). No behaviour change.
- 3 interact deciding result on a failed mount: FIXED, VERIFIED.
  `AbstractChestBoat.interact` (:80-97) offers the container when the ride did not consume
  the click. The pure `chest_opens_container` (boat.rs:135-141) feeds `boat_interact_outcome`
  (boat.rs:161-177), used by `interact` (boat.rs:823-835), with pure unit tests in the boat
  test module (boat.rs:1217-1285). `open_custom_inventory_screen` (boat.rs:290) keeps only
  the alive guard, matching vanilla (:100-106), which has no rider/spectator check.

No STILL OPEN rows remain.

Status: PASS fixed=0 open=0 files=docs/tracker/tentities-mangrove-chest-boat.md
