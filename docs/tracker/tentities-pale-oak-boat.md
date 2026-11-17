# entities / Pale Oak Boat — re-review

Source: `crates/pumpkin-core/src/item/items/boat.rs`, `crates/pumpkin-core/src/entity/vehicle/boat.rs`
(and submodules `physics.rs`, `passengers.rs`, `dismount.rs`, `block_effects.rs`). Last structural change
touching this row: 570d3ea1f "feat: improve boat water physics (#4073)", which rewrote the boat tick,
floating and collision code. Reference: vanilla `net.minecraft.world.entity.vehicle.boat.BoatEntity`
(+ `AbstractBoat` seating/attachment members) for the current MC target. No decompiled sources in
`.vendor/`, so each row names the vanilla member it must match.

Pale oak is not a special-cased wood: it rides the shared boat entity and shared `BoatItem`. The audit
therefore covers (a) every place a pale-oak boat must be registered, and (b) the shared boat behaviour
that the 570d3ea1f rewrite changed.

## Wiring verified (no gaps)

| Where | Check |
|---|---|
| item/items/boat.rs:39-40 | `PALE_OAK_BOAT` / `PALE_OAK_CHEST_BOAT` items map to their entity types |
| item/items/boat.rs:89-90 | both ids registered through `ItemMetadata::ids`, and `mod.rs:158` registers one `BoatItem` for the whole list |
| entity/type.rs:344-345 | both entity types construct `BoatEntity` |
| entity/vehicle/boat.rs:52-53 | `is_chest_boat` includes `PALE_OAK_CHEST_BOAT` (27-slot inventory, 1 passenger) |
| entity/vehicle/boat.rs:43-55 | non-chest pale oak boat keeps 2 passengers |
| net/java/play/pick_item.rs:100 | middle-click pick block falls back to `BoatItem::entity_to_item`, which resolves pale oak |
| entity/player.rs:4375,4385 | `boat_one_cm` statistic includes both pale oak types |
| block/blocks/redstone/dispenser.rs:326-329 | dispenser boat path iterates `BoatItem::ids()`, so pale oak is dispensed |
| entity/vehicle/boat/passengers.rs:67-69,71-79 | single-rider offset 0.15 for chest boats, ride height `height/3` (raft exception) matches vanilla `AbstractBoat.getPassengerAttachmentPos` / `getRideHeight` |
| entity/vehicle/boat.rs:347-357 | boat break drop resolves the item from the entity registry key; `pumpkin-data` has no `entities/<boat>` loot table, so this matches vanilla `BoatEntity` dropping its item rather than a loot table |

## Remaining gaps

| # | Where | Vanilla member | Gap | Fix | Size |
|---|-------|----------------|-----|-----|------|
| 1 | entity/vehicle/boat.rs:377-380 `interact` | `BoatEntity.interact` chest-boat branch | The container-open result is discarded (`let _ = self.open_custom_inventory_screen(player); return true;`). A failed open (out of reach, plugin denied, screen already open) is reported to the client as success, so no fallback happens and the interaction is silently swallowed | Return the result of `open_custom_inventory_screen` (and let the caller fall through to riding when it is `false`); confirm against the decompiled chest-boat `interact` whether a failed open should return `PASS` | S |
| 2 | entity/vehicle/boat.rs:365-366 `interact` | `BoatEntity.interact` mount gate | Mounting is additionally blocked when the player's eye is inside a water block (`!self.is_eye_in_water()`). Vanilla gates only on `ticksUnderwater < 60`, "not already riding" and seat availability, so a player whose eye is submerged next to an empty boat cannot board it here but can in vanilla | Drop the `is_eye_in_water()` term from `can_add_passenger` (or move the check to where vanilla actually uses it) | S |
| 3 | entity/vehicle/boat.rs:327-332 `damage_with_context` | `BoatEntity.damage` / `dropItems` | Break detection is a pre-computed prediction (`will_break`) that must still agree with `VehicleEntity::damage_with_context` (vehicle.rs:243). They currently agree (`damage + amount*10 > 40`), but the coupling is implicit: any change to the vehicle threshold silently stops boat + chest contents from dropping. Vanilla decides once, inside `damage` | Have `vehicle.damage_with_context` return whether it destroyed the entity, or drop from a single post-damage check (`is_removed() && !creative`) instead of the pre-computed flag | S |
| 4 | entity/vehicle/boat/physics.rs:172-213 `get_ground_friction` | `AbstractBoat.getGroundFriction` | `friction / count as f32` yields `NaN` when no block under the boat has a collision shape; it is only harmless because entity/vehicle/boat.rs:84 tests `friction > 0.0`. Vanilla sums then divides only inside the block loop's non-empty case | Guard `count == 0` and return `0.0` explicitly; add a unit test for the empty case | S |
| 5 | entity/vehicle/boat.rs:288-289 tick | `BoatEntity.tick` client-controlled branch | A client-controlled boat has its velocity zeroed every tick (`entity.velocity.store(Vector3::default())`) but `last_yd` is never updated from the client move, so `get_water_level_above` (physics.rs:146) keeps using a stale `last_yd` after a client move until the boat becomes server-controlled (`move_from_client`, physics.rs:319, does store it) | Store `last_yd` in the client-controlled branch from the accepted `move_from_client` delta as well, and confirm the vanilla client-authoritative path | M |

## Not gaps (checked)

Pale oak tag membership in `minecraft:boat`/`c_boats` statistics paths, chest boat inventory size 27,
`TIME_TO_EJECT` 60-tick underwater ejection, custom-name preservation on both placement
(`spawn_egg::apply_entity_variant`, spawn_egg.rs:138-146) and break drop (boat.rs:351-355), wobble
metadata pairing (vehicle.rs:167-188), seat indices 0.2/-0.6 for two passengers, dismount spot search,
`blocks_building = true` on construction, and dispenser spawning.
