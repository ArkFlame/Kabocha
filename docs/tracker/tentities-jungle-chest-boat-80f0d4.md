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
| 5 | `entity/vehicle/boat.rs:356-363` | `AbstractBoat.hurtServer` | Instabuild breaks skip `destroy`, so a creative break drops neither contents nor the boat item. One earlier review claims vanilla still takes the `destroy` branch when `hasCustomName()` is set; unverified | Check `hurtServer` for a `hasCustomName()` clause; if present, drop contents plus the item for a named boat even on an instabuild break | S |
| 6 | `entity/vehicle/boat.rs:339-359` | `AbstractBoat.hurtServer` | A passenger can damage the chest boat it rides (own arrow, own splash potion), so the rider can spill the contents; vanilla is reported to early-return `false` when the source entity is a passenger of the same vehicle | Confirm the guard, then early-return `false` when `source` is one of `self.vehicle.entity.passengers` | S |
| 7 | `entity/vehicle/boat.rs:380-382` | `AbstractBoat.getDropItem` | The drop item is resolved with `Item::from_registry_key(resource_name)`, duplicating the chest-boat mapping that `BoatItem::entity_to_item` already holds; both agree for `jungle_chest_boat` today | Prefer `BoatItem::entity_to_item(...)` and keep the registry lookup as fallback | S |

## Notes

- No cargo/build run (worker rule: the orchestrator batch-compiles); nothing was edited this pass.
- Client-only effects (chest-boat rendering, paddle animation, prediction) not assessed.
- Rows 1-6 live in shared `boat.rs` / `minecart/container.rs` code, so a fix lands once for all 11 chest variants rather than only for the jungle boat.
