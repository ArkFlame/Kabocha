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
| 1 | `minecart/container.rs:202-233`, `boat.rs:96` | `AbstractChestBoat.canPlayerUse` / `AbstractContainerMenu.stillValid` | Entity-backed screen never becomes invalid: no 8-block distance or entity-liveness check, so the menu survives walking away, the boat being destroyed or unloaded. | Give `MinecartScreenFactory` a validity closure (`is_alive() && squaredDistance <= 64`) and close the menu when it fails; use it for chest boats and chest minecarts. | M |
| 2 | `item/items/boat.rs:187-198` | `BoatItem.use` + `minecraft:container` component (1.21.2+) | Placing a chest boat item ignores the `minecraft:container` component, so a serialized boat with contents spawns empty. | After `BoatEntity::new`, copy `ContainerImpl` stacks slot-for-slot (clamp 27) before `world.spawn_entity`. Confirm the target-version `BoatItem` still forwards the component. | M |
| 3 | `boat.rs:266-281`, hopper path | `AbstractChestBoat implements Container` only | `has_container_inventory` exposes the chest boat as a worldly container, so a hopper above may insert; vanilla chest boats are plain `Container` (extract only). Verify the hopper/worldly-container path before changing. | Gate insertion per vehicle kind if the hopper path currently allows it. | M |
| 4 | `boat.rs:348-354` | `Containers.dropContents(level, entity, container)` | Scattered contents use `entity.block_pos` (block origin) instead of the entity's exact `pos`. Cosmetic item-spawn offset only. | Add a position-based `World::scatter_inventory` variant and use `entity.pos` here and in `minecart.rs`. | S |
| 5 | `boat.rs:242-252` | `AbstractChestBoat` save/load | No automated test for the chest-boat 27-slot NBT round trip or the new open guard. | Unit test: `BoatEntity` for `PALE_OAK_CHEST_BOAT` vs `PALE_OAK_BOAT`, assert `has_container_inventory()` and an `Items` round trip through slot 26. | S |

Client-only effects (chest rendering, screen layout, paddle wobble) were not assessed.
No cargo/test run (worker constraint); the orchestrator batch-compiles.

