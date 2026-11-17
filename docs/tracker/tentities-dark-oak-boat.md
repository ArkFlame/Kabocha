# Tracker: entities / Dark Oak Boat (RE-REVIEW)

Source: `crates/pumpkin-core/src/entity/vehicle/boat.rs` + `boat/{physics,block_effects,dismount,passengers}.rs`,
item side `crates/pumpkin-core/src/item/items/boat.rs`, registration `entity/type.rs:335-350`,
dispenser `block/blocks/redstone/dispenser.rs:529-562`, pickup `net/java/play/pick_item.rs:96-100`.
Vanilla reference classes: `net.minecraft.world.entity.vehicle.AbstractBoat` / `Boat`, `net.minecraft.world.item.BoatItem`,
`net.minecraft.core.dispenser.BoatDispenseItemBehavior` (1.21.x).
No `.vendor/` decompiled tree in this worktree; items marked *verify* need the decompiled class before editing.
The boat code is per-wood-type generic (`item_to_entity` / `is_chest_boat` / `entity_type.boat` seat data), so
Dark Oak has no special-case code: everything below applies to `EntityType::DARK_OAK_BOAT` and
`DARK_OAK_CHEST_BOAT` equally. No small certain gap was found in this pass, so no code commit.

## Remaining gaps

| # | file:line | Vanilla class / behaviour | Suggested fix | Size |
|---|---|---|---|---|
| 1 | entity/vehicle/boat.rs:364-397 | `AbstractBoat.interact`: only `player.isSecondaryUseActive()` + `startRiding`, where the passenger cap comes from `canAddPassenger`. The Rust path adds an extra `ticksUnderwater < 60` and `!is_eye_in_water()` gate, so a boat that is partially submerged (or just under a waterfall) refuses to take a rider, while vanilla still mounts and only ejects after 60 ticks via `tick` (`boat.rs:143-164`, already ported). *verify* against the decompiled class, then drop the two extra conditions from `can_add_passenger` and keep them only in `tick_underwater` | S |
| 2 | entity/vehicle/boat.rs:315-356 | `AbstractBoat.destroy(DamageSource)` drops `getPickResult()` with `spawnAtLocation` at the entity position and `Containers.dropContents(level, this, inventory)` for the chest contents; both use the entity centre. The Rust path uses `block_pos` for `scatter_inventory` + `drop_stack`, so a boat destroyed mid-air/on a slope drops the item into a different block, and chest contents are scattered from a block corner instead of the boat centre. Use `self.vehicle.entity.pos.load()` for both, and a single `drop_stack`-style `spawnAtLocation`. *verify* drop position vs decompiled `destroy` | S |
| 3 | entity/vehicle/boat.rs:364-397 | `ChestBoat.interact` returns `sidedSuccess(...)` / `CONSUME` and does not depend on whether the menu open succeeded; Rust returns `true` even when `open_custom_inventory_screen` fails (no inventory / plugin cancel), swallowing the interaction. Return the result of `open_custom_inventory_screen` (or `false` on failure) so a cancelled `ContainerOpen`/inventory event does not silently consume the click | S |
| 4 | item/items/boat.rs:118-175 | `BoatItem.use` entity sweep: vanilla iterates `level.getEntities(player, player.getBoundingBox().expandTowards(viewVector.scale(5.0)).inflate(1.0))` and rejects when the entity box inflated by `getPickRadius()` contains the player's eye position, for every non-spectator pickable entity. Rust first filters by `is_pushable() || can_hit()` and then does a *second*, separate pass after the fluid clip (`get_entities_at_box(boat_box)` with ITEM/EXPERIENCE_ORB exclusions plus a players pass). The second pass has no vanilla counterpart *verify*; the first pass uses `can_hit()` where vanilla's predicate is broader. Align the sweep with `EntitySelector.NO_SPECTATORS` + pickable, then re-check whether the post-clip passes exist in vanilla before keeping them | M |
| 5 | block/blocks/redstone/dispenser.rs:529-562 | `BoatDispenseItemBehavior` only dispenses into (or on water-adjacent) water and computes the spawn point/rotation from the dispenser centre and facing; the Rust port returns `false` (item drops as an item) when the target block is not water/air-over-water. Rim positions and the "no fluid source in the dispenser itself" case are not modelled, and the entity yaw uses `atan2` on the facing normal while vanilla derives yaw from the facing directly. *verify* the decompiled behaviour, then match spawn point and failure cases | M |
| 6 | entity/vehicle/boat.rs:43-56 | `is_chest_boat` enumerates the 11 chest-boat entity types by hand. Vanilla derives this from `EntityType`/`Boat.Type` (chest boat subclasses); a new wood type added by a future codegen bump will silently get 2 seats and no inventory. Prefer the generated tag/registry (`tag::EntityType::MINECRAFT_CHEST_BOATS` if present, else a `pumpkin-data` table) and keep the list only as a fallback | S |

## Checked and already matching vanilla

- `AbstractBoat` lifecycle: `blocks_building = true` on construction, `get_max_passengers` (2 boat / 1 chest boat),
  seat offsets from extracted `entity_type.boat` data plus the two-passenger `±0.2/-0.6` split and the `+0.2` animal
  shift, `get_controlling_passenger` = first living passenger.
- Damage: `hurt` = `damage * 10 > 40` (or creative) → drop behind the `entity_drops` game rule, chest contents
  claimed once via `claim_drops`, custom name copied onto the dropped boat, `can_hit()` = alive.
- Client control: ridden boats zero server velocity and let the client move (`move_from_client`, `correct_client_position`,
  VehicleMoveEvent), unridden boats run `float_boat`/`move_boat` and reset paddles when no paddle input arrives.
- Fluid status: `InWater`/`UnderWater`/`UnderFlowingWater`/`OnLand`/`InAir` with the `0.001` water offset, `is_eye_in_water`,
  `modify_passenger_fluid_interaction_box`, 60-tick underwater ejection, boat water physics from #4073, lava burning from #4386.
- Chest boat: 27 slots, `container::open` with the cross-language `entity.minecraft.<type>` title, `ContainerOpen`
  game event, NBT read/write via `MinecartInventory`, loot-table unpack on access.
- Item/entity wiring: all 11 wood types + bamboo raft in `item_to_entity` / `ids()`, `entity_to_item` used for
  creative middle-click pickup (`pick_item.rs:96-100`), placement emits `ENTITY_PLACE`, uses `FluidHandling.ANY` +
  `Block.OUTLINE` clip, `is_space_empty` rejection, `USED` statistic, creative-aware stack decrement.
