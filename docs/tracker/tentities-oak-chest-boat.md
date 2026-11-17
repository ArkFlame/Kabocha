# Tracker: entities / Oak Chest Boat

Reviewed: `crates/pumpkin-core/src/entity/vehicle/boat.rs` and its `boat/*` submodules,
`item/items/boat.rs`, `entity/type.rs`, `entity/vehicle/minecart/container.rs` at branch
`kab/tentities-oak-chest-boat`. Shared code covers every chest boat/raft variant (oak, spruce,
birch, jungle, acacia, dark oak, mangrove, cherry, pale oak, poplar, bamboo raft).
Vanilla reference: `net.minecraft.world.entity.vehicle.AbstractChestBoat` (1.21.x),
`AbstractBoat`, `BoatItem`, `ContainerHelper`, `Containers`.

## Verified as matching vanilla

| Behaviour | Evidence |
|---|---|
| Chest boat identified for all 11 variants (incl. bamboo chest raft, poplar) | `boat.rs:43-55`; registration in `entity/type.rs:332-356` |
| 27-slot inventory created only for chest boats | `boat.rs:39,74-75` |
| Single passenger seat (regular boats keep 2) | `boat.rs:94-96`; mob pickup cap `entity/mod.rs:1238-1249` |
| Single-rider seat offset 0.15 / ride height from generated boat seats | `passengers.rs:67-69`; `pumpkin-data .../entity_type.rs:7703-7706` |
| Right-click opens, sneak opens, occupied boat opens | `boat.rs:364-400` |
| Title: per-variant `entity.minecraft.<variant>`, Bedrock chest boat/raft key | `boat.rs:98-112` |
| `ContainerOpen` game event with player context | `boat.rs:114-123` |
| NBT `Items` / `LootTable` + `LootTableSeed`, deferred loot preserved | `boat.rs:235-245`; `minecart/container.rs:40-109,287-329` |
| Loot table unpacked on open and on hopper access | `boat.rs:98-125,263-274`; `minecart/container.rs:90-109,202-233` |
| Contents spilled on break, boat item dropped with custom name, `doEntityDrops` honoured | `boat.rs:315-361` |
| Creative break drops nothing (vanilla `AbstractBoat.hurt` calls `discard()`, not `destroy`) | `boat.rs:324-358` |
| Chest boat item <-> entity mapping, dispenser, pick block | `item/items/boat.rs:21-58`; `redstone/dispenser.rs:543-557`; `pick_item.rs:100` |
| `has_container_inventory` exposed to hoppers | `boat.rs:259-274` |

## Remaining gaps

### 1. Entity container screen never becomes invalid - size M

`crates/pumpkin-core/src/entity/vehicle/minecart/container.rs:178-200`
(`MinecartScreenFactory::create_screen_handler`). Vanilla
`AbstractChestBoat.canPlayerUse(Player)` is
`!isRemoved() && player.squaredDistanceTo(this) <= 64.0`, checked every tick by
`AbstractContainerMenu.stillValid`. Pumpkin's generic 9x3 handler has no entity liveness or
distance callback, so a player can keep the chest boat screen open after walking away, after
the boat is destroyed, or after the entity is unloaded. Suggested fix: give
`MinecartScreenFactory` (and the handler it creates) a `Box<dyn Fn() -> bool>` validity
closure for chest boats and chest minecarts that checks `entity.is_alive()` and the 8-block
distance, and close the menu when it fails.

### 2. Chest opens in states where vanilla only attempts boarding - size S

`crates/pumpkin-core/src/entity/vehicle/boat.rs:377`.
`if self.inventory.is_some() && (!can_add_passenger || player.get_entity().is_sneaking())`
uses `can_add_passenger`, which is also false because the player's eyes are in water, the
boat passed `TIME_TO_EJECT`, or the player already rides another entity
(`boat.rs:365-375`). Vanilla `AbstractChestBoat.interact` opens the container only when
`player.isSecondaryUseActive() || this.isVehicle()`; otherwise it defers to
`AbstractBoat.interact` (board or `PASS`). Suggested fix: open only when
`player.get_entity().is_sneaking() || !passengers.is_empty()`.

### 3. Spilled contents spawn at the boat's block origin - size S

`crates/pumpkin-core/src/entity/vehicle/boat.rs:335,344-345` passes
`self.vehicle.entity.block_pos.load()` to `world.scatter_inventory`. Vanilla
`Containers.dropContents(level, entity, container)` uses the entity's
`getX()/getY()/getZ()`. Suggested fix: add a position-based variant of
`World::scatter_inventory` (loop `scatter_stack` with `entity.pos`) and use it here and in
`minecart.rs:891`.

### 4. No chest-boat-specific automated test - size S

`crates/pumpkin-core/src/entity/vehicle/minecart/container.rs:287-329` tests the shared
inventory NBT, but nothing asserts that `BoatEntity::new` allocates 27 slots only for chest
boat entity types, that `write_custom_nbt`/`read_custom_nbt` round-trip through the boat, or
that the destroy path spills contents exactly once. Suggested fix: a unit/gametest
constructing `BoatEntity` for `OAK_CHEST_BOAT` vs `OAK_BOAT` and asserting
`has_container_inventory()` plus an NBT round trip.

## Notes

- Compilation and tests were not run (worker rule: orchestrator batch-compiles).
- Client-only effects (chest-boat open animation, prediction) were not assessed.
