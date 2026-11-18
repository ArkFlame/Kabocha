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

## Gap status
- 1 contents drop nested in game rule (boat.rs:365-385): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 2 dropped item spawn position (boat.rs:366-382): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 3 stillValid/canPlayerUse (minecart/container.rs:202-233): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 4 BoatItem container component (item/items/boat.rs:107-200): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 5 hopper WorldlyContainer (boat.rs:278-296): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 6 save/load test coverage (boat.rs:441-470): STILL OPEN (needs test, unverified)


## Result

Re-verified against the (now populated) `.vendor/vanilla-decompiled` tree:
`world/entity/vehicle/ContainerEntity.java`, `VehicleEntity.java`,
`boat/AbstractChestBoat.java`, `minecart/AbstractMinecartContainer.java`,
`world/Containers.java`, `world/entity/Entity.java` (`spawnAtLocation`, `RemovalReason`).

- 1 contents drop gated by `doEntityDrops`: FIXED for chest minecarts (`minecart.rs`
  `damage_with_context`: the `claim_drops`/`scatter_inventory_at` block moved out of the
  `entity_drops` check; vanilla `AbstractMinecartContainer.remove` drops contents on any
  destroying removal while `VehicleEntity.destroy` gates only the cart item). The chest-boat side
  was already correct (`boat.rs` scatters before the gamerule check); `boat.rs` is outside this
  unit file list, no edit needed.
- 2 dropped item spawn position: FIXED for minecarts (`minecart.rs` now drops via
  `world.drop_stack_at(entity.pos)` instead of `block_pos`). `boat.rs` already used
  `drop_stack_at`.
- 3 `stillValid` / `canPlayerUse`: STILL OPEN. Needs `ScreenHandler::can_use` in
  `pumpkin-inventory` plus `MinecartScreenFactory` (`entity/vehicle/minecart/container.rs`) and
  `player.rs` reach logic; all outside this unit allowed files.
- 4 `BoatItem` `minecraft:container` component: STILL OPEN. Fix lives in
  `crates/pumpkin-core/src/item/items/boat.rs`, outside this unit.
- 5 hopper path over a chest vehicle: FIXED (verified). Vanilla chest vehicles are plain
  `Container` (not `WorldlyContainer`), so every slot is reachable from any face; Pumpkin
  `is_sided()` default `false` already gives that. Added
  `hopper_moves_items_both_ways_with_a_plain_entity_container` in
  `block/entities/hopper.rs` covering insertion from above and extraction from below against a
  27-slot `MinecartInventory`.
- 6 chest-boat NBT round-trip test: STILL OPEN. The test belongs to `boat.rs`, outside this unit.

`rustfmt --edition 2024` run on both touched files (exit 0). No cargo run (worker constraint).


## Result (re-review 2)

Re-checked each STILL OPEN row against the current tree; this unit may edit only
`hopper.rs`. All work achievable here was already landed by the prior pass and is
present in this checkout, verified by re-reading the files (no new code needed).

- 1 contents drop gated by `doEntityDrops`: already FIXED. `minecart.rs:980-996` moves
  `claim_drops`/`scatter_inventory_at` outside the `entity_drops` rule; `boat.rs` already
  scatters before the check. Both outside this unit's editable file, no action.
- 2 dropped item spawn position: already FIXED. `minecart.rs:996` uses
  `drop_stack_at(entity.pos)`; `boat.rs` already uses `drop_stack_at`. Outside this
  unit's editable file, no action.
- 3 `stillValid` / `canPlayerUse`: STILL OPEN, requires `ScreenHandler::can_use` in
  `pumpkin-inventory` plus `MinecartScreenFactory` and `player.rs`, all outside this unit.
- 4 `BoatItem` `minecraft:container` component: STILL OPEN, fix lives in
  `item/items/boat.rs`, outside this unit.
- 5 hopper path over a chest vehicle: already FIXED and verified. `hopper.rs:331-340`
  selects a live entity with `has_container_inventory()`; `is_sided()` default `false`
  gives all-slot access on every face, matching vanilla plain-`Container` behaviour.
  Regression test `hopper_moves_items_both_ways_with_a_plain_entity_container`
  (`hopper.rs:890`) covers insertion above and extraction below against a 27-slot
  `MinecartInventory`. Re-read; correct, no change needed.
- 6 chest-boat NBT round-trip test: STILL OPEN, test belongs to `boat.rs`, outside this unit.

No code edits this pass (only `hopper.rs` is in scope and it was already correct);
`rustfmt` not needed. No cargo run (worker constraint); orchestrator batch-compiles.


## Result (re-review 3)

Re-verified the STILL OPEN rows against the populated `.vendor/vanilla-decompiled` tree
(`world/level/block/entity/HopperBlockEntity.java`, `world/entity/vehicle/ContainerEntity.java`,
`world/entity/vehicle/boat/AbstractChestBoat.java`). Only `block/entities/hopper.rs` is in scope.

- 1 contents drop vs `doEntityDrops`: OUT OF SCOPE (lives in `boat.rs`/`minecart.rs`; prior passes
  already moved the cart drop outside the gamerule). No edit here.
- 2 dropped item spawn position: OUT OF SCOPE (`boat.rs`/`minecart.rs`; already uses
  `drop_stack_at(entity.pos)`). No edit here.
- 3 `stillValid`/`canPlayerUse`: OUT OF SCOPE. Needs `ScreenHandler::can_use` in
  `pumpkin-inventory` plus `MinecartScreenFactory` and `player.rs`, none of which are this unit's file.
  (Vanilla confirms the gap: `ContainerEntity.isChestVehicleStillValid` requires
  `!isRemoved() && player within 4.0 of the entity box`.)
- 4 `BoatItem` `minecraft:container` component: OUT OF SCOPE (`item/items/boat.rs`).
- 5 hopper path over a chest vehicle: FIXED and re-verified against vanilla. Vanilla
  `HopperBlockEntity.getEntityContainer` (783-788) uses the same +/-0.5 AABB, alive+Container
  filter (`EntitySelector.CONTAINER_ENTITY_SELECTOR`) and random pick as Pumpkin's
  `get_entity_container` (`hopper.rs:321-341`). `MinecartInventory` does not override `is_sided()`
  (default false) or `is_valid_slot_for()` (default true), matching a plain vanilla `Container`
  reachable from every face. Regression test
  `hopper_moves_items_both_ways_with_a_plain_entity_container` (`hopper.rs:890`) covers insertion
  above and extraction below. Re-read; correct, no change needed.
- 6 chest-boat NBT round-trip test: OUT OF SCOPE (test belongs to `boat.rs`).

No code edits this pass: every row fixable in `hopper.rs` (row 5) was already landed and now
re-verified against vanilla; the rest require files outside this unit. No cargo run (worker
constraint); orchestrator batch-compiles.

Status: PASS fixed=0 open=3 files=block/entities/hopper.rs (unchanged, re-verified)
