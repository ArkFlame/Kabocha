# entities / Spruce Chest Boat — re-review

Source: `crates/pumpkin-core/src/entity/vehicle/boat.rs` (shared by every boat/chest boat; latest
touched by `570d3ea1f` "feat: improve boat water physics", which split out
`boat/{physics,block_effects,passengers,dismount}.rs`). Placement:
`crates/pumpkin-core/src/item/items/boat.rs`, dispatch: `entity/type.rs:332`,
variant mapping: `boat.rs:43-56`.
Reference: vanilla `net.minecraft.world.entity.vehicle.boat.AbstractBoat` and
`AbstractChestBoat` (1.21.9+ container-entity line). No decompiled sources in `.vendor/`, so each
row names the vanilla member it must match.

## Fixed this pass
Nothing to fix. Every chest-boat-specific behaviour that the vanilla classes define is present and
matches; see the checklist below. No code change was made, so no behaviour could regress.

## Verified as matching (chest boat specifics)

| Behaviour | Rust | Vanilla member |
|---|---|---|
| 27-slot container, created per chest boat, saved/loaded as `Items`/`LootTable`/`LootTableSeed` | `boat.rs:39`, `boat.rs:65-68`, `boat.rs:235-245`, `minecart/container.rs:39-79` | `AbstractChestBoat` container + `ContainerHelper.saveAllItems` |
| Right-click opens the chest only with a full seat or while sneaking, otherwise boards | `boat.rs:364-400` | `AbstractChestBoat.interact` |
| Open while riding via the vehicle-inventory key | `player.rs:7580-7592` -> `boat.rs:98-125` | `AbstractChestBoat.openCustomInventoryScreen` |
| Screen title is the entity display name, custom name wins; bamboo chest raft uses the raft name | `boat.rs:102-113`, `minecart/container.rs:203-233` | `AbstractChestBoat.getDisplayName`, `Entity.getCustomName` |
| `CONTAINER_OPEN` game event at the boat with the player as source | `boat.rs:114-124` | `GameEvent.CONTAINER_OPEN` emission on open |
| One seat only; chest boats take no second passenger and pick up no mob | `boat.rs:94-96`, `entity/mod.rs:1237-1262` | `AbstractChestBoat` single-seat rule |
| Single-rider offset 0.15 | `passengers.rs:66-69` (chest boat branch), extracted `BoatSeats.single_passenger_x_offset = 0.15` | `AbstractBoat.getPassengerAttachmentPoint` + chest-boat offset |
| Eye-in-water / 60-tick underwater ejection | `boat.rs:144-167`, `physics.rs` `is_eye_in_water` | `AbstractBoat.tick` `ticksUnderwater` |
| Destruction drops the boat item (with custom name) plus scattered contents, respecting `entity_drops`; creative destroys without drops | `boat.rs:315-362` | `AbstractBoat.hurtServer`/`destroy`, `AbstractChestBoat.dropContents` |
| Lava ignition/damage reaches boats through `Entity::base_tick` -> `update_fluid_state` | `boat.rs:276-301` -> `entity/mod.rs:6806-6818` | `Entity.baseTick` fluid `entityInside` |
| Place item -> entity, space/collision checks, yaw, stack decrement, `ENTITY_PLACE` event | `item/items/boat.rs:107-204` | `BoatItem.use` |
| Pick-block returns the matching chest boat item | `pick_item.rs:100` via `item/items/boat.rs:53-58` | `AbstractBoat.getPickResult` |
| All 11 chest-boat/raft entity types mapped to `BoatEntity` and to their items | `entity/type.rs:332-354`, `item/items/boat.rs:21-98`, `boat.rs:43-56` | `EntityType` registrations |

## Remaining gaps

| # | Where | Vanilla member | Gap | Fix | Size |
|---|-------|----------------|-----|-----|------|
| 1 | `boat.rs:287-298` | `AbstractBoat.tick` float/move | When a player rides, the server zeroes velocity and skips `float_boat`/`move_boat` entirely, so the server-side boat only follows client packets: block effects, status and underwater counters advance but boat-vs-block/piston/entity resolution does not. Affects chest boats with a rider identically to plain boats (known item, `docs/LEDGER.tsv` #1679, and `AGENTS.md` makes player movement client-authoritative) | Port the vanilla input-driven `float`/`move` path instead of the early return; needs a client/multiplayer repro | M |
| 2 | `passengers.rs:68` | `AbstractBoat.getSinglePassengerXOffset` | Offset is hardcoded to `0.15` for chest boats while the extracted `BoatSeats.single_passenger_x_offset` (already used at `boat.rs:198`) carries the same value; two sources for one number can diverge when `assets/` changes | Read `entity.entity_type.boat?.single_passenger_x_offset` here and drop the `is_chest_boat` branch | S |
| 3 | `boat.rs:347-357` | `AbstractBoat.getPickResult` | Drop item resolves via `Item::from_registry_key(resource_name)` instead of the existing `BoatItem::entity_to_item` (`item/items/boat.rs:53`), so the item mapping lives twice | Use `BoatItem::entity_to_item(self.vehicle.entity.entity_type)` and fall back to the registry lookup | S |
| 4 | `player.rs:4377-4385` | `Player.getMovementStatistic` (`getVehicle() instanceof AbstractBoat`) | The chest-boat list omits `POPLAR_CHEST_BOAT`, and neither `minecraft:boat` nor `c:boats` contains `poplar_chest_boat`, so riding a poplar chest boat awards no `minecraft.custom:minecraft.boat_one_cm`. Spruce is listed, so this row's entity is unaffected | Add `entity_type == &EntityType::POPLAR_CHEST_BOAT`; ideally replace the whole hand list with the `c:boats` tag once the tag data covers poplar | S |
| 5 | `boat.rs:43-56`, `item/items/boat.rs:21-98`, `entity/type.rs:332-354`, `entity/mod.rs:1239`, `player.rs:4377-4385` | `AbstractChestBoat` subclasses | Five hand-maintained lists of the 11 chest-boat types must be edited together when a wood type is added; missing one silently drops a behaviour (see #4) | Add one `BoatKind`/flag on `EntityType` (codegen from `resource_name` suffix) and derive all five checks from it | M |
| 6 | `boat.rs:259-274`, `minecart/container.rs:88-120` | 1.21.5+ container-entity loot tables | Chest boats reuse `MinecartInventory`, so they accept `LootTable`/`LootTableSeed`. Correct only if vanilla applies container loot tables to chest boats; otherwise an empty chest boat summoned with `LootTable` would generate loot where vanilla ignores it | Confirm against the decompiled `AbstractChestBoat`; if chest boats are plain `SimpleContainer`, add a `loot_table: false` mode | M |

## Not gaps (checked)
Chest-boat interact ordering, sneaking open path, E-key open while riding, 27 slots, `Items` NBT
round trip, drop-with-contents on destruction, creative no-drop, custom name on the dropped item,
seat count 1 for both player boarding and mob pickup, single-rider 0.15 offset, `blocksBuilding`,
`can_hit`/`is_collidable`/`is_pushable`, item placement volume/collision checks, pick-block item,
bedrock `entity.chest_boat.name`/`entity.chest_raft.name` keys, spectator denial (handled centrally
at `player.rs:7560-7570`).
