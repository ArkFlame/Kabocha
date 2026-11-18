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

## Result (final verification)

Re-verified the three STILL OPEN table rows and all four Remaining gaps against
`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractChestBoat.java` and
`.../vehicle/ContainerEntity.java`. All already match vanilla; no source edit was needed this
pass, and this unit declares no editable source files (basenames: none).

STILL OPEN table rows:
- 27-slot inventory: VERIFIED. Vanilla `CONTAINER_SIZE = 27`, `getContainerSize() = 27`;
  Pumpkin `CHEST_BOAT_CONTAINER_SIZE = 27` (boat.rs:44) returned only for chest boats by
  `boat_container_size()` (boat.rs:84-86).
- Single passenger seat: VERIFIED. Vanilla `getMaxPassengers() = 1`; Pumpkin
  `max_passengers()` returns `CHEST_BOAT_MAX_PASSENGERS = 1` vs `BOAT_MAX_PASSENGERS = 2`
  (boat.rs:47-48, 95-101).
- Seat offset 0.15: VERIFIED. Vanilla `getSinglePassengerXOffset() = 0.15F`; Pumpkin consumes
  generated `single_passenger_x_offset` via `passenger_seat_offset` (boat.rs:181-218). Generated
  data off-limits; not edited.

Remaining gaps (all now FIXED in the checkout by other workers):
- Gap 1 (screen validity): FIXED in `minecart/container.rs:179-299`. `EntityContainerScreenHandler`
  + `validity` closure check `is_alive()` and `can_interact_with_entity(..., 4.0)`. Note the
  tracker's "squaredDistanceTo <= 64.0 / 8 blocks" was stale: decompiled vanilla
  `isChestVehicleStillValid` uses `player.isWithinEntityInteractionRange(boundingBox, 4.0)`,
  which the current Pumpkin code matches.
- Gap 2 (chest open states): FIXED in boat.rs via `chest_opens_container` (135-141) and
  `boat_interact_outcome` (161-177): open container only on secondary use or when no seat is
  free, matching vanilla `interact`. Pure tests at 1197-1264.
- Gap 3 (spill position): FIXED. Both boat.rs:287 and minecart.rs:985 call `scatter_inventory_at`
  at the entity position.
- Gap 4 (chest-boat test): FIXED. `seat_and_container_tests` module (boat.rs:1188+) covers
  container size, NBT round trip and interact outcomes.

No source files edited; only this doc. No rustfmt needed. No tests run (worker rule: orchestrator compiles).

## Result (worker re-verification pass)

Re-verified all rows marked STILL OPEN in this unit against
`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractChestBoat.java` and
current Pumpkin sources in the checkout.

STILL OPEN table rows (verified, no edit needed):
- 27-slot inventory: VERIFIED. Vanilla `CONTAINER_SIZE = 27` / `getContainerSize() = 27`
  (`AbstractChestBoat.java:33,114-116`); Pumpkin `CHEST_BOAT_CONTAINER_SIZE = 27` and
  `boat_container_size()` allocate only for chest boats (`boat.rs:44,84-86`).
- Single passenger seat: VERIFIED. Vanilla `getMaxPassengers() = 1` (`AbstractChestBoat.java:48-50`);
  Pumpkin `max_passengers()` returns `CHEST_BOAT_MAX_PASSENGERS = 1` for chest boats vs
  `BOAT_MAX_PASSENGERS = 2` (`boat.rs:47-48,95-101`).
- Seat offset 0.15: VERIFIED. Vanilla `getSinglePassengerXOffset() = 0.15F`
  (`AbstractChestBoat.java:43-45`); Pumpkin consumes generated `single_passenger_x_offset` via
  `passenger_seat_offset` (`boat.rs:181-218`). Generated data is off-limits; not edited.

Remaining gaps:
- Gap 1 (screen validity): already fixed outside this unit's editable scope (`minecart/container.rs`
  `EntityContainerScreenHandler` validity checks `is_alive()` and `can_interact_with_entity(..., 4.0)`).
- Gap 2 (chest open states): already fixed outside this unit's editable scope (`boat.rs`
  `chest_opens_container` / `boat_interact_outcome`, tested in `seat_and_container_tests`).
- Gap 3 (spill position): already fixed outside this unit's editable scope (`boat.rs:287` and
  `minecart.rs:985` call `scatter_inventory_at` at entity position).
- Gap 4 (chest-boat test): already fixed outside this unit's editable scope (`boat.rs`
  `seat_and_container_tests`).

No source files edited; unit allows no source files (basenames: none). No rustfmt needed. No tests
run (worker rule: orchestrator compiles).


## Result (assigned CODER verification pass)

Re-verified every row marked STILL OPEN and all four Remaining gaps against
`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractChestBoat.java` and
`.../vehicle/ContainerEntity.java`, plus the current Pumpkin sources. This unit declares no
editable source files (basenames: none); every row and gap already matches vanilla, so no source
edit was needed this pass.

STILL OPEN table rows (verified, no edit needed):
- 27-slot inventory: VERIFIED. Vanilla `CONTAINER_SIZE = 27` / `getContainerSize() = 27`
  (AbstractChestBoat.java:33,114-116); Pumpkin `CHEST_BOAT_CONTAINER_SIZE = 27` is returned only
  for chest boats by `boat_container_size()` (boat.rs:44,84-86).
- Single passenger seat: VERIFIED. Vanilla `getMaxPassengers() = 1`
  (AbstractChestBoat.java:48-50); Pumpkin `max_passengers()` returns
  `CHEST_BOAT_MAX_PASSENGERS = 1` for chest boats vs `BOAT_MAX_PASSENGERS = 2` otherwise
  (boat.rs:47-48,95-101).
- Seat offset 0.15: VERIFIED. Vanilla `getSinglePassengerXOffset() = 0.15F`
  (AbstractChestBoat.java:43-45); Pumpkin consumes generated `single_passenger_x_offset` via
  `passenger_seat_offset` / `passenger_attachment` (boat.rs:181-222). Generated data is
  off-limits and already correct; not edited.

Remaining gaps (all already fixed in the checkout by other workers; re-verified here):
- Gap 1 (screen validity): FIXED. `EntityContainerScreenHandler` in
  `minecart/container.rs:179-254` wraps the 9x3 handler with a validity closure checking
  `is_alive()` and `can_interact_with_entity(..., 4.0)` (container.rs:281-290). This matches
  decompiled vanilla `ContainerEntity.isChestVehicleStillValid = !isRemoved() &&
  player.isWithinEntityInteractionRange(boundingBox, 4.0)` (ContainerEntity.java:181-182); the
  tracker's "squaredDistanceTo <= 64.0 / 8 blocks" wording was stale.
- Gap 2 (chest open states): FIXED. `chest_opens_container` (boat.rs:135-141) opens only on
  secondary use or when no seat is free, matching vanilla `AbstractChestBoat.interact`
  (AbstractChestBoat.java:79-97); pure tests at boat.rs:1212-1269.
- Gap 3 (spill position): FIXED. Both boat.rs:287 and minecart.rs:985 call `scatter_inventory_at`
  at the entity position, matching vanilla `Containers.dropContents(level, entity, container)`.
- Gap 4 (chest-boat test): FIXED. `seat_and_container_tests` (boat.rs:1203+) covers container
  size, NBT round trip, single-use scatter claim and interact outcomes.

No source files edited (unit allows none; nothing needed a fix). No rustfmt needed. No tests run
(worker rule: orchestrator compiles).

## Result (second-pass CODER verification, full edit scope)

Re-verified every STILL OPEN table row and all four Remaining gaps against decompiled vanilla
AbstractChestBoat.java, AbstractBoat.java, ContainerEntity.java and current Pumpkin sources.
All are genuinely fixed in the checkout; no source edit was needed this pass. Edit scope was
granted over any file a row needs, so no row was left open for scope reasons.

STILL OPEN table rows (verified, no edit needed):
- 27-slot inventory: VERIFIED. Vanilla CONTAINER_SIZE = 27 / getContainerSize() = 27
  (AbstractChestBoat.java:33,114-116). Pumpkin CHEST_BOAT_CONTAINER_SIZE = 27 is returned only
  for chest boats by boat_container_size() (boat.rs:44,84-86).
- Single passenger seat: VERIFIED. Vanilla getMaxPassengers() = 1 (AbstractChestBoat.java:48-50).
  Pumpkin max_passengers() returns CHEST_BOAT_MAX_PASSENGERS = 1 vs BOAT_MAX_PASSENGERS = 2
  (boat.rs:47-48,95-101).
- Seat offset 0.15: VERIFIED. Vanilla getSinglePassengerXOffset() = 0.15F
  (AbstractChestBoat.java:43-45). Pumpkin consumes generated single_passenger_x_offset via
  passenger_seat_offset / passenger_attachment (boat.rs:181-222). Generated data is off-limits
  and already correct; not edited.

Remaining gaps (all confirmed fixed in the checkout, traced end to end):
- Gap 1 (screen validity): FIXED. EntityContainerScreenHandler.can_use (container.rs:206-208)
  runs the validity closure built in open() (container.rs:277-290), which checks the tracked
  entity is_alive() and can_interact_with_entity(..., 4.0). Confirmed the handler is polled
  every tick in player.rs:3584-3595 (can_use false closes the screen), so the screen really
  closes on death or walking away. Matches vanilla
  ContainerEntity.isChestVehicleStillValid = !isRemoved() &&
  player.isWithinEntityInteractionRange(getBoundingBox(), 4.0) (ContainerEntity.java:181-182).
  The tracker's original 8 blocks / squaredDistanceTo <= 64.0 wording was stale.
- Gap 2 (chest open states): FIXED. Traced boat_interact_outcome (boat.rs:161-177) against the
  vanilla chain: AbstractBoat.interact (AbstractBoat.java:779-787) returns SUCCESS only when
  secondary use, out-of-control and failed startRiding are all absent, else PASS; then
  AbstractChestBoat.interact (AbstractChestBoat.java:79-97) opens the container on secondary use
  or when canAddPassenger is false. can_add_passenger = seat_available && !eye_in_water matches
  vanilla canAddPassenger (AbstractBoat.java:836-838). Pure tests cover the chain
  (boat.rs:1213-1280).
- Gap 3 (spill position): FIXED. scatter_chest_contents scatters at entity.pos.load()
  (boat.rs:279-287) and the chest minecart at entity.pos.load() (minecart.rs:985), both via
  scatter_inventory_at, matching vanilla Containers.dropContents(level, entity, container).
- Gap 4 (chest-boat test): FIXED. seat_and_container_tests (boat.rs:1203+) covers container size,
  NBT round trip and interact outcomes; container.rs:350-426 covers item/loot NBT round trips.

No source files edited; only this doc. No rustfmt needed (no .rs file touched). No tests run
(worker rule: orchestrator compiles).

## Result (third-pass CODER verification, full edit scope)

Re-verified every STILL OPEN table row and all four Remaining gaps against decompiled vanilla
and the current Pumpkin checkout. All are already fixed; no source edit was needed. Edit scope was
granted over any file a row needs, so no row is left open for scope reasons.

STILL OPEN table rows:
- 27-slot inventory: VERIFIED. Vanilla `AbstractChestBoat.CONTAINER_SIZE = 27` /
  `getContainerSize() = 27` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractChestBoat.java:33,114-116`).
  Pumpkin `CHEST_BOAT_CONTAINER_SIZE = 27` is returned only for chest boats by `boat_container_size()`
  (boat.rs:44,84-86). No generated field needed.
- Single passenger seat: VERIFIED. Vanilla `getMaxPassengers() = 1` (AbstractChestBoat.java:48-50)
  vs 2 for plain boats (AbstractBoat.java:840-842); Pumpkin `max_passengers()` returns
  `CHEST_BOAT_MAX_PASSENGERS = 1` for chest boats vs `BOAT_MAX_PASSENGERS = 2`
  (boat.rs:47-48,95-101). No generated field needed.
- Seat offset 0.15: VERIFIED. Vanilla `getSinglePassengerXOffset() = 0.15F`
  (AbstractChestBoat.java:43-45); Pumpkin consumes generated `single_passenger_x_offset` via
  `passenger_seat_offset` / `passenger_attachment` (boat.rs:181-222). Generated data is already
  correct and off-limits; not edited, no missing field.

Remaining gaps:
- Gap 1 (screen validity): FIXED. `EntityContainerScreenHandler.can_use` (container.rs:206-208)
  runs the validity closure built in `open()` (container.rs:281-290) checking `is_alive()` and
  `can_interact_with_entity(..., 4.0)`; `can_use` is polled each tick (player.rs:3602), closing
  the screen. Matches vanilla `ContainerEntity.isChestVehicleStillValid = !isRemoved() &&
  player.isWithinEntityInteractionRange(getBoundingBox(), 4.0)`
  (`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/ContainerEntity.java:181-182`).
  The tracker's "8 blocks / squaredDistanceTo <= 64.0" wording was stale.
- Gap 2 (chest open states): FIXED. `boat_interact_outcome` / `chest_opens_container`
  (boat.rs:135-177), used at boat.rs:823-830, mirror the vanilla chain: `AbstractBoat.interact`
  (AbstractBoat.java:779-787) and `canAddPassenger` (AbstractBoat.java:836-838) gate boarding,
  while `AbstractChestBoat.interact` (AbstractChestBoat.java:79-97) opens the container only on
  secondary use or when no seat is free. Pure tests at boat.rs:1212-1285.
- Gap 3 (spill position): FIXED. `scatter_chest_contents` scatters at `entity.pos.load()`
  (boat.rs:279-287) and the chest minecart at `entity.pos.load()` (minecart.rs:985), both via
  `scatter_inventory_at`, matching vanilla `Containers.dropContents(level, entity, container)`
  on removal (AbstractChestBoat.java:70-77).
- Gap 4 (chest-boat test): FIXED. `seat_and_container_tests` (boat.rs:1203+) covers container
  size, NBT round trip and interact outcomes; container.rs tests cover item/loot NBT round trips.

No source files edited (all rows/gaps already match vanilla). No rustfmt needed (no .rs touched).
No tests run (worker rule: orchestrator compiles).


## Result (final third-pass CODER verification)

Re-verified every STILL OPEN table row and all four Remaining gaps against decompiled vanilla
and the current Pumpkin checkout. Every row is VERIFIED or FIXED; none remains STILL OPEN. No
source edit was needed: all fixes are present and match vanilla. Edit scope covered any file a
row needed, so no row was left open for scope reasons.

STILL OPEN table rows:
- 27-slot inventory: VERIFIED. Vanilla `CONTAINER_SIZE = 27`, `NonNullList.withSize(27, ...)` and
  `getContainerSize() = 27`
  (`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractChestBoat.java:33,34,114-116`).
  Pumpkin `CHEST_BOAT_CONTAINER_SIZE = 27`, returned only for chest boats by `boat_container_size()`
  (boat.rs:44,84-86). No generated field needed.
- Single passenger seat: VERIFIED. Vanilla `getMaxPassengers() = 1` (AbstractChestBoat.java:48-50)
  vs 2 for plain boats (AbstractBoat.java:840-842). Pumpkin `max_passengers()` returns
  `CHEST_BOAT_MAX_PASSENGERS = 1` for chest boats vs `BOAT_MAX_PASSENGERS = 2`
  (boat.rs:47-48,95-101). No generated field needed.
- Seat offset 0.15: VERIFIED. Vanilla `getSinglePassengerXOffset() = 0.15F`
  (AbstractChestBoat.java:43-45). Pumpkin consumes generated `single_passenger_x_offset` via
  `passenger_seat_offset` / `passenger_attachment` (boat.rs:181-222). Generated data is already
  correct and off-limits; not edited, no missing field.

Remaining gaps:
- Gap 1 (screen validity): FIXED. `EntityContainerScreenHandler.can_use` (container.rs:206-208)
  runs the validity closure built in `open()` (container.rs:281-290), checking the tracked entity
  `is_alive()` and `can_interact_with_entity(..., 4.0)`. Matches vanilla
  `ContainerEntity.isChestVehicleStillValid = !isRemoved() &&
  player.isWithinEntityInteractionRange(getBoundingBox(), 4.0)`
  (`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/ContainerEntity.java:181-182`).
  The tracker's "8 blocks / squaredDistanceTo <= 64.0" wording was stale.
- Gap 2 (chest open states): FIXED. `chest_opens_container` / `boat_interact_outcome`
  (boat.rs:135-177), applied at boat.rs:823-830. Confirmed against the vanilla chain:
  `AbstractBoat.interact` (AbstractBoat.java:779-787, 784) and `canAddPassenger`
  (AbstractBoat.java:836-837) gate boarding, and `AbstractChestBoat.interact`
  (AbstractChestBoat.java:79-97) opens the container only on secondary use or when no seat is
  free; out-of-control and already-riding never open it. Pure tests at boat.rs:1212-1285.
- Gap 3 (spill position): FIXED. `scatter_chest_contents` scatters at `entity.pos.load()` via
  `scatter_inventory_at` (boat.rs:279-287) and the chest minecart likewise (minecart.rs:985),
  matching vanilla `Containers.dropContents(this.level(), this, this)` on removal
  (AbstractChestBoat.java:70-77).
- Gap 4 (chest-boat test): FIXED. `seat_and_container_tests` (boat.rs:1209+) covers container size,
  NBT round trip and interact outcomes; container.rs tests cover item/loot NBT round trips.

No source files edited (none needed a fix; unit declares no source basenames). No rustfmt needed
(no .rs touched). No tests run (worker rule: orchestrator compiles).


## Result (FINAL PASS - all rows closed)

Worked every row in order against decompiled vanilla under
`.vendor/vanilla-decompiled/net/minecraft/`. Every row ends VERIFIED or FIXED; none remains STILL
OPEN. No source edit was needed because each fix is already present in the checkout and matches
vanilla. Full edit scope was granted across any file a row needs, so no row was left open for scope
reasons.

STILL OPEN table rows (all VERIFIED against vanilla):
- 27-slot inventory: VERIFIED. Vanilla `AbstractChestBoat.CONTAINER_SIZE = 27`,
  `NonNullList.withSize(27, ...)` and `getContainerSize() = 27`
  (`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractChestBoat.java:33,34,114-116`).
  Pumpkin `CHEST_BOAT_CONTAINER_SIZE = 27` is returned only for chest boats by `boat_container_size()`
  (boat.rs:44,84-86). No generated field needed.
- Single passenger seat: VERIFIED. Vanilla `AbstractChestBoat.getMaxPassengers() = 1`
  (AbstractChestBoat.java:48-50), plain `AbstractBoat` uses 2. Pumpkin `max_passengers()` returns
  `CHEST_BOAT_MAX_PASSENGERS = 1` for chest boats vs `BOAT_MAX_PASSENGERS = 2` (boat.rs:47-48,95-101).
  No generated field needed.
- Seat offset 0.15: VERIFIED. Vanilla `AbstractChestBoat.getSinglePassengerXOffset() = 0.15F`
  (AbstractChestBoat.java:43-45). Pumpkin consumes generated `single_passenger_x_offset` via
  `passenger_seat_offset` / `passenger_attachment` (boat.rs:181-222). Generated data is already
  correct and off-limits; not edited, no missing generated field.

Remaining gaps (all FIXED in the checkout, re-traced end to end):
- Gap 1 (screen validity): FIXED. `EntityContainerScreenHandler.can_use` (minecart/container.rs:206-208)
  runs the validity closure built in `open()` (container.rs:281-290), which checks the tracked vehicle
  `is_alive()` and `can_interact_with_entity(vehicle, 4.0)`. `can_interact_with_entity`
  (player.rs:5868-5876) uses `(ENTITY_INTERACTION_RANGE + 4.0)` against the box-to-eye distance
  (boundingbox.rs:494-500), matching vanilla `ContainerEntity.isChestVehicleStillValid = !isRemoved() &&
  player.isWithinEntityInteractionRange(getBoundingBox(), 4.0)`
  (`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/ContainerEntity.java:181-182`).
  The tracker's original "8 blocks / squaredDistanceTo <= 64.0" wording was stale.
- Gap 2 (chest open states): FIXED. `boat_interact_outcome` / `chest_opens_container`
  (boat.rs:135-177, applied at boat.rs:823-830) mirror the vanilla chain: `AbstractBoat.interact` and
  `AbstractChestBoat.interact` (AbstractChestBoat.java:79-97) open the container only on secondary use
  or when `canAddPassenger` is false (no free seat or eye in water); out-of-control and already-riding
  return PASS and never open it. Pure tests at boat.rs:1212-1285.
- Gap 3 (spill position): FIXED. `scatter_chest_contents` scatters at `entity.pos.load()` via
  `scatter_inventory_at` (boat.rs:272-287) and the chest minecart likewise (minecart.rs:1050), matching
  vanilla `Containers.dropContents(this.level(), this, this)` on removal (AbstractChestBoat.java:70-77).
- Gap 4 (chest-boat test): FIXED. `seat_and_container_tests` (boat.rs:1209+) covers container size, NBT
  round trip and interact outcomes; minecart/container.rs tests cover item/loot NBT round trips.

No source files edited; only this doc. No rustfmt needed (no .rs touched). No tests run (worker rule:
orchestrator compiles).

## Result (x_04 pass)
- Gap 1 (screen validity): VERIFIED. Rust `entity/vehicle/minecart/container.rs:288-289` checks `is_alive()` and `can_interact_with_entity(vehicle, 4.0)`; vanilla `world/entity/ContainerEntity.java:181-182`.
- Gap 2 (chest open states): VERIFIED. Rust `entity/vehicle/boat.rs:135-141` opens only when `is_sneaking || !can_add_passenger`; vanilla `vehicle/boat/AbstractChestBoat.java:86`.
- Gap 3 (spill position): VERIFIED. Rust `boat.rs:272-287` uses `scatter_inventory_at(entity.pos)`; `minecart.rs:1051` likewise.
- Gap 4 (chest-boat test): VERIFIED. Tests `boat.rs:1174-1260` (container size, NBT round trip, interact outcomes).
- Scope: the four Remaining-gap rows were re-checked. Earlier table rows were not re-read in this pass. No Rust edited; no checks run.
