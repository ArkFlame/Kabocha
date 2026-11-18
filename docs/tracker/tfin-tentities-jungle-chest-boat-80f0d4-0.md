# Final rows from docs/tracker/tentities-jungle-chest-boat-80f0d4.md

FINAL PASS. Every row was re-verified directly against `.vendor/vanilla-decompiled` and the
current main-checkout source. Every fix is already present in code, so no `.rs` file needed an
edit (rustfmt is not applicable). No build run (worker rule: orchestrator compiles). Stale
fragments from earlier passes are superseded by the per-row results in `## Result`.

## Result

- Row 7 (drop item resolution): FIXED. `boat_drop_item` (boat.rs:106-109) prefers
  `BoatItem::entity_to_item` and keeps `Item::from_registry_key` only as a fallback, matching
  vanilla `AbstractBoat.getDropItem`
  (.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java).
  Covered by `every_boat_entity_type_resolves_a_drop_item`.
- Row 5 (instabuild destroy skip): VERIFIED (matches vanilla, NOT A BUG). Vanilla
  `VehicleEntity.hurtServer`
  (.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/VehicleEntity.java:59-80)
  discards on a creative (`instabuild`) hit without calling `destroy()` and without a
  `hasCustomName()` clause. `drops_item_when_destroyed(creative, _named) = !creative`
  (boat.rs:117-119) matches.
- Row 6 (passenger damage to own boat): VERIFIED (matches vanilla, NOT A BUG). Vanilla guard is
  in `Projectile.canHitEntity`
  (.vendor/vanilla-decompiled/net/minecraft/world/entity/projectile/Projectile.java:551-567):
  `!entity.canBeHitByProjectile()` precheck, Paper vanish check, then
  `owner == null || leftOwner || !owner.isPassengerOfSameVehicle(entity)` (line 566). Pumpkin
  `ProjectileOwner::can_hit` (owner.rs:234-246) mirrors it: vanish precheck first, then
  `left_owner || owner.get(world).is_none_or(|owner| !owner.is_passenger_of_same_vehicle(entity))`;
  `is_none_or` covers `owner == null`. `can_hit_entity` (projectile/mod.rs) applies
  `can_be_hit_by_projectile` first. No `hurtServer` early return was added in boat.rs
  (comment lines 725-726), matching vanilla.
- Row 3 (canPlayerUse / stillValid): FIXED. Vanilla `ContainerEntity.isChestVehicleStillValid`
  is `!isRemoved() && player.isWithinEntityInteractionRange(boundingBox, 4.0)`
  (.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/ContainerEntity.java:181-183).
  `EntityContainerScreenHandler.can_use` (minecart/container.rs:206-208) runs a validity closure
  checking `is_alive() && can_interact_with_entity(vehicle, 4.0)` against the vehicle tracked by
  id (container.rs:279-290), so the screen closes when the boat is destroyed or the player walks
  away.
- Row 4 (container-entity loot tables): FIXED. `AbstractChestBoat implements ContainerEntity`
  (.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractChestBoat.java:32,35-36)
  with `lootTable`/`lootTableSeed` read, write and `unpackLootTable` from `createMenu` (lines
  152-184; ContainerEntity.java:39-120). `MinecartInventory::read_nbt`/`write_nbt` persist
  `LootTable`/`LootTableSeed` (container.rs:43-88), `open` calls `inventory.unpack_loot`
  (container.rs:265), and `scatter_chest_contents` unpacks before scattering (boat.rs:281). No
  `without_loot_table` filter remains. Loot JSONs exist under
  `assets/datapack/data/minecraft/loot_table/`, so no BLOCKED-DATA condition applies.
- Row 2 (failed boarding result): VERIFIED (matches vanilla, NOT A BUG). Vanilla
  `AbstractBoat.interact`
  (.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java:779-788)
  returns `PASS` when sneaking, `outOfControlTicks >= 60`, or `startRiding` fails; Pumpkin
  returns `false` (PASS) via `BoatInteractOutcome::Nothing` (boat.rs:833) on those paths.
- Row 1 (chest boat interact ordering): FIXED. Vanilla `AbstractChestBoat.interact`
  (.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractChestBoat.java:80-97)
  returns `super.interact` when it is not PASS, then offers the container when
  `canAddPassenger(player) && !player.isSecondaryUseActive()` is false (seat taken or secondary
  use). `chest_opens_container` / `boat_interact_outcome` (boat.rs:135-177) mirror that chain;
  pure tests `boat_interact_outcome` / `chest_opens_container` cover seat-taken, sneaking,
  out-of-control and already-riding cases.

No row remains STILL OPEN: rows 1/3/4/7 FIXED, rows 2/5/6 VERIFIED matching vanilla. No new pure
test needed (all seams already covered). No `.rs` file touched, so rustfmt is not applicable. No
build run (worker rule: orchestrator compiles).

Status: PASS fixed=4 open=0 files=docs/tracker/tfin-tentities-jungle-chest-boat-80f0d4-0.md
