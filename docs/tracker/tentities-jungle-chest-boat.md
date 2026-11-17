# entities / Jungle Chest Boat (RE-REVIEW)

Reviewed: `crates/pumpkin-core/src/entity/vehicle/boat.rs`, `boat/passengers.rs`, `boat/physics.rs`, `boat/dismount.rs`, `entity/vehicle/minecart/container.rs`, `entity/type.rs`, `item/items/boat.rs`. No decompiled vanilla source is available under `.vendor/`, so parity below is against the vanilla classes named in each row.

No code change was made: the chest-boat paths line up with vanilla on the main behaviours, and the residual items are unverified details, not confirmed gaps.

Verified present (vanilla `ChestBoat` / `AbstractChestBoat`):

- `is_chest_boat` covers all 11 chest variants including jungle (`boat.rs:43-55`), and `entity/type.rs:332-356` routes the entity type to `BoatEntity`.
- 27-slot container (`boat.rs:39,74-75`), matching `AbstractChestBoat` `NonNullList.withSize(27, ...)`.
- Container persistence of `Items` / `LootTable` / `LootTableSeed` (`boat.rs:235-245` via `minecart/container.rs:40-81`), matching `ContainerHelper.saveAllItems` / `loadAllItems`.
- Break path drops container contents and then the wood-specific boat item, gated by `entity_drops` and creative (`boat.rs:315-362`); dropped item resolves from `entity_type.resource_name`, so `jungle_chest_boat` -> `Item::JUNGLE_CHEST_BOAT`.
- Right-click: sneaking or no free seat opens the chest, otherwise the player mounts (`boat.rs:364-400`); `get_max_passengers()` = 1 for chest boats (`boat.rs:94-96`) and the same limit is used for mob pickup (`entity/mod.rs:1236-1249`).
- Screen title `entity.minecraft.{resource_name}`, custom name honoured, `CONTAINER_OPEN` game event emitted on open (`boat.rs:98-125`, `minecart/container.rs:227`).
- Single-passenger x offset 0.15 from generated boat data (`passengers.rs:67-69`, assets `single_passenger_x_offset` 0.15), matching `AbstractChestBoat.getSinglePassengerXOffset`.
- Item mapping both ways (`item/items/boat.rs:30,53-58`), so placing/breaking/pick-block round-trip.

## Remaining gaps

| file:line | vanilla | suggested fix | size |
|---|---|---|---|
| boat.rs:364-380 | `ContainerEntity.interactWithChestVehicle` does not open a container for a spectator; Pumpkin's `container::open` only skips loot unpacking for spectators (`minecart/container.rs:209`) and still builds the screen | verify against decompiled `AbstractChestBoat`/`ContainerEntity`, then early-return `false` in `interact` for `player.is_spectator()` before `open_custom_inventory_screen` | S (verify) |
| boat.rs:332-358 | `Boat.hurt` calls `destroy(source)` for a creative (instabuild) break when `DOENTITYDROPS` is on, and `AbstractChestBoat.destroy` spills the container; Pumpkin skips the container drop whenever `creative` is true | confirm vanilla creative chest-boat spill; if contents do drop, remove `!creative` from the container-drop gate only (keep it for the boat item) | S (verify) |
| boat.rs:351-355 | `AbstractBoat.destroy` is `spawnAtLocation(this.getDropItem())`, which does not copy the entity custom name onto the dropped item; Pumpkin sets `CustomNameImpl` on the drop | confirm in decompiled `Boat`/`AbstractBoat`; if no name transfer exists, drop the `CustomNameImpl` block | S (verify) |
| boat.rs:335 vs 356 | vanilla `chestVehicleDestroyed` scatters the container at the entity position (`getX/Y/Z`); Pumpkin passes `block_pos` to `world.scatter_inventory` | pass the entity position to `scatter_inventory` and keep `block_pos` only for `drop_stack` | S |
| boat.rs:276-301 | `AbstractBoat` also maintains bubble/air state while a player rides underwater; no `DATA_ID_BUBBLE_TIME` (tracked_data id 13) is ever sent for any boat | set/publish bubble-time metadata from the underwater tick, shared by all boat types | M |
