# entities / Acacia Chest Boat - audit (RE-REVIEW-UNTRACKED)

Re-audited after the source changed (commits 65d819a2c, cd310a832, bc9600f36 this branch).
Reviewed `crates/pumpkin-core/src/entity/vehicle/boat.rs`, `boat/{passengers,dismount,physics,block_effects}.rs`,
`crates/pumpkin-core/src/item/items/boat.rs`, `entity/type.rs` boat routing, `entity/mod.rs:1285-1292`
(mob boarding seat limit), `entity/vehicle/minecart.rs:891-897` (sibling destroy drop).
No `.vendor/` decompiled tree in this worktree, so each row names the vanilla member it must match:
1.21.x `AbstractBoat` / `AbstractChestBoat` / `BoatItem`.

## Verified correct (no change needed)
| Where | Vanilla member | Note |
|---|---|---|
| boat.rs:43-55 `is_chest_boat` | `AbstractChestBoat` type set | All 11 chest boats/rafts incl. `ACACIA_CHEST_BOAT`, `PALE_OAK`, `POPLAR`; unit test iterates `EntityType::ALL`. |
| boat.rs:59-65, entity/mod.rs:1285-1292, boat.rs:402 | `Boat.getMaxPassengers` | One shared `max_passengers(&EntityType)` helper (1 for chest boats, 2 otherwise) used by both the player and mob boarding paths. |
| boat.rs:84-85 + generated `entity_type.rs` | `AbstractChestBoat` ctor | 27-slot `MinecartInventory` only for chest boats; acacia data has `ride_height 0.1875`, `single_passenger_x_offset 0.15`, attachment `[0,0.5625,0]`. |
| boat.rs:100-138 | `AbstractChestBoat.openCustomInventoryScreen` | Spectator/occupied-by-other denied, custom name honoured via translate_cross, no `CONTAINER_OPEN` game event (entity containers do not emit it). |
| boat.rs:254-264 | `AbstractChestBoat.add/readAdditionalSaveData` | `Items` + `LootTable` round-trip through `MinecartInventory`, loot unpacked on access. |
| boat.rs:350-357 | `Boat.hurtServer` | Invulnerability check before wobble; break threshold `damage + amount*10 > 40` or creative. |
| boat.rs:406-412 | `AbstractChestBoat.interact` / `Boat.interact` | Secondary use opens, plain use boards a free seat (`ticks_underwater < 60`, no existing vehicle) or does nothing. |

## Fixed this pass
| # | Where | Vanilla member | Gap | Fix |
|---|-------|----------------|-----|-----|
| 1 | boat.rs:379-393 | `AbstractBoat.destroy` | `bc9600f36` removed the `CustomName` copy from the dropped boat item, contradicting vanilla (`if (hasCustomName()) stack.set(CUSTOM_NAME, getCustomName())`) and the sibling chest-minecart drop (`minecart.rs:891-897`); a renamed chest boat lost its name on break. | Restored the `CustomNameImpl` copy on the dropped stack. |
| 2 | boat.rs:381-383 | `AbstractBoat.destroy` (`!source.is(DamageTypeTags.IS_EXPLOSION)`) | The boat item dropped for every destroying damage source, including explosions, where vanilla drops no item. | Gate the item drop on `!damage_type.has_tag(tag::MINECRAFT_IS_EXPLOSION)`; contents scatter unchanged. |

## Remaining gaps
| # | Where | Vanilla member | Gap | Suggested fix | Size |
|---|-------|----------------|-----|---------------|------|
| 1 | boat.rs:359-397 | `AbstractChestBoat.remove(RemovalReason)` / `chestVehicleDestroyed` | Contents and the boat item drop only from the damage path. A destroying removal that bypasses `hurtServer` (out-of-world teleport, command `discard`, chunk cleanup) deletes the boat and its 27 slots silently. | Hook entity removal where `removal_reason.should_destroy()`, scatter the container once, and keep the item drop owned by the damage path. | M |
| 2 | item/items/boat.rs:187-198 (`BoatItem::use`) | `BoatItem.use` + `minecraft:container` component | Placing a chest boat item never reads the `minecraft:container` component, so a chest boat pick-blocked/serialized with contents spawns empty. | After `BoatEntity::new`, copy the `ContainerImpl` stacks into the 27 slots (clamped) before `world.spawn_entity`; confirm the 26.3 place path first. | M |
| 3 | boat.rs:384-388 (`Item::from_registry_key`) vs `pick_item.rs:100` (`BoatItem::entity_to_item`) | `AbstractBoat.getDropItem` / `getPickResult` (single mapping) | The drop path resolves the item from the entity `resource_name` while the pick path uses the item mapping; a variant whose keys diverge would drop nothing. | Use `BoatItem::entity_to_item` in `destroy` too. | S |
| 4 | block/entities/hopper.rs:342 (`has_container_inventory`) | `HopperBlockEntity.getContainerAt` | Chest boats expose `MinecartInventory` to hoppers with no `WorldlyContainer` face rules. Vanilla hoppers accept a plain `Container`, so this may already match. | Verify hopper insert/extract against vanilla before changing; gate only if a face-restricted path is reached. | S |
| 5 | entity/player.rs `open_handled_screen` / `container::open` | Bedrock `ContainerOpen` packet | Entity-backed containers are sent with `target_entity_id: -1` and `BlockPos::ZERO`, so a chest boat UI is not tied to the vehicle on Bedrock. | Thread an optional `target_entity_id` through and pass the boat entity id. | M |
| 6 | tests | - | No automated coverage for chest-boat contents. | `pumpkin-gametest`: spawn an acacia chest boat, fill a slot, break it with a player, assert contents + boat item with the custom name; assert nothing on a creative break or an explosion break. | M |

## Notes
- No cargo/test run and no runtime repro (worker constraint); the fixes are source-level parity changes against `AbstractBoat.destroy`.
- Client-predicted boat motion, seat rendering and paddle animation are out of scope for this row.
