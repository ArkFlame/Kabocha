# Tracker: entities / Pale Oak Chest Boat

Reviewed: `crates/pumpkin-core/src/entity/vehicle/boat.rs` (whole file),
`crates/pumpkin-core/src/entity/vehicle/boat/passengers.rs`,
`crates/pumpkin-core/src/entity/type.rs:319-355`,
`crates/pumpkin-core/src/item/items/boat.rs:21-98`,
generated `crates/pumpkin-data/src/generated/entity_type.rs:7987-8043`,
at branch `kab/tentities-pale-oak-chest-boat` (after `feat: improve boat water physics`
#4073, `fix(entity): burn boats in lava` #4386, `feat(inventory): implement 26.3 worldly
container ... vehicle container parity`).
Vanilla reference: `net.minecraft.world.entity.vehicle.AbstractChestBoat`,
`AbstractBoat` and `net.minecraft.world.item.BoatItem` (1.21.x). No `.vendor` tree is
present in this checkout, so parity below was checked against the vanilla class bodies.

## Verified parity (no change needed)

| Behaviour | Pumpkin |
|---|---|
| Entity type is created as a boat, not a generic entity | `entity/type.rs:319-355` lists `PALE_OAK_CHEST_BOAT`. |
| Chest boat is a chest variant | `boat.rs:43-55` `is_chest_boat` includes `PALE_OAK_CHEST_BOAT` (and `BAMBOO_CHEST_RAFT`). |
| 27-slot container, `Items` NBT, loot table/seed, `claim_drops` | `boat.rs:74-75, 235-245` + `MinecartInventory` (`minecart/container.rs:40-80`). |
| One seat, not two | `boat.rs:94-96` (`get_max_passengers` = 1 for chest boats). |
| Chest-boat single-rider X offset 0.15 and `height / 3` ride height (bamboo `* 0.8888889`) | `passengers.rs:53-80`; `AbstractBoat#getPassengerAttachmentPoint/getRideYOffset`. |
| Open container when sneaking or when the seat is taken | `boat.rs:364-400`. |
| Container title `entity.minecraft.<type>`, custom name wins, container-open game event | `boat.rs:98-125`, custom name applied in `container::open` (`minecart/container.rs:227`). |
| Breaking drops the chest contents (scattered) and the boat item, honouring `doEntityDrops` and creative | `boat.rs:315-362`; chest boat item resolved from `resource_name` = `pale_oak_chest_boat`. |
| Boat item places the matching entity and consumes one | `item/items/boat.rs:39-40` (map), `:89-94` (ids), `:152-203` (`use`). |
| Entity dimensions 1.375 x 0.5625, eye 0.5625, single_passenger_x_offset 0.15 | generated `entity_type.rs:8023-8042`; matches `EntityType.CHEST_BOAT`. |
| Passengers persisted (`Passengers` NBT) so a ridden chest boat survives restart | `entity/mod.rs:5989`. |

The previous "partial" assessment predates #4073/#4386 and the vehicle-container parity
work (water physics, lava destruction, container open/game event, break drops). Those fill
the gaps the row was tracking.

## Remaining gaps

### 1. Chest boat opens its container when the seat is occupied and the player is not sneaking - size S

`crates/pumpkin-core/src/entity/vehicle/boat.rs:377`

```rust
if self.inventory.is_some() && (!can_add_passenger || player.get_entity().is_sneaking()) {
    let _ = self.open_custom_inventory_screen(player);
    return true;
}
```

Vanilla `AbstractChestBoat.interact` delegates to `AbstractBoat.interact`, which only opens
the container for `player.isSecondaryUseActive()` (sneak); when the boat is full and the
player is not sneaking the interaction is `PASS` (nothing happens). Pumpkin additionally
opens the inventory for a non-sneaking player whenever the 1 chest-boat seat is taken.

Fix (needs the vanilla method body confirmed first, it is the only item here that could not
be verified without the decompiled tree): gate the open on `player.get_entity().is_sneaking()`
and let the full-boat case fall through to `return false`.

### 2. No automated test for chest-boat container persistence - size S

`crates/pumpkin-core/src/entity/vehicle/boat.rs:235-245`

`write_custom_nbt`/`read_custom_nbt` delegate to `MinecartInventory`, but nothing exercises
the boat path with the 27-slot size. Add a `pumpkin-gametest`/unit test that builds a
`PALE_OAK_CHEST_BOAT` `BoatEntity`, writes an `ItemStack` into slot 26, round-trips the NBT
and asserts the stack (and that a plain `OAK_BOAT` has no inventory).

### 3. Client-only behaviour not verifiable here

Container screen title layout, chest-boat rendering and paddle wobble are client-side; not
audited, not blocking.
