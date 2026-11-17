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
- 1 ChestBoat.remove scatter on any destroying removal (boat.rs:315-362): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 2 shared getMaxPassengers helper (entity/mod.rs:1238-1243): STILL OPEN (refactor, no behaviour change identified)
- 3 interact deciding result on failed mount (boat.rs:365-400): STILL OPEN (unverified: no .vendor decompile in h6; not changed)


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
