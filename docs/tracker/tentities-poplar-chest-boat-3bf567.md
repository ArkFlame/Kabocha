# entities / Poplar Chest Boat (RE-REVIEW)

Re-review after the shared chest-boat source changed (`bc9600f36` drop item, `cd310a832`
secondary-use open, `65d819a2c` damage invulnerability gate, `1b0b6471f` no container-open
event, `b114b29da` bubble-time sync, `64ba0df4b` one-passenger cap). Shared implementation:
`crates/pumpkin-core/src/entity/vehicle/boat.rs`,
`entity/vehicle/minecart/container.rs`, `item/items/boat.rs`,
generated `pumpkin-data/src/generated/entity_type.rs` (`POPLAR_CHEST_BOAT`, id 107).
`.vendor/` is empty in this checkout, so the comparison below is against
`net.minecraft.world.entity.vehicle.AbstractChestBoat` / `AbstractBoat` / `BoatItem`
behaviour, not quoted decompiled source.

## Verified parity (current code)

| Behaviour | Pumpkin |
|---|---|
| `poplar_chest_boat` is a boat entity with generated seat data | `entity_type.rs:9187-9215` (`boat: Some(BoatSeats { ride_height 0.1875, .. })`) |
| Chest variant detection covers every variant including poplar | `boat.rs:43-55` (+ test asserting suffix coverage) |
| 27-slot container only for chest boats | `boat.rs:37,84-85`; `minecart/container.rs:19-33` |
| One passenger for chest boats, two for plain boats | `boat.rs:57-65,390` |
| Snorkel/boarding gate `ticks_underwater < 60`, bubble metadata | `boat.rs:157-189,390-402` |
| Secondary-use-only open, spectator/alive/occupancy guard | `boat.rs:100-138,404-407` |
| Title `entity.minecraft.<variant>`, Bedrock raft key, custom name wins | `boat.rs:125-134`; `minecart/container.rs:227` |
| No `GameEvent.CONTAINER_OPEN` for entity containers | `minecart/container.rs:202-233` |
| Break threshold `damage + amount*10 > 40`, creative instabuild, invulnerability first | `boat.rs:344-363` |
| Contents scatter at exact entity `pos`; boat item dropped non-creative only | `boat.rs:365-385` |
| `LootTable`/`LootTableSeed` deferred unpack, `Items` at container root | `minecart/container.rs:40-81` |
| Item -> entity mapping and pick result for poplar chest boat | `item/items/boat.rs:41-42,63-71,91-92` |
| Boat movement statistic includes `POPLAR_CHEST_BOAT` | `entity/player.rs` (earlier fix) |

## Fixed this pass

- `minecart/container.rs:58-84` — the shared chest-vehicle NBT writer always emitted an
  `Items` list, even for an empty 27-slot container. Vanilla
  `AbstractChestBoat.addAdditionalSaveData` (and `AbstractMinecartContainer`) only writes the
  list when at least one slot is filled. Now omitted when empty, with a regression test
  `empty_container_writes_no_items_list`. Commit: fix(entity): omit empty chest vehicle items list.

## Remaining gaps

| # | file:line | vanilla class | gap | suggested fix | size |
|---|---|---|---|---|---|
| 1 | `boat.rs:365-385` | `ContainerEntity` / `Containers.dropContents` | Contents drop is nested inside `game_rules.entity_drops`; vanilla container vehicles drop contents independently of `doEntityDrops` (only the vehicle item is gated). Verify against decompiled `AbstractChestBoat`/`chestVehicleDestroyed`, then move the `claim_drops`/`scatter_inventory_at` block out of the gamerule check. | S (verify) |
| 2 | `boat.rs:366-382` | `AbstractBoat.destroy` -> `spawnAtLocation` | Dropped chest-boat item spawns at `entity.block_pos` (integer), while vanilla uses the entity's exact position; contents already use `entity.pos`. | Add a position-based `World::drop_stack` (or reuse the scatter spawn helper) and use `entity.pos` here and in `minecart.rs:897`. | S |
| 3 | `minecart/container.rs:202-233`, `boat.rs:100` | `AbstractContainerMenu.stillValid` / `Container.canPlayerUse` | Entity-backed screen never invalidates: no 8-block distance, entity-alive or unload check, so the 27 slots stay usable after walking away or after the boat is destroyed. | Give `MinecartScreenFactory` a validity check (`is_alive() && distance_squared <= 64`) and close the screen when it fails; applies to chest boats and chest minecarts. | M |
| 4 | `item/items/boat.rs:107-200` | `BoatItem.use` + `minecraft:container` component (1.21.2+) | Placing a chest-boat item ignores its `minecraft:container` component, so a component-carrying chest boat spawns empty. | After `BoatEntity::new`, copy `ContainerImpl` stacks slot-for-slot (clamp to 27) before `world.spawn_entity`. | M |
| 5 | `boat.rs:278-296`, `hopper.rs:328-350` | `AbstractChestBoat implements Container` (no `WorldlyContainer`) | `has_container_inventory` puts chest boats on the hopper entity path; confirm insertion above a boat matches vanilla (plain `Container`, so all slots are reachable) rather than extraction-only. | Add a hopper test: push into a chest boat from above and pull from below; adjust the entity container path if insertion is missing. | M |
| 6 | `boat.rs:441-470` | `AbstractChestBoat` save/load | Tests only cover chest-boat type detection; there is no chest-boat 27-slot NBT round trip or open-guard test. | Unit test: `POPLAR_CHEST_BOAT` vs `POPLAR_BOAT` `has_container_inventory()`, and `Items` round trip through slot 26. | S |

Client-only effects (chest rendering, screen layout, paddle wobble) were not assessed.
No cargo or test run was performed (worker constraint); the orchestrator batch-compiles.
