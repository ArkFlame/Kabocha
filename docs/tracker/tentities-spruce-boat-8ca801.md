# Tracker: entities / Spruce Boat (RE-REVIEW-UNTRACKED)

Re-review date: 2026-11-17. Branch `kab/tentities-spruce-boat-8ca801`. The earlier review
(`4df90a24e`) described a boat that still wrote no bubble time and had no damage game event;
both landed since, so this pass re-reads the current source. No decompiled vanilla sources exist in
`.vendor/` in this worktree, so each row names the vanilla member it must match.

`SPRUCE_BOAT` / `SPRUCE_CHEST_BOAT` still share the generic `BoatEntity`: the only per-variant
data is `EntityType.boat` (`BoatSeats { ride_height: 0.1875, single_passenger_x_offset: 0.0 }`),
item -> entity `item/items/boat.rs:25-26`, id list `item/items/boat.rs:75-76`, constructor
dispatch `entity/type.rs:348-349`, and `is_chest_boat` (`boat.rs:46-58`) which excludes
`SPRUCE_BOAT` (2 seats) and includes `SPRUCE_CHEST_BOAT` (1 seat, 27 slots).

## Source map (current)

- `entity/vehicle/boat.rs`: state :82, tick :317, tick_underwater (bubble time + 60-tick eject)
  :179, damage_with_context :361, interact :421, seat attachment :232, chest NBT :276.
- `entity/vehicle/boat/physics.rs`: get_status :44, is_underwater :62, check_in_water :89,
  get_water_level_above :114, get_ground_friction :142, float_boat :185, move_boat :235,
  move_from_client :257 (dead).
- `entity/vehicle/boat/{passengers,dismount,block_effects}.rs`: rider seating, vanilla dismount
  search, `checkInsideBlocks` sweep.
- `entity/vehicle/vehicle.rs`: hurt/damage decay, wobble metadata, damage/destroy + game event.
- `net/java/play/{paddle_boat,move_vehicle}.rs`: client input packets.

## Fixed this pass

- `entity/mod.rs:1422` `EntityBase::kill`: `/kill` on a non-living entity went straight to
  `Entity::remove`, which drops nothing, so a `SPRUCE_CHEST_BOAT` (and every chest minecart) lost
  its 27 slots instead of scattering them. Vanilla `AbstractChestBoat.remove` /
  `AbstractMinecartContainer.remove` call `Containers.dropContents` whenever
  `reason.shouldDestroy()` (KILLED/DISCARDED), independent of `doEntityDrops`; the kill path now
  scatters `get_container_inventory()` at the entity position before removal. The damage path keeps
  its `claim_drops` guard, so a destroyed boat still scatters exactly once.

## Verified, not gaps

- `boat.rs:179-194` `tick_underwater` now syncs `ID_BUBBLE_TIME` from the underwater tick count
  and ejects at 60, matching `AbstractBoat.tick` (`outOfControlTicks`, `DATA_ID_BUBBLE_TIME`).
- Bubble columns are handled generically: `Entity::on_inside_bubble_column` /
  `on_above_bubble_column` (`entity/mod.rs:661-678`) apply `bubble_column` constants 0.06/0.7 up,
  0.1/1.8 at the surface, -0.03/-0.3 down (`block/blocks/bubble_column.rs:19-24`), and
  `float_boat` then applies its per-tick friction exactly like `AbstractBoat.floatBoat`. Nothing
  boat-specific is missing.
- `physics.rs:142-183` `get_ground_friction`: generated block data has `slipperiness: 0.6` for
  normal blocks (`pumpkin-data/src/generated/block.rs:548722`), i.e. the vanilla
  `Block.getFriction()` value, so `land_friction` is right; the `-1 / ceil` scan bounds only add
  a ring that cannot intersect the contact box.
- `physics.rs:185-233` `float_boat`: gravity -0.04, 0.9/0.45/7.0e-4 branches, buoyancy
  `(v + e * gravity/0.65) * 0.75`, air -> water snap with `last_yd` reset - matches
  `AbstractBoat.floatBoat`.
- `boat.rs:361-419` damage: invulnerability first, creative = no drops, explosion skips the item
  drop, custom name copied - matches `AbstractBoat.hurt`/`destroy`.
- `boat.rs:421-460` `interact`: 60-tick gate, eye-in-water gate, secondary use opens the chest
  only, plain use boards a free seat - matches `AbstractBoat.interact` + `AbstractChestBoat`.
- `passengers.rs:34-65` and `boat.rs:232-264` agree on the seat offsets (0.2/-0.6, animal +0.2)
  and rotation sign; `passenger_attachment_point` is what the generic `position_rider` uses.

## Remaining gaps

| # | file:line | vanilla | gap / suggested fix | size |
|---|---|---|---|---|
| 1 | `entity/mod.rs:1570` `position_rider`, `boat.rs:232` `passenger_attachment_point` | `AbstractBoat.clampRotation` | No rider-rotation clamp for non-player riders: vanilla sets body yaw to the boat yaw, clamps the rider yaw delta to +/-105 deg and syncs head yaw after every `positionRider`. Add an `EntityBase::clamp_rotation` hook called at the end of `position_rider` (default no-op) and the boat override. Players are client-authoritative, so only mob riders change. Needs the decompiled method to pin the `yRotO` adjustment. | M |
| 2 | `physics.rs:257-297` `move_from_client` + `correct_client_position` | n/a | Dead code: the boat MoveVehicle path is unreachable (the generic `net/java/play/move_vehicle.rs` + `has_new_boat_collision` supersede it). Delete both methods and the now-unused `Player`/`CMoveVehicle` imports; `position_rider` stays (reached from `passengers.rs:12`/`entity/mod.rs:6079`). | S |
| 3 | `server/builtin_game_tests/` | runtime parity | No boat gametest (only the dispenser boat test). Add one covering the air -> water snap, ejection after 60 submerged ticks, and chest scatter on break + on `/kill` (the last one guards the fix above). | S |
