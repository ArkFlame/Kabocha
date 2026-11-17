# Tracker: entities / Dark Oak Boat (RE-REVIEW)

Source (unchanged since the last review, re-read in full for this pass):
`crates/pumpkin-core/src/entity/vehicle/boat.rs` + `boat/{physics,block_effects,dismount,passengers}.rs`,
item side `crates/pumpkin-core/src/item/items/boat.rs`, mob-into-boat pickup `entity/mod.rs:1238-1267`,
registration `entity/type.rs:332-353`, dispenser `block/blocks/redstone/dispenser.rs:529-569`,
creative pickup `net/java/play/pick_item.rs:87-100`.
Vanilla reference classes: `net.minecraft.world.entity.vehicle.AbstractBoat` / `Boat` /
`AbstractChestBoat`, `net.minecraft.world.item.BoatItem`, `net.minecraft.core.dispenser.BoatDispenseItemBehavior`.
No `.vendor/` decompiled tree in this worktree (`glob .vendor/**/*Boat*.java` = 0 files), so rows tagged
*verify* need the decompiled class before editing.
The boat code is per-wood-type generic (`item_to_entity` / `is_chest_boat` / `entity_type.boat` seat data),
so Dark Oak has no special-case code: every row applies to `EntityType::DARK_OAK_BOAT` and
`DARK_OAK_CHEST_BOAT` equally, and data for both types is correct (ids 34/35, `boat: ride_height 0.1875`,
`single_passenger_x_offset 0.0`, dimension `1.375 x 0.5625`, eye height `0.5625`).

Changes since the previous review: the old row "`ChestBoat.interact` returns `true` even when
`open_custom_inventory_screen` fails" is **resolved** - `boat.rs:372-374` now returns the
`container::open` result (`boat.rs:96-117`), so a failed/cancelled screen no longer consumes the click.
No other source change was found. No code commit in this pass: rows 1, 5 and 6 are blocked on the
decompiled class, the rest are drift risk or a missing hook, not a certain one-file fix.

## Remaining gaps

| # | file:line | Vanilla class / behaviour | Suggested fix | Size |
|---|---|---|---|---|
| 1 | entity/vehicle/boat.rs:359-394 (`interact`) | `AbstractBoat.interact` rejects only `player.isSecondaryUseActive()` and `outOfControlTicks >= 60`. The Rust `can_add_passenger` additionally rejects when the **boat's** eye is in water (`is_eye_in_water`, physics.rs:45-72), so a boat at the surface with its eye point inside the fluid refuses a rider even though the 60-tick ejection timer has not run out. *verify* the decompiled `interact`; if confirmed, drop `!self.is_eye_in_water()` and keep the underwater rule only in `tick_underwater` (boat.rs:136-159) | S |
| 2 | entity/vehicle/boat.rs:326-353 (destroy path) | `AbstractBoat.destroy` / `AbstractChestBoat.destroy` use `spawnAtLocation(...)` (entity position, bounding-box centre) and `Containers.dropContents(level, this, inventory)`; Rust scatters the 27 slots and drops the boat item at `block_pos` (`boat.rs:329`), i.e. the block corner, so a boat broken mid-air/over a slope can drop into a different block. Use `self.vehicle.entity.pos.load()` for both calls | S |
| 3 | entity/vehicle/boat.rs:268-293 (`tick`) | `AbstractBoat` maintains `DATA_ID_BUBBLE_TIME` (generated accessor `ID_BUBBLE_TIME` = `DATA_ID_BUBBLE_TIME`, generated/tracked_data.rs:157, next to `ID_PADDLE_LEFT`), driving the submerged-boat bubble effect for the rider. Grep `ID_BUBBLE_TIME`/`bubble_time` in `pumpkin-core` = 0 hits, so the value is always `0` and the client never renders bubbles. Add an `AtomicI32` updated from `tick` on `UnderWater`/`UnderFlowingWater` and written with `send_meta_data`; *verify* the exact increment/decay rule against `AbstractBoat.tick` first | S |
| 4 | entity/vehicle/boat.rs:183-215, passengers.rs:44-65 | `AbstractBoat.clampRotation` clamps a non-player rider's body yaw to +/-105 deg around the boat yaw (and syncs head yaw). No `clamp_rotation` hook exists in `pumpkin-core` (grep `clamp_rotation`/`105` in entity = 0 hits), so mob riders behind the seat are never clamped server-side. Add a vehicle rider-rotation hook; keep players out of it (client authoritative) | M |
| 5 | item/items/boat.rs:131-182 | `BoatItem.use` entity sweep is `level.getEntities(player, box, EntitySelector.NO_SPECTATORS.and(Entity::isPickable))` and only the eye-inside-inflated-box test rejects. Rust uses `!is_spectator() && can_hit()` instead of `isPickable`, and then adds a second, post-clip pass (`get_entities_at_box(boat_box)` with ITEM/EXPERIENCE_ORB exclusions plus `get_players_at_box`) that has no vanilla counterpart. *verify*; if confirmed, delete the second pass and use the pickable predicate | M |
| 6 | block/blocks/redstone/dispenser.rs:529-543 | `BoatDispenseItemBehavior` (older form) tests `world.getFluidState(pos).isStill()` before the +1.0 lift and for the air-over-water case; Rust's `is_water` closure accepts `FLOWING_WATER` too, so a dispenser aimed at flowing water spawns a boat where vanilla drops the item. *verify*; if confirmed, require `state.is_source` (predicate already used in physics.rs:58/111) | S |
| 7 | entity/vehicle/boat.rs:41-53 (`is_chest_boat`) | Vanilla derives chest-ness from the entity class / `Boat.Type`; Rust hand-lists the 11 chest-boat entity types, duplicated again at entity/mod.rs:1238-1250 (passenger cap). A future wood type from a codegen bump silently gets 2 seats and no inventory. The generated item tag `tag::Item::MINECRAFT_CHEST_BOATS` (tag.rs:24247) already lists exactly these 11 items, so map `entity_type.resource_name` -> `Item::from_registry_key` -> `has_tag`, and expose one `max_passengers` helper for both call sites | S |
| 8 | entity/vehicle/boat.rs (whole) | No boat coverage in `pumpkin-gametest` (glob `crates/pumpkin-gametest/**/*boat*` = 0 files); the rewritten water physics, the 60-tick ejection and the chest break-scatter are only covered by unit tests on the block sweep (block_effects.rs:389-495) | add one gametest: boat rises to the surface after a fall, ejects after 60 underwater ticks, chest boat scatters its slots and drops the matching item on break | S |

## Checked and already matching vanilla

- `AbstractBoat` lifecycle: `blocks_building = true` on construction, 2 seats for boats / 1 for chest boats,
  seat offsets from extracted `entity_type.boat` data plus the two-passenger `+0.2/-0.6` split and the
  `+0.2` animal shift, `get_controlling_passenger` = first passenger only if it is a `LivingEntity`.
- Damage: `hurt` = `damage * 10 > 40` or instabuild -> break, behind the `entity_drops` game rule, chest
  contents claimed once via `claim_drops`, custom name copied onto the dropped item, `can_hit()` = alive.
  Chest contents are dropped for any destroying removal, only the item drop is creative-gated.
- Fluid status: `InWater`/`UnderWater`/`UnderFlowingWater`/`OnLand`/`InAir` with the `0.001` water offset,
  `get_water_level_above`, ground friction (lily pads skipped, edge cells ignored), buoyancy
  `(water_level - y)/height`, `(v + buoyancy * (0.04/0.65)) * 0.75`, `-0.04` sink and the fall-to-surface
  branch with its `is_space_empty` + collision guard, `modify_passenger_fluid_interaction_box`.
- Client control: ridden boats zero server velocity and move from `move_from_client` with client-position
  correction + `VehicleMoveEvent`; unridden boats run `float_boat`/`move_boat`, paddles reset when no input.
- Dismount: `get_dismount_location_for_passenger` escape vector, water fallback, standing/crouching/swimming
  pose probes, world-border check, `DISMOUNT_EPSILON`.
- Chest boat: 27 slots, `container::open` with the cross-language `entity.minecraft.<type>` title,
  `ContainerOpen` event, NBT via `MinecartInventory`, loot-table unpack on access, spectator no-open.
- Placement: `FluidHandling.ANY` + `Block.OUTLINE` clip, 5.0 reach view vector inflated by 1.0, player eye
  excluded, pick radius `0.3`, `is_space_empty`, `ENTITY_PLACE` event, `USED` statistic, creative-aware
  stack decrement, yaw = player yaw; entity -> item used for creative middle-click pickup.
- Dispenser: spawn point `0.5625 + width/2` horizontally and `1.125` vertically from the block centre, +1.0
  lift when the target holds water, air-over-water case, drop-as-item otherwise, yaw from the facing
  (`atan2(x, z)` is equivalent to `Direction.toYRot()` modulo 360), `SoundDispenserDispense`.

## Considered and rejected (do not re-open)

- `is_client_controlled` (boat.rs:161-169) uses `passengers.first()` while vanilla uses
  `getControllingPassenger() instanceof Player`. Since `get_controlling_passenger` is "first passenger if
  living", both evaluate to "the first passenger is a player": a non-living first passenger with a player
  behind yields `None`/false in both. No divergence.
- `vertical_speed = -self.get_gravity()` in `float_boat` (physics.rs:248) matches vanilla for normal boats
  (`0.04`) and improves parity for `NoGravity` boats (0 instead of the hard-coded sink), so replacing it
  with the constant would be a regression.
