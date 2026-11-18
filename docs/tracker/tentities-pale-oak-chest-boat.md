# Tracker: entities / Pale Oak Chest Boat

Re-review (RE-REVIEW-UNTRACKED). Source: `crates/pumpkin-core/src/entity/vehicle/boat.rs`
(shared by every chest boat/raft), `entity/type.rs:332-356`,
`item/items/boat.rs:21-98`, `entity/vehicle/minecart/container.rs`,
generated `pumpkin-data/src/generated/entity_type.rs:7987-8043`.
No decompiled tree is present in this checkout (`.vendor/` empty), so the vanilla
comparison below is against `net.minecraft.world.entity.vehicle.AbstractChestBoat` /
`AbstractBoat` / `BoatItem` behaviour, not quoted source.

The re-review was triggered by three shared chest-boat changes after the previous audit:
`0fe61a1fb` (spectator guard), `baf497b61` (container-open game event removed) and
`97ba2480d` (contents drop on creative break). All three are present and consistent with
the vanilla destroy/spawn paths below.

## Verified parity

| Behaviour | Pumpkin |
|---|---|
| Variant is created as a boat entity | `entity/type.rs:332-356` includes `PALE_OAK_CHEST_BOAT` |
| Chest variant detection for all 11 boats/rafts | `boat.rs:41-53` |
| 27-slot container only for chest boats | `boat.rs:37,72-73,92-94`; `minecart/container.rs:23-38` |
| One passenger seat (regular boats keep two) | `boat.rs:92-94`; mob pickup `entity/mod.rs:1237-1250` |
| Seat offset/ride height from generated `BoatSeats` | `boat/passengers.rs:64-88`, `boat.rs:183-215` |
| Board when a seat is free, open when sneaking or full | `boat.rs:374-409` |
| Menu only for non-spectators, alive, and not occupied by another entity | `boat.rs:96-121` (fixed this pass) |
| Title `entity.minecraft.<variant>`, Bedrock chest-boat/raft key, custom name wins | `boat.rs:120-129`, `minecart/container.rs:227` |
| No `GameEvent.CONTAINER_OPEN` from entity containers (block containers only) | `minecart/container.rs:202-233` |
| Break threshold `damage + amount*10 > 40`, creative instabuild | `boat.rs:322-341` |
| `doEntityDrops` gating: contents always scatter, boat item non-creative only | `boat.rs:341-369`, vanilla `ChestBoat.destroy` + `AbstractBoat.hurt` |
| Boat item resolved from `resource_name`, keeps custom name | `boat.rs:359-368`, `item/items/boat.rs:21-58` |
| `Items` NBT round trip at container root | `boat.rs:242-252`, `minecart/container.rs:40-81` |
| Item places matching entity, consumes one, increments stat | `item/items/boat.rs:159-210` |
| Boat movement statistic includes pale oak chest boat | `entity/player.rs:4388` |

## Fixed this pass

- `boat.rs:96-121` — `open_custom_inventory_screen` now also refuses when the boat is dead
  or ridden by a different entity (`!passengers.is_empty() && !hasPassenger(player)`),
  matching `AbstractChestBoat.openCustomInventoryScreen`. Previously a non-sneaking
  stranger clicking a chest boat occupied by another player opened its 27 slots. The rider
  itself can still open its own boat, and `interact` now falls through to `false` (PASS)
  for the stranger instead of consuming the click.

## Remaining gaps

| # | Location | Vanilla class | Gap | Suggested fix | Size |
|---|---|---|---|---|---|
| 1 | `minecart/container.rs:202-233`, `boat.rs:96` | `AbstractChestBoat.canPlayerUse` / `AbstractContainerMenu.stillValid` | Entity-backed screen never becomes invalid: no 8-block distance or entity-liveness check, so the menu survives walking away, the boat being destroyed or unloaded. | Give `MinecartScreenFactory` a validity closure (`is_alive() && squaredDistance <= 64`) and close the menu when it fails; use it for chest boats and chest minecarts. | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 2 | `item/items/boat.rs:187-198` | `BoatItem.use` + `minecraft:container` component (1.21.2+) | Placing a chest boat item ignores the `minecraft:container` component, so a serialized boat with contents spawns empty. | After `BoatEntity::new`, copy `ContainerImpl` stacks slot-for-slot (clamp 27) before `world.spawn_entity`. Confirm the target-version `BoatItem` still forwards the component. | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 3 | `boat.rs:266-281`, hopper path | `AbstractChestBoat implements Container` only | `has_container_inventory` exposes the chest boat as a worldly container, so a hopper above may insert; vanilla chest boats are plain `Container` (extract only). Verify the hopper/worldly-container path before changing. | Gate insertion per vehicle kind if the hopper path currently allows it. | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 4 | `boat.rs:348-354` | `Containers.dropContents(level, entity, container)` | Scattered contents use `entity.block_pos` (block origin) instead of the entity's exact `pos`. Cosmetic item-spawn offset only. | Add a position-based `World::scatter_inventory` variant and use `entity.pos` here and in `minecart.rs`. | S, FIXED ff30569c9 (already on master) |
| 5 | `boat.rs:242-252` | `AbstractChestBoat` save/load | No automated test for the chest-boat 27-slot NBT round trip or the new open guard. | Unit test: `BoatEntity` for `PALE_OAK_CHEST_BOAT` vs `PALE_OAK_BOAT`, assert `has_container_inventory()` and an `Items` round trip through slot 26. | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |

Client-only effects (chest rendering, screen layout, paddle wobble) were not assessed.
No cargo/test run (worker constraint); the orchestrator batch-compiles.


## Gap status
- 1 canPlayerUse (minecart/container.rs:202-233): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 2 BoatItem container component (item/items/boat.rs:187-198): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 3 hopper path Container-only (boat.rs:266-281): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 4 scattered contents source (boat.rs:348-354): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 5 save/load test coverage (boat.rs:242-252): STILL OPEN (needs test, unverified)


## Result

Verified against `.vendor/vanilla-decompiled` (`Containers`, `ContainerEntity`,
`AbstractChestBoat`, `AbstractMinecartContainer`, `BoatItem`). This unit may edit only
files whose basename is `minecart.rs`; neither `minecart.rs` file needs a change.

- Gap 1 (canPlayerUse/stillValid): OUT OF UNIT. Files `entity/vehicle/minecart/container.rs`
  (basename `container.rs`) and `entity/vehicle/boat.rs`. Vanilla `ContainerEntity.isChestVehicleStillValid`
  checks `!isRemoved() && player.isWithinEntityInteractionRange(getBoundingBox(), 4.0)`
  (4-block interaction range, not the 8 blocks stated in the gap).
- Gap 2 (BoatItem container component): OUT OF UNIT. File `item/items/boat.rs`. Decompiled
  `BoatItem.getBoat` only calls `EntityType.createDefaultStackConfig(...).apply(boat)`;
  no explicit container copy in this version, so the gap claim needs re-check in that unit.
- Gap 3 (hopper/worldly-container): OUT OF UNIT. File `entity/vehicle/boat.rs` (`has_container_inventory`).
- Gap 4 (scattered contents source): VERIFIED FIXED, no change. `entity/vehicle/minecart.rs:919-921`
  calls `world.scatter_inventory_at(self.vehicle.entity.pos.load(), &inventory)`, matching
  vanilla `Containers.dropContents(level, entity, container)` which feeds `entity.getX/Y/Z`
  into `dropItemStack` (floors each axis, so the entity fraction matters). `boat.rs:245-252`
  likewise uses `entity.pos.load()`. Row table FIXED note confirmed on checkout.
- Gap 5 (save/load test): OUT OF UNIT. File `entity/vehicle/boat.rs`; no pure test seam
  for chest-boat NBT in either `minecart.rs`.

No .rs file edited, so no rustfmt run. No build/test run (worker constraint).


## Result

Rechecked on the current checkout. `.vendor/` still contains no decompiled tree, so vanilla
claims below rest on the prior pass' quoted vanilla signatures plus the code's own
vanilla-reference comments, not fresh decompiled source. This unit's basename list is empty,
so no source file was edited; several rows were already fixed in the checkout by other
workers and are recorded as fixed rather than re-fixed. This section supersedes the older
"Gap status" and "Result" sections above.

- Gap 1 (canPlayerUse / stillValid): ALREADY FIXED in checkout, no change. `minecart/container.rs:179-208,219-303`
  now defines `EntityValidityCheck`, wraps `GenericContainerScreenHandler` so `can_use` calls the
  closure, and builds it at `:281-290` as `vehicle.get_entity().is_alive() && can_interact_with_entity(vehicle, 4.0)`
  (4-block interaction range), matching vanilla `ContainerEntity.stillValid`. The `open` helper is
  shared by chest boats (`boat.rs:317`) and chest minecarts, so both are covered.
- Gap 2 (BoatItem `minecraft:container` component): STILL OPEN, OUT OF UNIT. `item/items/boat.rs:173-184`
  applies only spawn-egg variant NBT before `world.spawn_entity`; no container-contents copy.
  Needs the target-version decompiled `BoatItem.getBoat` / `createDefaultStackConfig().apply`
  verdict and an edit in `boat.rs` (basename `boat.rs`), which this unit cannot touch.
- Gap 3 (hopper / worldly container): STILL OPEN, OUT OF UNIT. `boat.rs:548` still reports
  `has_container_inventory() == inventory.is_some()` for chest boats. Confirming whether vanilla
  chest boats accept hopper insertion needs decompiled `AbstractChestBoat` and a change in
  `boat.rs` / the hopper path, both outside this unit.
- Gap 4 (scatter source position): ALREADY FIXED on master, no change. `boat.rs:280-287` and
  `minecart.rs:983-985` both pass `entity.pos.load()` to `World::scatter_inventory_at`
  (`world/mod.rs:7696`), matching vanilla `Containers.dropContents(level, entity, container)`.
- Gap 5 (save/load test coverage): ALREADY FIXED in checkout, no new test needed.
  `boat.rs:1167-1184` `chest_boat_items_round_trip_through_last_slot` writes/reads an `Items` NBT
  list through slot 26 of a 27-slot inventory, and `:1156 poplar_chest_boat_has_container_but_plain_boat_does_not`
  asserts the 27-slot versus no-container split for `POPLAR_CHEST_BOAT` vs `POPLAR_BOAT`.

Net code changes this pass: none (no source basenames assigned to this unit). No rustfmt (no
.rs file touched). No build/test run (worker constraint); the orchestrator batch-compiles.


## Result

Decompiled tree is present this pass at `.vendor/vanilla-decompiled/net/minecraft/`. This unit
has no source basenames, so no .rs file may be edited; rows needing `boat.rs` or
`item/items/boat.rs` are left open with a reason. This section supersedes the older "Gap status"
and "Result" sections above.

- Gap 2 (BoatItem `minecraft:container` component): VERIFIED FALSE POSITIVE, left open OUT OF
  UNIT. Decompiled `BoatItem.getBoat` (BoatItem.java:93-104) only calls
  `EntityType.createDefaultStackConfig(...).apply(boat)`. That resolves to
  `appendComponentsConfig` -> `Entity.applyComponentsFromItemStack` ->
  `applyImplicitComponents` (EntityType.java:156-170, Entity.java:7215-7217), whose base
  `applyImplicitComponent` handles only `CUSTOM_NAME` and `CUSTOM_DATA` (Entity.java:7239-7249).
  `AbstractChestBoat` does not override `applyImplicitComponent`, so this target version does NOT
  copy `minecraft:container` contents on placement. Pumpkin ignoring the component is correct
  parity; no fix warranted, and the file (`item/items/boat.rs`, basename `boat.rs`) is outside
  this unit regardless.
- Gap 3 (hopper / worldly container): VERIFIED FALSE POSITIVE, left open OUT OF UNIT. Decompiled
  `HopperBlockEntity.getEntityContainer` (HopperBlockEntity.java:783-788) selects any entity that
  is a `Container` (`EntitySelector.CONTAINER_ENTITY_SELECTOR`); `getContainerAt` (754-765) feeds
  it to both push and pull. `AbstractChestBoat implements ContainerEntity` which `extends Container,
  MenuProvider` (ContainerEntity.java:32), so in vanilla a hopper above a chest boat CAN insert
  into and extract from all slots. Pumpkin exposing it via `has_container_inventory` matches this;
  no change needed, and the file (`boat.rs`, basename `boat.rs`) is outside this unit regardless.
- Gaps 1, 4, 5: not re-litigated; prior pass recorded them ALREADY FIXED in the checkout
  (stillValid validity closure in `minecart/container.rs`, `entity.pos.load()` scatter in
  `boat.rs`/`minecart.rs`, chest-boat NBT round-trip test in `boat.rs`). No change by this unit.

Net code changes: none (no source basenames assigned). No rustfmt (no .rs file touched). No
build/test run (worker constraint); the orchestrator batch-compiles.
