# Tracker: entities / Spruce Boat (RE-REVIEW-UNTRACKED)

Re-review date: 2026-11-17. Branch `kab/tentities-spruce-boat-8ca801`. Prior review:
`docs/tracker/tentities-spruce-boat.md` (written before `570d3ea1f` split the boat into
`entity/vehicle/boat/{physics,block_effects,passengers,dismount}.rs`, so its line numbers are stale).
No decompiled vanilla sources exist in `.vendor/` in this worktree, so each row names the vanilla
member it must match and marks items that need the decompiled method before changing code.

`SPRUCE_BOAT` / `SPRUCE_CHEST_BOAT` share the generic `BoatEntity`: the only per-variant data is
`EntityType.boat` (`BoatSeats { ride_height: 0.1875, single_passenger_x_offset: 0.0 }`), item -> entity
`item/items/boat.rs:25-26`, id list `item/items/boat.rs:75-76`, constructor dispatch `entity/type.rs:348-349`,
and `is_chest_boat` (`boat.rs:45-57`) which correctly excludes `SPRUCE_BOAT` (2 seats, no inventory) and
includes `SPRUCE_CHEST_BOAT` (1 seat, 27 slots).

## Source map (current)

- `entity/vehicle/boat.rs`: state, `tick` :297, `tick_underwater` :159, `damage_with_context` :341,
  `interact` :400, seat attachment :212, chest NBT :256.
- `entity/vehicle/boat/physics.rs`: `get_status` :44, `is_underwater` :62, `check_in_water` :89,
  `get_water_level_above` :114, `get_ground_friction` :142, `float_boat` :185, `move_boat` :235,
  `move_from_client` :257 (dead).
- `entity/vehicle/boat/block_effects.rs`: `Entity.checkInsideBlocks` sweep (:24, :49, 16-step / 100-movement caps).
- `entity/vehicle/boat/{passengers,dismount}.rs`: rider seating and the vanilla dismount search.
- `entity/vehicle/vehicle.rs`: hurt time/damage decay (:34), wobble metadata (:167), damage/destroy (:216).
- `net/java/play/{paddle_boat,move_vehicle}.rs`: client input packets.

## Fixed this pass

- `entity/vehicle/vehicle.rs:239-247` `damage_with_context`: no `GameEvent.ENTITY_DAMAGE` was fired for any
  vehicle hit, so sculk sensors/wardens never heard a boat (or minecart) being hit or broken. Vanilla
  `AbstractBoat.hurt`/`AbstractMinecart.hurt` call `gameEvent(GameEvent.ENTITY_DAMAGE, source.getEntity())`
  right after the wobble state; the emission now runs at the same point with the attacker as source entity,
  after the plugin cancel checks (cancelled damage still emits nothing).

## Verified, not gaps

- `physics.rs:197-206` air -> water snap: the offset is now returned unconditionally and `last_yd` is reset,
  matching `AbstractBoat.floatBoat`; the earlier review row for this is closed.
- `physics.rs:208` gravity: `get_gravity()` returns `no_gravity ? 0 : 0.04`, which mirrors
  `AbstractBoat.getDefaultGravity() = 0.04` behind `Entity.getGravity()`. No gap.
- `boat.rs:193` `is_client_controlled` uses `get_controlling_passenger` (first `LivingEntity`), matching
  `AbstractBoat.getControllingPassenger`.
- `boat.rs:400-438` `interact`: no eye-in-water gate, `ticks_underwater < 60`, secondary use only, chest boat
  opens the container only on sneak - ordering matches `AbstractBoat.interact` + chest-boat open.
- `boat.rs:341-398` damage: invulnerability first, creative = instant break with no drop, survival drop skips
  explosions and copies the custom name - `AbstractBoat.hurt`/`destroy`.
- `paddle_boat.rs:6` uses the direct vehicle, vanilla uses `getRootVehicle() instanceof AbstractBoat`; a player
  can only ever ride the boat directly (boats carry no vehicle of their own), so the behaviour is identical.
- `move_vehicle.rs:73` stores the packet yaw/pitch raw; vanilla `absMoveTo` only applies `% 360`, a cosmetic
  normalisation with no behavioural effect for a client-authoritative boat.

## Remaining gaps

| # | file:line | vanilla | gap / suggested fix | size |
|---|---|---|---|---|
| 1 | `boat.rs:297` `tick`, `physics.rs:185` `float_boat`, `tracked_data.rs` `ID_BUBBLE_TIME` | `AbstractBoat.tickBubbleColumn`, `onInsideBubbleColumn`/`onAboveBubbleColumn`, `DATA_ID_BUBBLE_TIME` | Bubble columns are unhandled: `ID_BUBBLE_TIME` is generated but never written, and the only effect is the generic `bubble_column::push_entity` velocity, which `float_boat` then multiplies by 0.9 and re-adds gravity to. Boats do not rise/fall at vanilla speed and never show the bubble wobble. Track the bubble-column direction plus a bubble timer, apply vanilla 0.06 (0.1 at surface) acceleration with 0.7/1.8 caps and -0.03/-0.3 downward before `float_boat`, and sync `ID_BUBBLE_TIME`. Needs the decompiled `AbstractBoat` to pin the constant order. | M |
| 2 | `passengers.rs:34` `rider_position`, `boat.rs:212` `passenger_attachment_point` | `AbstractBoat.clampRotation` | No rider-rotation clamp: vanilla clamps a mob rider's body yaw to +/-105 deg around the boat yaw and syncs head yaw. Players stay client-authoritative, so only non-player riders need the hook. Add a vehicle rider-rotation hook and the boat override. | M |
| 3 | `physics.rs:257` `move_from_client` | n/a | Dead code: the boat MoveVehicle path is unreachable (only definition; the generic `net/java/play/move_vehicle.rs` + `client_move.rs:299` `has_new_boat_collision` supersede it). Delete it, and keep `position_rider` (still reached from `entity/mod.rs:1570`). | S |
| 4 | `server/builtin_game_tests/` | runtime parity | No boat gametest (grep finds only the dispenser boat test), so the rewritten water physics, the 60-tick underwater ejection and the chest-boat 27-slot scatter have no runtime net. Add one gametest covering float-up after a fall, ejection after 60 submerged ticks, and scatter on break. | S |
| 5 | `physics.rs:142-183` `get_ground_friction` | `AbstractBoat.getGroundFriction` | Uses `Block.slipperiness` as the block friction term; confirm that generated value is vanilla `Block.getFriction()` (default 0.6, ice 0.98) and not the slipperiness attribute, otherwise a boat on normal ground gets the wrong `land_friction`. Needs a data check, no code change expected. | S (verify) |

