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

## h6 status
- Rider yaw clamp (AbstractBoat.clampRotation, +/-105 deg, body and head sync): FIXED bd8b9bf97 in entity/vehicle/boat/passengers.rs. Unit test: rider_yaw_is_clamped_to_105_degrees_around_boat_yaw. Delta-rotation accumulation not added: UNVERIFIED, STILL OPEN.

## Result

Re-review against `.vendor/vanilla-decompiled/net/minecraft` (AbstractBoat / BoatItem / BoatDispenseItemBehavior / EntitySelector / Player).

- Row 1 (interact / eye in water): verified no change. Vanilla `AbstractBoat.canAddPassenger` (AbstractBoat.java:836-838) is `size < max && !isEyeInFluid(WATER)`; `interact` reaches it via `player.startRiding`. Rust already applies the same gate at boat.rs:621-631. No fix.
- Row 2 (destroy drop position): verified already fixed. `drop_stack_at(self.vehicle.entity.pos.load())` at boat.rs:565 and `scatter_inventory_at(position)` at 213/220 use the entity position, matching vanilla `spawnAtLocation`/`dropContents`. No fix.
- Row 3 (BUBBLE_TIME): STILL OPEN. Decompiled `AbstractBoat` drives `DATA_ID_BUBBLE_TIME` only from bubble columns (`onAboveBubbleColumn` sets 60 at line 170, `tickBubbleColumn` decrements at 368-370), never from `UnderWater`/`UnderFlowingWater`. Rust additionally writes underwater ticks into that id (boat.rs:286-291), which diverges and needs a client-effect regression check outside this unit. Left open with reason.
- Row 4 (clampRotation): verified already fixed. `clamp_rider_yaw` / `rider_yaw_with_delta` (boat.rs:707-723) applied in `on_rider_positioned` (666-677); players excluded (668-670). No fix.
- Row 5 (BoatItem.use sweep): FIXED. Vanilla (BoatItem.java:40-70) filters the eye-block sweep with `EntitySelector.CAN_BE_PICKED` (= `Entity::isPickable`, EntitySelector.java:19; Player.isPickable excludes spectators) and has no post-clip entity/player pass, only `noCollision`. Deleted the extra `get_entities_at_box`/`get_players_at_box` block at `item/items/boat.rs`. No pure unit-test seam for the removed world query.
- Row 6 (dispenser water): FIXED. Decompiled `BoatDispenseItemBehavior` (lines 34/37) tests `getFluidState(pos).is(FluidTags.WATER)`, which matches flowing water as well as a source; the tracker's `isStill` claim is stale for this version. Relaxed the Rust `is_still_water` closure to `fluid.has_tag(&tag::Fluid::MINECRAFT_WATER)` at `block/blocks/redstone/dispenser.rs`. No pure unit-test seam.
- Row 7 (is_chest_boat): verified already fixed. `is_chest_boat` now derives from resource-name suffix (boat.rs:62-65) and `max_passengers` (74-80) is the shared helper; no hardcoded 11-type list remains. No fix.
- Row 8 (boat gametest): STILL OPEN. Needs a new file under `crates/pumpkin-gametest`, outside the source files named in this unit. Left open with reason.
- h6 (delta rotation): verified already fixed per code. `yaw_delta` field (boat.rs:169), computed at 433-439 and applied via `rider_yaw_with_delta` at 673, replaces the missing accumulation noted in the previous pass. No fix.

Rustfmt `--edition 2024` run on both edited .rs files (exit 0). Did not compile (CODER rule).
## Result

Re-review pass against the decompiled tree at `.vendor/vanilla-decompiled` (AbstractBoat.java reached via the symlink). Only rows still marked STILL OPEN in the prior Result were revisited.

- Row 3 (BUBBLE_TIME): verified already fixed in the current `boat.rs`. `tick_underwater` (lines 392-419) now only increments `ticks_underwater` for the 60-tick eject and deliberately does not write `ID_BUBBLE_TIME` (comment lines 393-395). A grep for `ID_BUBBLE_TIME`/`bubble_time` in pumpkin-core finds exactly three writes, all in bubble-column paths: `on_above_bubble_column` (639-642, sets 60 once) and `tick_bubble_column` (436, 446, decrement to 0 and clear). This matches vanilla `AbstractBoat.onAboveBubbleColumn` (line 165-172, `setBubbleTime(60)` only when 0) and `tickBubbleColumn` (351-387, decrement while above a column, then sink/launch effect at 0); `handleBubbleColumnEffect` (399-408) is mirrored by `bubble_column_vertical_velocity`. No code change. The prior STILL OPEN note and its boat.rs:286-291 line reference are stale.
- Row 8 (boat gametest): left open. Adding it needs a new file under `crates/pumpkin-gametest`, which is not a source file named in this unit. Reason: new gametest file outside this unit.

No .rs files touched this pass, so rustfmt was not run.

## Result

Third re-review pass against the decompiled tree at `.vendor/vanilla-decompiled` (AbstractBoat, BoatItem, EntitySelector, Entity, LivingEntity, Player). Only the prior STILL OPEN items were revisited.

- Row 3 (BUBBLE_TIME / underwater ticks): verified already fixed. `ID_BUBBLE_TIME` writes in boat.rs are confined to bubble columns: `on_above_bubble_column` (line 640, sets 60) and `tick_bubble_column` (lines 436, 446, decrement/clear). `tick_underwater` (lines 392-402) only accumulates `ticks_underwater` for the 60-tick eject and deliberately does not touch the id, matching `AbstractBoat.onAboveBubbleColumn` / `tickBubbleColumn`. No change.
- h6 (delta rotation): verified already fixed. `yaw_delta` field (line 232), stored at 577, applied via `rider_yaw_with_delta`/`clamp_rider_yaw` at 867-870 in `on_rider_positioned`. No change.
- Row 8 (boat gametest): left open. Requires a new file under `crates/pumpkin-gametest`, which is outside this unit (basenames: none). Reason: new gametest file outside this unit.

Spot-check of the other rows confirmed they remain fixed in the current source (e.g. `is_chest_boat`/`max_passengers` helpers at boat.rs:76/95, `drop_stack_at(self.vehicle.entity.pos.load())` at boat.rs:743, dispenser water predicate `has_tag(MINECRAFT_WATER)` at dispenser.rs:532, BoatItem eye-block sweep with the post-clip pass removed at item/items/boat.rs:138-168). No source files edited this pass (unit grants no source basenames); rustfmt not run.

## Result

Third-pass final review against `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java` (plus BoatItem / EntitySelector / Player / BoatDispenseItemBehavior). Every row is now terminal; no STILL OPEN rows remain.

- Row 1 (interact / eye in water): VERIFIED. Vanilla `AbstractBoat.interact` (AbstractBoat.java:779-788) rejects on `isSecondaryUseActive`, `outOfControlTicks >= 60`, or `!player.startRiding(this)`; `canAddPassenger` (836-837) = `size < max && !isEyeInFluid(WATER)`. Rust `interact` (boat.rs:767-836) computes the same via `boat_interact_outcome` using sneaking, `ticks_underwater >= TIME_TO_EJECT` and `is_eye_in_water` (826). No change.
- Row 2 (destroy drop position): VERIFIED. `drop_stack_at(self.vehicle.entity.pos.load(), stack)` (boat.rs:759) matches vanilla `spawnAtLocation`/`Containers.dropContents`. No change.
- Row 3 (BUBBLE_TIME): VERIFIED. `ID_BUBBLE_TIME` writes are confined to `tick_bubble_column` (436, 446) and `on_above_bubble_column` (656); `tick_underwater` (392-394) deliberately does not touch it, matching `AbstractBoat.onAboveBubbleColumn`/`tickBubbleColumn` (bubble columns only). No change.
- Row 4 / h6 (clampRotation + delta rotation): VERIFIED. `clamp_rider_yaw` (1009), `rider_yaw_with_delta` (1019) applied at 886/932; players excluded. No change.
- Row 5 (BoatItem.use sweep): FIXED in an earlier pass; post-clip world query removed at `item/items/boat.rs`. No change this pass.
- Row 6 (dispenser water): VERIFIED. `fluid.has_tag(&tag::Fluid::MINECRAFT_WATER)` at `dispenser.rs:532` matches vanilla `getFluidState(pos).is(FluidTags.WATER)` (flowing included). No change.
- Row 7 (is_chest_boat): VERIFIED. `is_chest_boat` (76) / `max_passengers` (95) shared helpers; no hardcoded 11-type list. No change.
- Row 8 (boat gametest): FIXED / VERIFIED. The three scenarios requested are already implemented in `crates/pumpkin-core/src/server/builtin_game_tests/boat_block_effects.rs` and registered in that module's `mod.rs`: boat floats up after a fall (`test_boat_floats_up_after_fall`), ejects a rider after 60 underwater ticks (`test_boat_ejects_passenger_after_60_underwater_ticks`), and chest boat scatters its 27 slots and drops the matching item on break (`test_chest_boat_scatters_27_slots_on_break`, `test_chest_boat_drops_item_on_break`). All nine boat tests appear in `BUILTIN_TEST_IDS` (mod.rs:272-280) and are wired with `register_function_test` (mod.rs:1812-1874). The prior "needs a new file under crates/pumpkin-gametest, outside this unit" reason was based on looking in the wrong crate: built-in gametests live under `pumpkin-core/src/server/builtin_game_tests/`, which already contains this file. No new code needed.

No .rs files edited this pass, so rustfmt was not run. Spot-checked the shipped code and helper API; all rows verified against the decompiled vanilla path cited above.


## Result

Fourth-pass verification. Read the unit's last Result and the shipped source; every row is terminal and no STILL OPEN row remains.

- Row 1 (interact / eye in water): VERIFIED against `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java` (`interact` plus `canAddPassenger = size < max && !isEyeInFluid(WATER)`). No change.
- Row 2 (destroy drop position): VERIFIED against `AbstractBoat.java` / `AbstractChestBoat.java` (`spawnAtLocation` / `Containers.dropContents` at the entity position). Rust uses `self.vehicle.entity.pos.load()`. No change.
- Row 3 (BUBBLE_TIME): VERIFIED against `AbstractBoat.java` (`onAboveBubbleColumn` sets 60, `tickBubbleColumn` decrements, bubble columns only). Rust confines `ID_BUBBLE_TIME` writes to the bubble-column paths and keeps `tick_underwater` for the 60-tick eject only. No change.
- Row 4 / h6 (clampRotation + delta rotation): VERIFIED against `AbstractBoat.java` (`clampRotation` +/-105 deg, body and head sync). Rust `clamp_rider_yaw` / `rider_yaw_with_delta` applied in `on_rider_positioned` with players excluded. No change.
- Row 5 (BoatItem.use sweep): VERIFIED against `.vendor/vanilla-decompiled/net/minecraft/world/item/BoatItem.java` and `EntitySelector.java` (`CAN_BE_PICKED`, eye-block sweep only, no post-clip pass). Rust post-clip pass removed in an earlier pass. No change.
- Row 6 (dispenser water): VERIFIED against `.vendor/vanilla-decompiled/net/minecraft/core/dispenser/BoatDispenseItemBehavior.java` (`getFluidState(pos).is(FluidTags.WATER)`, flowing included). Rust uses `fluid.has_tag(MINECRAFT_WATER)`. No change.
- Row 7 (is_chest_boat): VERIFIED. `is_chest_boat` derives from the resource-name suffix and `max_passengers` is the shared helper; no hardcoded 11-type list. No change.
- Row 8 (boat gametest): VERIFIED. The three requested scenarios exist in the granted file `crates/pumpkin-core/src/server/builtin_game_tests/boat_block_effects.rs`: `test_boat_floats_up_after_fall`, `test_boat_ejects_passenger_after_60_underwater_ticks`, `test_chest_boat_scatters_27_slots_on_break`, plus `test_chest_boat_drops_item_on_break`. Confirmed by grep that all four are registered in that module's `mod.rs` (lines 1838/1845/1852/1859) and listed in `BUILTIN_TEST_IDS` (lines 272-280). No new code needed.

No source files edited: all eight rows are FIXED or VERIFIED, zero open, and the granted file already carries the required tests. No rustfmt run (no .rs changes). No cargo or git command run by this worker.
