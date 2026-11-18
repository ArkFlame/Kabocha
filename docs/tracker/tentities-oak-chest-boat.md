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

## Gap status
- Table rows (27-slot inventory, single seat, seat offset): RESOLVED - re-verified against decompiled vanilla; all three already match, no edit needed.


## Result

Verified against `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractChestBoat.java`,
`.../minecart/AbstractMinecartContainer.java`, `.../ContainerEntity.java` and `.../Containers.java`.

STILL OPEN table rows (all resolved, already match vanilla; no edit needed):
- 27-slot inventory: VERIFIED. Vanilla `CONTAINER_SIZE = 27` / `getContainerSize() = 27`; Pumpkin `CHEST_BOAT_CONTAINER_SIZE = 27` (boat.rs:42) is used only for chest boats (boat.rs:183-184).
- Single passenger seat: VERIFIED. Vanilla `AbstractChestBoat.getMaxPassengers() = 1` (vs 2 in `AbstractBoat`); Pumpkin `max_passengers()` returns 1 for chest boats, 2 otherwise (boat.rs:74-80).
- Seat offset 0.15: VERIFIED. Vanilla `getSinglePassengerXOffset() = 0.15F`; generated `single_passenger_x_offset = 0.15f32` for chest boats (pumpkin-data entity_type.rs:7705) consumed by `passenger_attachment` (boat.rs:148-155). Generated data; not edited.

Remaining gaps:
- Gap 1 (screen validity): left open - needs a liveness/distance callback in `minecart/container.rs`, a file outside this unit.
- Gap 2 (chest open states): left open - fix belongs in `boat.rs`, outside this unit.
- Gap 3 (spill position): boat.rs side already uses `scatter_inventory_at` (other worker). FIXED minecart.rs side - container destroy now scatters at `entity.pos.load()` via `scatter_inventory_at`, matching vanilla `Containers.dropContents(level, entity, container)`.
- Gap 4 (chest-boat test): left open - needs a boat.rs unit test, outside this unit.

No new pure unit test: this fix is a one-line call to the already-shared `scatter_inventory_at` helper (covered by its other callers); no new pure seam exists in minecart.rs for it.

Files edited: `crates/pumpkin-core/src/entity/vehicle/minecart.rs` (rustfmt --edition 2024 rc 0), this doc.


## Result (re-verification)

Re-checked the three STILL OPEN table rows against
`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractChestBoat.java`.
All three already match vanilla; no source edit was needed.

- 27-slot inventory: VERIFIED. Vanilla `CONTAINER_SIZE = 27`, `getContainerSize() = 27`,
  `NonNullList.withSize(27, ...)`; Pumpkin `CHEST_BOAT_CONTAINER_SIZE = 27` (boat.rs:44) is
  returned only for chest boats by `boat_container_size()` (boat.rs:84-86).
- Single passenger seat: VERIFIED. Vanilla `getMaxPassengers() = 1` (AbstractBoat uses 2);
  Pumpkin `max_passengers()` returns `CHEST_BOAT_MAX_PASSENGERS = 1` for chest boats and
  `BOAT_MAX_PASSENGERS = 2` otherwise (boat.rs:47-48, 95-101).
- Seat offset 0.15: VERIFIED. Vanilla `getSinglePassengerXOffset() = 0.15F`; generated
  `single_passenger_x_offset` is `0.15f32` for all 11 chest boats/rafts and `0f32` for the
  11 plain boats, consumed via `passenger_attachment` / `passenger_seat_offset` (boat.rs:181-218).
  Generated data already correct; not edited (generated files are off-limits).

Observed since the prior Result: `boat.rs` was refactored by other workers with pure seams
(`chest_opens_container`, `boat_interact_outcome`, `passenger_seat_offset`) and a
`seat_and_container_tests` module, so gap 2 and gap 4 are now covered elsewhere. Gap 1
still needs `minecart/container.rs` (outside this unit). Uncommitted `minecart.rs` changes
in the checkout were not made by this worker and were left untouched.

No source file edited; only this doc (stale Gap status updated and Result appended).
No rustfmt needed (no .rs file touched). No tests run (worker rule: orchestrator compiles).
