# entities / Jungle Chest Boat (RE-REVIEW)

Reviewed: `crates/pumpkin-core/src/entity/vehicle/boat.rs`, `boat/passengers.rs`, `boat/physics.rs`, `boat/dismount.rs`, `boat/block_effects.rs`, `entity/type.rs`, `item/items/boat.rs`, `entity/vehicle/minecart/container.rs`, `net/java/play/interact.rs`. No decompiled vanilla source under `.vendor/`, so parity is against vanilla `Boat`, `AbstractBoat`, `AbstractChestBoat`, `ChestBoat`, `JungleChestBoat`, `ContainerEntity`.

The source changed after the previous review (chest boats now seat two, creative break no longer spills contents, container-open event, boarding gates). Two gaps from that review are now closed, and one of its items was a false positive:

- Spectator opening: `interact.rs:39-43` sets the camera target and returns for spectators, and `open_custom_inventory_screen` also rejects spectators (`boat.rs:95`). Not a gap.
- Container scatter now uses the entity position (`boat.rs:357`), matching `chestVehicleDestroyed`.
- Two-passenger seating restored (`boat.rs:41,214-227`, `passengers.rs:53-64`); chest boats keep the chest and still seat two, as vanilla `AbstractBoat.getMaxPassengers` = 2.

Verified present (vanilla `ChestBoat` / `AbstractChestBoat` / `JungleChestBoat`):

- `is_chest_boat` covers all 11 chest variants including jungle (`boat.rs:43-56`); `entity/type.rs:334-357` routes `jungle_chest_boat` to `BoatEntity` (lines 340-341).
- 27-slot inventory on chest variants only (`boat.rs:39,75-76`), matching `AbstractChestBoat` `NonNullList.withSize(27, ...)`; `Items` / `LootTable` / `LootTableSeed` persistence (`boat.rs:245-255`).
- Break path: creative gate, `entity_drops` gate, loot unpack, contents scattered at the entity position, then the variant boat item from `entity_type.resource_name` so jungle drops `Item::JUNGLE_CHEST_BOAT` (`boat.rs:325-373`).
- Open gates: alive, no other rider, else empty/ridden by opener (`boat.rs:91-114`); title `entity.minecraft.jungle_chest_boat` with chest-raft bedrock fallback (`boat.rs:115-125`); `CONTAINER_OPEN` game event on success (`boat.rs:127-136`).
- Right-click: sneak or no free seat opens the chest, otherwise mount, with the 60-tick out-of-control gate (`boat.rs:375-411`).
- Item mapping both ways for jungle (`item/items/boat.rs:29-30,79-80,227-242`), so place/broken-drop/pick round-trip.
- Seat offsets from generated boat data, second-seat animal shift (`boat.rs:206-232`).

## Remaining gaps

| file:line | vanilla class | suggested fix | size |
|---|---|---|---|
| boat.rs:325-373 | `Boat.destroy` / `AbstractChestBoat.destroy` | confirm whether a creative (instabuild) hit spills chest contents in vanilla; Pumpkin skips both contents and item (`!creative` at 340 and 344). If vanilla spills contents on a creative break, drop only the boat item gate | S |
| boat.rs:359-368 | `Boat.destroy` (`spawnAtLocation(getDropItem())`) | vanilla does not copy the entity custom name onto the dropped boat item; Pumpkin adds `CustomNameImpl`. Remove the block if decompiled source confirms no name transfer | S |
| boat.rs:157-180 | `AbstractBoat` bubble time (`DATA_ID_BUBBLE_TIME`) | boat tracked data has no bubble-time id, so underwater passengers never see the vanilla bubble countdown; publish it from the underwater tick | M |
