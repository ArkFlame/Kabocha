# entities / Cherry Chest Boat

Re-review of `crates/pumpkin-core/src/entity/vehicle/boat.rs` (source changed since the previous
assessment). Cherry chest boat is one variant of the shared `BoatEntity`; the variant data is complete
(`EntityType::CHERRY_CHEST_BOAT`, id 24, `dimension` 1.375x0.5625, `boat` seats present, item
`minecraft:cherry_chest_boat` registered in `BoatItem::ids`/`item_to_entity`/`entity_to_item`).

## Verified matching vanilla
- 27-slot container (`CHEST_BOAT_CONTAINER_SIZE`), one seat for chest boats (`max_passengers`), two for
  plain boats. `is_chest_boat` covers all 10 chest boats + bamboo chest raft from generated data.
- Items saved/loaded under `Items` via `MinecartInventory` (mirrors `ContainerHelper.saveAllItems`),
  container contents scattered on destroy, boat item dropped with its custom name
  (`Boat.destroy` / `AbstractBoat.getDropItem` + hover name).
- Sneak-click opens, click with a free seat rides, click on a full boat opens
  (`Boat.interact` -> `startRiding` fails -> `ChestBoat.interact` opens).
- Opening from a riding player uses `Player::open_vehicle_inventory` (`HasCustomInventoryScreen`).
- Alternative-title key `entity.minecraft.<resource_name>` and the bedrock name key per raft/boat.
- Fixed today: spectators can no longer open the chest boat screen (`ChestBoat.interact` guards with
  `!player.isSpectator()`).

## Remaining gaps
| # | Location | Vanilla class | Gap | Suggested fix | Size |
|---|----------|---------------|-----|---------------|------|
| 1 | `crates/pumpkin-core/src/entity/vehicle/boat.rs:95` | `net.minecraft.world.entity.vehicle.AbstractChestBoat#openCustomInventoryScreen` | Vanilla only opens for `!hasPassengers() || hasPassenger(player)`; Pumpkin opens for any non-spectator, so a stranger can loot a chest boat occupied by another player. Confirm the guard text against the target version before changing. | Gate the open on `passengers.is_empty() \|\| passengers.contains(player)` and fall back to the ride path in `interact` (`boat.rs:301`). | S |
| 2 | `crates/pumpkin-core/src/entity/vehicle/boat.rs:97-104` | `net.minecraft.world.level.gameevent.GameEvent.CONTAINER_OPEN` | Emits `CONTAINER_OPEN` at the boat position; vanilla emits that event only from block containers, so sculk sensors/wardens react to a chest boat opening when they should not. Chest minecarts (`minecart/container.rs:202` `open`) emit nothing. | Drop the `emit_game_event_with_context` block, or move it to block-container-only paths. | S |
| 3 | `crates/pumpkin-core/src/entity/player.rs:7474-7479`, `entity/vehicle/minecart/container.rs:202-233` | Bedrock `ContainerOpen` packet | Entity-backed containers are sent with `target_entity_id: -1` and `position: BlockPos::ZERO`; Bedrock clients need the vehicle runtime id for chest boats/chest minecarts, so the chest UI is not tied to the entity and may open at the wrong place. | Thread an optional `target_entity_id` through `Player::open_handled_screen` / `container::open` and set the vehicle `entity_id` for boat/minecart containers. | M |
| 4 | `crates/pumpkin-core/src/entity/vehicle/boat.rs:172-182` | `net.minecraft.world.entity.vehicle.ChestBoat#addAdditionalSaveData` | Boat NBT takes the minecart path, so `LootTable`/`LootTableSeed` are accepted and later unpacked on open/destroy; vanilla chest boats store only `Items` (loot tables are minecart-only). A summoned/serialized chest boat can therefore spawn loot that vanilla would ignore. | Skip loot-table read/write (and `unpack_loot`/`claim_drops`) for `is_chest_boat` inventories. | S |
| 5 | `crates/pumpkin-core/src/entity/mod.rs:1227-1231` | `AbstractBoat#canAddPassenger` | The one-seat limit for chest boats is re-derived inline instead of using `BoatEntity::max_passengers`, so a new chest variant added to `is_chest_boat` can drift from the boat tick path. | Expose `BoatEntity::max_passengers` (or a free `max_passengers(entity_type)`) and reuse it in the push-mount path. | S |
