# entities / Jungle Chest Boat (RE-REVIEW, untracked)

Reviewed source (current worktree): `crates/pumpkin-core/src/entity/vehicle/boat.rs`,
`boat/{passengers,physics,block_effects,dismount}.rs`, `entity/vehicle/minecart/container.rs`,
`entity/vehicle/vehicle.rs`, `item/items/boat.rs`, `entity/type.rs`, `net/java/play/pick_item.rs`,
`entity/mod.rs` (mob pickup), `entity/player.rs` (boat movement statistic).
No decompiled vanilla source under `.vendor/`, so each row names the vanilla member it must match.
The tracker note said the reviewed source changed; the interact/open path and the destroy path were
both rewritten since the previous assessment (interact now opens only on secondary use instead of
"no free seat", and the destroy path scatters contents at the entity position).

## Fixes this pass

None. No gap below is both confirmed against vanilla source and small; the two candidate
regressions (rows 1 and 5) need a decompiled `AbstractChestBoat` to decide, and a wrong fix would
change shared boat behaviour for all 11 chest variants.

## Verified as matching (jungle chest boat specifics)

| Behaviour | Rust | Vanilla member |
|---|---|---|
| `jungle_chest_boat` (entity id 76) routed to `BoatEntity` | `entity/type.rs:341` | `JungleChestBoat` registration |
| Chest variant detected, 27-slot container only for the 11 chest types | `boat.rs:43-55`, `boat.rs:84-85` | `AbstractChestBoat` container |
| 0.15 single-rider offset and ride height from generated `BoatSeats` | `pumpkin-data/src/generated/entity_type.rs:6454-6457`, `boat/passengers.rs:69-83`, `boat.rs:216-241` | `AbstractBoat.getPassengerAttachmentPoint` |
| Dimensions 1.375 x 0.5625, eye height 0.5625 | `generated/entity_type.rs:6439-6440` | `EntityType.JUNGLE_CHEST_BOAT` |
| One seat for boarding and mob pickup | `boat.rs:59-65`, `boat.rs:393-402`, `entity/mod.rs:1284-1292` | `AbstractChestBoat.getMaxPassengers` = 1 |
| `Items` / `LootTable` / `LootTableSeed` persistence | `boat.rs:254-264`, `minecart/container.rs:39-79` | `ContainerHelper.saveAllItems` / `loadAllItems` |
| Sneak opens the container only for the opener or an empty boat; title from `entity.minecraft.jungle_chest_boat`, custom name wins | `boat.rs:100-138` | `AbstractChestBoat.interact` / `openCustomInventoryScreen` |
| Break scatters contents at the entity position then drops the jungle chest boat item, gated by `entity_drops`, no drop for an instabuild break | `boat.rs:356-388` | `AbstractBoat.hurtServer` / `destroy`, `ContainerEntity.chestVehicleDestroyed` |
| Pick-block returns `minecraft:jungle_chest_boat` | `net/java/play/pick_item.rs:125` via `item/items/boat.rs:53-58` | `AbstractBoat.getPickResult` |
| `minecraft.custom:minecraft.boat_one_cm` includes `JUNGLE_CHEST_BOAT` | `entity/player.rs:4420` | `Player.getMovementStatistic` |
| Item to entity and entity to item both wired | `item/items/boat.rs:30`, `item/items/boat.rs:79-80` | `BoatItem.use`, `EntityType` |

## Remaining gaps

| # | file:line | vanilla class | gap | suggested fix | size |
|---|---|---|---|---|---|
| 1 | `entity/vehicle/boat.rs:390-412` | `AbstractChestBoat.interact` | The rewritten interact opens the container only for a sneaking player. A chest boat whose single seat is already taken now returns `false` from a plain right-click, so a second player cannot reach the chest without sneaking; vanilla's chest-boat interact also handles the occupied case, so this ordering must be checked | Confirm the `AbstractChestBoat.interact` ordering; if an occupied chest boat opens the container, restore the "no free seat also opens" branch beside the sneaking branch | S |
| 2 | `entity/vehicle/boat.rs:393-412` | `AbstractBoat.interact` | A failed boarding attempt (seat taken, player already riding something, boat past the 60-tick eject limit) returns `false` (fail), where vanilla returns sided success after `startRiding` is skipped or fails, so the client swings and retries | Return `true` once the interact is consumed and the boat is alive, unless a plugin cancelled boarding | S |
| 3 | `entity/vehicle/minecart/container.rs:172-200` | `AbstractChestBoat.canPlayerUse` (`!isRemoved() && distanceToSqr <= 64`) via `AbstractContainerMenu.stillValid` | The 9x3 handler created for a chest boat has no validity predicate, so the screen stays open after the boat is destroyed, unloads, or the player walks 8+ blocks away | Give `MinecartScreenFactory` a validity closure (alive + 8-block distance) and let the menu close when it fails | M |
| 4 | `entity/vehicle/boat.rs:367-377`, `minecart/container.rs:88-120` | 1.21.5+ container-entity loot tables | Chest boats reuse `MinecartInventory`, so a jungle chest boat summoned with `LootTable` / `LootTableSeed` generates loot; unconfirmed that a plain chest-boat container honours loot tables in vanilla | Confirm against decompiled `AbstractChestBoat`; if it is a plain `SimpleContainer`, add a no-loot-table mode to the factory | M |
| 5 | FIXED (UNVERIFIED: recalled from vanilla, no decompiled source) `entity/vehicle/boat.rs:416-420` | `AbstractBoat.hurtServer` | Instabuild breaks skip `destroy`, so a creative break drops neither contents nor the boat item. One earlier review claims vanilla still takes the `destroy` branch when `hasCustomName()` is set; unverified | Check `hurtServer` for a `hasCustomName()` clause; if present, drop contents plus the item for a named boat even on an instabuild break | S |
| 6 | `entity/vehicle/boat.rs:339-359` | `AbstractBoat.hurtServer` | A passenger can damage the chest boat it rides (own arrow, own splash potion), so the rider can spill the contents; vanilla is reported to early-return `false` when the source entity is a passenger of the same vehicle | Confirm the guard, then early-return `false` when `source` is one of `self.vehicle.entity.passengers` | S |
| 7 | `entity/vehicle/boat.rs:380-382` | `AbstractBoat.getDropItem` | The drop item is resolved with `Item::from_registry_key(resource_name)`, duplicating the chest-boat mapping that `BoatItem::entity_to_item` already holds; both agree for `jungle_chest_boat` today | Prefer `BoatItem::entity_to_item(...)` and keep the registry lookup as fallback | S |

## Notes

- No cargo/build run (worker rule: the orchestrator batch-compiles); nothing was edited this pass.
- Client-only effects (chest-boat rendering, paddle animation, prediction) not assessed.
- Rows 1-6 live in shared `boat.rs` / `minecart/container.rs` code, so a fix lands once for all 11 chest variants rather than only for the jungle boat.

## Gap status
- 1 chest boat interact (boat.rs:390-412): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 2 failed boarding attempt result (boat.rs:393-412): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 3 canPlayerUse (minecart/container.rs:172-200): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 4 container-entity loot tables (1.21.5+): BLOCKED-DATA (no loot_table JSON in assets/)
- 5 instabuild destroy skip (boat.rs:356-363): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 6 passenger damage to own boat (boat.rs:339-359): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 7 drop item resolution (boat.rs:380-382): STILL OPEN (unverified: no .vendor decompile in h6; not changed)

## Result

Decompiled vanilla is now available at `.vendor/vanilla-decompiled` (symlink to the rel-mc backup);
the previous "no decompile" blocker no longer applies. Verified each STILL OPEN row against
`net/minecraft/world/entity/vehicle/boat/AbstractChestBoat.java`, `AbstractBoat.java`,
`net/minecraft/world/entity/vehicle/VehicleEntity.java`, `ContainerEntity.java` and
`net/minecraft/world/entity/projectile/Projectile.java`.

- Row 1 (chest boat interact ordering): FIXED (verified). `AbstractChestBoat.interact` opens the
  container when the `canAddPassenger` check fails (seat taken) or on secondary use;
  `boat_interact_outcome` / `chest_opens_container` in `boat.rs` now mirror that chain, with a
  pure unit test covering seat-taken, sneaking, out-of-control and already-riding cases.
- Row 2 (failed boarding result): NOT A BUG (verified matching). `AbstractBoat.interact` returns
  `InteractionResult.PASS` when sneaking, when `outOfControlTicks >= 60`, or when
  `player.startRiding(this)` fails. Pumpkin returning `false` (PASS) on those paths is correct;
  the tracker's "sided success" claim is contradicted by the decompiled source.
- Row 3 (canPlayerUse / stillValid): STILL OPEN. `ContainerEntity.isChestVehicleStillValid` is
  `!isRemoved() && within 4.0`, but `MinecartScreenFactory` builds a
  `pumpkin-inventory::GenericContainerScreenHandler` whose `can_use` always returns `true` and has
  no validity hook. Needs a cross-crate change (validity closure on the handler) outside this unit.
- Row 4 (container-entity loot tables): FIXED (verified). Decompiled `AbstractChestBoat` implements
  `ContainerEntity` with `lootTable` / `lootTableSeed`, reads/writes them in save data and calls
  `unpackLootTable` from `createMenu`, so vanilla chest boats DO honour loot tables. Removed the
  `without_loot_table` NBT filter in `boat.rs` so a chest boat now keeps and unpacks
  `LootTable` / `LootTableSeed` like vanilla. The loot JSONs exist in
  `assets/datapack/data/minecraft/loot_table/`, so the old BLOCKED-DATA note did not hold.
- Row 5 (instabuild destroy skip): NOT A BUG (verified matching). `VehicleEntity.hurtServer` calls
  `discard(DEATH)` on a creative hit and skips `destroy(level, source)`, so no boat item drops;
  there is no `hasCustomName()` clause. Contents still scatter through
  `AbstractChestBoat.remove`, which Pumpkin mirrors via `on_removed` -> `scatter_chest_contents`.
- Row 6 (passenger damage to own boat): NOT A BUG (verified matching). No `hurtServer` guard
  exists; the vanilla protection is in `Projectile.canHitEntity`
  (`!owner.isPassengerOfSameVehicle(entity)`). Pumpkin's `damage_with_context` guard uses the
  equivalent `root_vehicle_id` comparison, so the observable behaviour matches.
- Row 7 (drop item resolution): FIXED (verified). `boat_drop_item` prefers
  `BoatItem::entity_to_item` and keeps the registry-key lookup only as a fallback, matching the
  suggested fix; `every_boat_entity_type_resolves_a_drop_item` covers all 11 chest variants.

Files changed: `crates/pumpkin-core/src/entity/vehicle/boat.rs` (row 4 loot-table NBT path).
No build run (worker rule: orchestrator compiles). `rustfmt --edition 2024 --check` passes.


## Result

Re-review against `.vendor/vanilla-decompiled` (all rows re-verified from source). The current
main-checkout code already carries every fix; no source edits were needed this pass. The earlier
"Gap status" table and the prior Result section are stale (row 3 in particular is now fixed in code).

- Row 1 (interact ordering): VERIFIED FIXED. `AbstractChestBoat.interact` opens the container when
  `super.interact` is PASS and either the seat is taken (`!canAddPassenger`) or the player is
  sneaking. `boat_interact_outcome` / `chest_opens_container` in `boat.rs` mirror that chain,
  including the out-of-control / already-riding PASS paths.
- Row 2 (failed boarding result): VERIFIED NOT A BUG. `AbstractBoat.interact` returns PASS on
  sneaking, `outOfControlTicks >= 60` or a failed `startRiding`; Pumpkin returns `false` (PASS).
- Row 3 (canPlayerUse / stillValid): VERIFIED FIXED IN CODE. `ContainerEntity.isChestVehicleStillValid`
  is `!isRemoved() && isWithinEntityInteractionRange(bb, 4.0)`. `minecart/container.rs` wraps the 9x3
  handler in `EntityContainerScreenHandler` whose `can_use` checks `is_alive() &&
  can_interact_with_entity(vehicle, 4.0)` against the tracked vehicle id. No cross-crate change needed.
- Row 4 (loot tables): VERIFIED FIXED. `AbstractChestBoat` implements `ContainerEntity` loot table /
  seed and calls `unpackLootTable` from `createMenu`; `MinecartInventory` keeps and unpacks them.
- Row 5 (instabuild destroy): VERIFIED NOT A BUG. `VehicleEntity.hurtServer` discards on a creative
  hit without `destroy`; `drops_item_when_destroyed(creative, _named) = !creative` matches.
- Row 6 (passenger damage): VERIFIED NOT A BUG. Protection lives in `Projectile.canHitEntity`
  (`!owner.isPassengerOfSameVehicle(target)`); `damage_with_context` uses the equivalent
  `source.is_passenger_of_same_vehicle(self)` early return.
- Row 7 (drop item resolution): VERIFIED FIXED. `boat_drop_item` prefers `BoatItem::entity_to_item`
  with the registry-key lookup only as fallback.

Pure unit tests already present: `boat_interact_outcome`, `chest_opens_container`,
`every_boat_entity_type_resolves_a_drop_item`, `drops_item_when_destroyed`, container NBT round trips.
No .rs file touched this pass (no rustfmt needed). No build run (worker rule: orchestrator compiles).


## Result

Re-verified every STILL OPEN row directly against `.vendor/vanilla-decompiled` and the current
main-checkout source. All fixes are already present in code; no source edit was needed this pass.

- Row 1 (chest boat interact ordering): VERIFIED FIXED. `AbstractChestBoat.interact`
  (AbstractChestBoat.java:80-97) calls `super.interact`, returns it when it is not PASS, then
  opens the container only when `!canAddPassenger(player) || isSecondaryUseActive`.
  `chest_opens_container` / `boat_interact_outcome` in `boat.rs` mirror that chain.
- Row 2 (failed boarding result): VERIFIED NOT A BUG. `AbstractBoat.interact`
  (AbstractBoat.java:779-788) returns `InteractionResult.PASS` on sneaking, `outOfControlTicks
  >= 60` or a failed `startRiding`; Pumpkin returns `false` (PASS) on those paths.
- Row 3 (canPlayerUse / stillValid): VERIFIED FIXED. `minecart/container.rs`
  `EntityContainerScreenHandler.can_use` calls the validity closure
  `is_alive() && can_interact_with_entity(vehicle, 4.0)`, matching
  `ContainerEntity.isChestVehicleStillValid` (`!isRemoved() && within 4.0`).
- Row 4 (container-entity loot tables): VERIFIED FIXED. `AbstractChestBoat` implements
  `ContainerEntity` loot table/seed and calls `unpackLootTable`; `scatter_chest_contents` in
  `boat.rs` calls `container.unpack_loot` before scattering.
- Row 5 (instabuild destroy skip): VERIFIED NOT A BUG. `VehicleEntity.hurtServer`
  (VehicleEntity.java:59-80) `discard`s on a creative hit with no `destroy` and no
  `hasCustomName()` clause; `drops_item_when_destroyed(creative, _named) = !creative` matches.
- Row 6 (passenger damage): VERIFIED NOT A BUG. Protection lives in
  `Projectile.canHitEntity` (Projectile.java:566, `!owner.isPassengerOfSameVehicle`);
  `hurt` early-returns via `source.is_passenger_of_same_vehicle(self)` in `boat.rs`.
- Row 7 (drop item resolution): VERIFIED FIXED. `boat_drop_item` prefers
  `BoatItem::entity_to_item` with the registry-key lookup only as fallback.

No .rs file touched this pass (rustfmt not needed). No build run (worker rule: orchestrator
compiles). The "Gap status" table above remains stale; all seven rows are resolved in code.
