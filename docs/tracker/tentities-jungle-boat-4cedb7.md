# entities / Jungle Boat - audit (RE-REVIEW-UNTRACKED)

Tracker row: `entities | Jungle Boat | RE-REVIEW-UNTRACKED` (file id 4cedb7).
Re-audited after the boat water-physics rewrite (570d3ea1f) and the later chest-boat
slices. Reviewed `crates/pumpkin-core/src/entity/vehicle/boat.rs` and
`boat/{physics,passengers,dismount,block_effects}.rs`,
`crates/pumpkin-core/src/entity/vehicle/vehicle.rs`,
`crates/pumpkin-core/src/item/items/boat.rs`, `entity/type.rs` (JUNGLE_BOAT routing),
`entity/mod.rs:1235-1270` (push-mount cap), `entity/player.rs:4360-4390` (boat_one_cm).
No decompiled vanilla tree in this worktree (`.vendor/` absent), so vanilla behaviour is
from knowledge of `AbstractBoat` / `Boat` / `BoatItem` (1.21.x semantics; target 26.3 is
not available to diff).

## Verified present (jungle-specific wiring)
- `Item::JUNGLE_BOAT` -> `EntityType::JUNGLE_BOAT` (`item/items/boat.rs:29`, id list `:79`);
  construction routed in `entity/type.rs:338-355`; generated dimensions `[1.375, 0.5625]`,
  eye height `0.5625`, `BoatSeats` ride height / single-passenger offset from data.
- Placement (`item/items/boat.rs:105-235`): fluid+outline clip, entity-in-path block,
  inflated-0.3 eye check, `is_space_empty`, collidable-entity and player checks, yaw copy,
  `ENTITY_PLACE` game event, survival decrement, `boat_one_cm` statistic - matches
  `BoatItem.use` plus the `EntityPredicates.CAN_HIT` skip list.
- Damage path (`boat.rs:325-380`): creative instabuild, damage > 40 break, wobble via
  `VehicleEntity`, chest contents scatter gated by `entity_drops`, boat item drop with
  custom name, no drop on creative break.
- Fire path: `boat_one_cm`/`player.rs` untouched; interact gating on
  `ticks_underwater < 60`, free seat, sneak, spectator (chest open) present.
- Physics: status sampling, buoyancy, land/water/ice friction, `float_boat` surface jump,
  client-authoritative `move_from_client` with collision rewind (documented Pumpkin
  difference, not a gap).

## Fixed in this pass
- `boat.rs:325-345`: `hurt` had no invulnerability gate. Vanilla `Boat.hurt` returns false
  for `isInvulnerableTo(source)` before touching wobble/damage/chest contents; added
  `Entity::is_invulnerable_to(&damage_type, cause)` at the top of
  `BoatEntity::damage_with_context`, so datapack `Invulnerable` and fire immunity now apply.

## Remaining gaps (shared `BoatEntity`, visible on a jungle boat)
| file:line | vanilla class / behaviour | suggested fix | size |
|---|---|---|---|
| `boat.rs:201-233`, `passengers.rs:34-65` | `AbstractBoat.positionRider` adds `this.deltaRotation` to the rider's yaw and clamps it to +/-105 deg (`Entity.clampRotation`). No `delta_rotation`/`clamp_rotation` exists anywhere under `entity/vehicle` (grep), so a rider can look fully backwards and does not follow the boat when it turns. Live with two players. | Track per-tick yaw delta on `BoatEntity`, then apply and clamp rider yaw/head yaw in `passengers.rs::rider_position`. | M |
| `boat.rs:140-151` | `AbstractBoat` publishes `DATA_ID_BUBBLE_TIME` (generated id exists: `pumpkin-data/src/generated/tracked_data.rs:107`) while underwater to drive bubble particles; Pumpkin never sends `tracked_data::boat::ID_BUBBLE_TIME` (grep: no use in `crates/pumpkin-core`). | Add a `bubble_time` cell: 60 on entering `UnderWater`, decrement each tick, reset to 0 otherwise, publish with the paddle metadata. | M |
| `boat.rs:157-180` | `Boat.tick` gates `outOfControlTicks` on `Status.UNDER_WATER`, not the whole `is_underwater()` set; Pumpkin also counts `UnderFlowingWater`, and `get_status` overwrites `water_level` with the hull top in that branch so the vanilla `waterLevel > maxY + 1e-4` guard cannot be reproduced as-is. Eject timing under waterfalls may differ. | Confirm against decompiled `Boat.tick`, then split flowing-water out of the eject counter (keep the 60-tick reset). | S (verify) |
| `boat.rs:375-411` | `AbstractBoat` implements `Leashable`; boats can be leashed. `BoatEntity::interact` never calls `interact_leash`/`leash_to` on a LEAD and no leash hook is set, so a jungle boat cannot be leashed. | Add the lead branch before the seat logic, mirroring `MobEntity::mob_interact` leash handling. | M (verify against 26.3) |
| `boat.rs:359-374` | `AbstractBoat.destroy` drops `getDropItem()` without copying the entity custom name; Pumpkin attaches `CustomNameImpl` to the dropped boat item (`boat.rs:362-366`). | If confirmed, remove the custom-name copy from the item drop. | S (verify) |
| `entity/mod.rs:1248-1253` | Push-mount caps chest boats at 1 passenger while `boat.rs:387` (fixed in ffae1b27c, "seat two passengers in chest boats") allows 2; vanilla `canAddPassenger` has a single rule. Plain jungle boat is unaffected, but the two paths disagree for a pushed mob. | Use one `max_passengers(&EntityType)` helper from both sites. | S |
| `boat.rs:201-233` vs `passengers.rs` | `AbstractBoat.positionRider` adds `0.2` for an `Animal` in the two-seat branch; `passenger_attachment_point` applies it but `rider_position` does not. Latent only while `rider_position` is player-only. | Extract one seat-offset helper used by both paths. | S |
| `boat.rs:433-451` | No test exercises seat-offset selection (two-seat `0.2`/`-0.6`, animal shift) or the eject counter; the only boat tests are chest-boat classification and item round trip. | Pure-function test on seat offsets and the 60-tick counter transitions. | S |

## Notes
- No cargo/build was run (worker constraint); the code change is syntax-reviewed only.
- Client-predicted boat motion, seat rendering, paddle animation and bubble rendering are
  client-side effects; only the metadata side is tracked above.
- `entity/player.rs:4369-4387` re-enumerates every boat type even though the `minecraft:boat`
  tag branch above already covers them - hygiene only, left untouched.

## h6 status
- Rider yaw clamp (AbstractBoat.clampRotation, +/-105 deg, body and head sync): FIXED. Delta-rotation accumulation: FIXED and verified in a later pass (deltaRotation sampled into yaw_delta each tick and applied in on_rider_positioned / on_passenger_turned); stale residual open marker retired.

## Result

Audit against `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/{AbstractBoat,Boat}.java`
and `world/entity/vehicle/VehicleEntity.java`. All rows resolved; no generated-data, WIT or
cross-crate change needed.

- Row 40 (rider yaw delta + clamp): FIXED. `AbstractBoat.positionRider` (L696-705) applies
  `deltaRotation` before `clampRotation`; `clampRotation` (L740-754) clamps to +/-105 deg.
  `BoatEntity` now samples consecutive server yaw (`last_yaw`/`yaw_delta`) and applies it through
  `rider_yaw_with_delta` for non-player riders; `clamp_rider_yaw` enforces the window. Tests:
  `rider_yaw_clamps_to_boat_heading_window`, `rider_yaw_follows_boat_yaw_delta_then_clamps`.
- Row 41 (bubble time metadata): FIXED. `AbstractBoat` publishes `DATA_ID_BUBBLE_TIME` from bubble
  columns (L169, L351-385). `BoatEntity` now tracks `bubble_column_time`/`bubble_column_drag_down`
  and publishes it; `bubble_column_vertical_velocity` test added. `ticks_underwater` remains the
  separate eject counter.
- Row 42 (eject counter / flowing water): VERIFIED, NO CHANGE. Decompiled `AbstractBoat.tick`
  (L243) increments `outOfControlTicks` for BOTH `UNDER_WATER` and `UNDER_FLOWING_WATER`;
  Pumpkin's `status.is_underwater()` matches. The doc's suggested split of flowing water out of
  the counter would diverge from vanilla and was not applied.
- Row 43 (leash): FIXED. `AbstractBoat` implements `Leashable`; `Entity.interact` handles leads.
  `BoatEntity::interact` now has the LEAD branch (tie to player / untie with lead drop, sound,
  creative handling), `tick` calls `entity.tick_leash`, and `support_quad_leash` returns true.
- Row 44 (custom name on drop): VERIFIED. `VehicleEntity.destroy` (L94-100) copies
  `CUSTOM_NAME` onto the dropped item, so Pumpkin's `CustomNameImpl` copy is correct; the doc's
  suggestion to remove it was wrong. Separately fixed `drops_item_when_destroyed`: vanilla
  `hurtServer` (L59-80) discards on an instabuild hit with no drop regardless of name, so the
  helper now returns `!creative` (the prior `!creative || named` contradicted its own test).
- Row 45 (push-mount passenger cap): VERIFIED, already fixed. `entity/mod.rs` uses the shared
  `boat::max_passengers(entity_type)` helper for push-mount and `interact` alike.
- Row 46 (shared seat-offset helper): FIXED. `passenger_seat_offset`/`passenger_attachment` are
  used by both `passenger_attachment_point` and `passengers.rs::get_passenger_attachment_point`,
  including the +0.2 animal shift.
- Row 47 (pure-function tests): FIXED. Added tests for seat offsets (single/multi/animal), rider
  yaw delta + clamp, bubble-column velocity, create break drop, chest container gating and boat
  drop-item resolution.

Also tightened `damage_with_context` with the vanilla `isRemoved()` early return before the
invulnerability check. Edited `boat.rs` directly (rustfmt --edition 2024 run); `passengers.rs`
and `entity/mod.rs` changes came from concurrent work in the same checkout. No cargo/git run
(worker constraint); code is syntax-reviewed, not compiled.


## Result

Re-review against `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java`.
The only residual open marker (h6 note, line 57: delta-rotation accumulation) is stale. The current
`boat.rs` already samples consecutive server yaw into `last_yaw`/`yaw_delta` (lines 231-232,
575-577) and applies the delta to non-player, non-CAN_TURN_IN_BOATS riders in
`on_rider_positioned` (lines 873-889) before the +/-105 deg `clamp_rider_yaw` window;
`on_passenger_turned` (lines 891-939) re-clamps and folds the adjustment into `yRotO`.
This matches vanilla `positionRider` (L696-705), `clampRotation` (L740-754) and
`onPassengerTurned` (L762-766). Wiring confirmed at `entity/mod.rs:1673` and `:4475`;
tests `rider_yaw_clamps_to_boat_heading_window` and `rider_yaw_follows_boat_yaw_delta_then_clamps`
cover it. No source change needed and no generated-data, WIT or cross-crate change required.

## Result

Independent verification of the single residual open marker (line 57, delta-rotation accumulation)
against `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java`.
The marker is stale; no source change was needed this pass.

- `AbstractBoat.positionRider` (L696-705) adds `deltaRotation` to rider yaw and head yaw for
  passengers outside `CAN_TURN_IN_BOATS`, then calls `clampRotation` (L740-754, +/-105 deg).
  `onPassengerTurned` (L762-766) re-clamps and folds the adjustment into `yRotO`.
- Pumpkin already matches: `boat.rs` samples `yaw_delta` from consecutive yaw in `tick`
  (L575-577), applies it in `on_rider_positioned` (L840-883) with the +/-105 `clamp_rider_yaw`
  (L993-997), and re-clamps in `on_passenger_turned` (L885-933) folding the adjustment into
  `y_rot_o`. Players and `CAN_TURN_IN_BOATS` riders are skipped, mobs get the hull heading as
  body yaw with the head clamped to `get_max_head_rotation()`, matching the Mob branch of
  `clampRotation`.
- Wiring confirmed live, not dead code: `position_passenger` calls `on_rider_positioned`
  (`entity/mod.rs:1673`) and `Entity::turn` calls `on_passenger_turned` (`entity/mod.rs:4485`;
  the earlier note said 4475, the actual call is the next line). Tests
  `rider_yaw_clamps_to_boat_heading_window` and `rider_yaw_follows_boat_yaw_delta_then_clamps`
  cover the clamp and delta paths.

All rows in this unit are resolved (open=0). No generated-data, WIT, cross-crate change or
file outside the unit was touched. No cargo/git run (worker constraint); read-only verification.

## Result

Re-checked the only residual open marker (h6 note, line 57: delta-rotation accumulation) against
the decompiled tree at `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java`.
The marker is stale; no source change was needed this pass.

- `AbstractBoat.positionRider` (L696-706) adds `deltaRotation` to a non-`CAN_TURN_IN_BOATS`
  rider's yaw and head yaw, then calls `clampRotation`; `clampRotation` (L740-754) clamps a
  non-mob rider to +/-105 deg and sets a mob's yaw to the hull heading with
  `clampHeadRotationToBody`; `onPassengerTurned` (L762-766) re-clamps and folds `yDelta` into
  `yRotO`. `calculatePassengerBodyYRot` (L756-760) gives a full-boat animal a 90/270 offset.
- Pumpkin matches: `boat.rs::tick` samples `yaw_delta` from consecutive yaw (L574-577),
  `on_rider_positioned` (L840-883) applies the delta and the clamp, `on_passenger_turned`
  (L885-933) re-clamps and folds the adjustment into `y_rot_o`, and the helpers
  `rider_body_yaw` (L966-978), `clamp_rider_yaw` (L993-997), `clamp_head_to_body` (L982-986)
  and `rider_clamp_adjustment` (L1002-1004) reproduce the vanilla math. Tests
  `rider_yaw_clamps_to_boat_heading_window` and `rider_yaw_follows_boat_yaw_delta_then_clamps`
  cover the delta + clamp paths.
- Hooks are live, not dead: `entity/mod.rs:1673` calls `on_rider_positioned` and
  `entity/mod.rs:4486` calls `on_passenger_turned`.

No source file edited (unit names no source basenames). No generated-data, WIT or cross-crate
change required. Worker constraint: no cargo/git run; read-only verification only.


## Result

Second-pass verification against `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java`,
`ChestBoat`, and `world/entity/vehicle/VehicleEntity.java`. The only residual open marker (line 57,
delta-rotation accumulation) is stale; the current source already implements every gap in the
table, so no source change was needed this pass.

- Row 40 / h6 marker (rider yaw delta): FIXED and live. `AbstractBoat.positionRider` L696-705
  adds `deltaRotation` to yaw/head then `clampRotation` (L740-754, +/-105deg), and
  `onPassengerTurned` L762-766 re-clamps and folds the delta into `yRotO`. Pumpkin matches:
  `boat.rs::tick` samples `yaw_delta` from consecutive yaw (L585-588), `on_rider_positioned`
  applies the delta then clamps (L851-894), `on_passenger_turned` re-clamps (L896-944), and the
  helpers `rider_body_yaw`, `clamp_rider_yaw`, `clamp_head_to_body`, `rider_clamp_adjustment`
  (L977-1014) reproduce the vanilla math. Hooks are live: `entity/mod.rs:1686` calls
  `on_rider_positioned` and `entity/mod.rs:4499` calls `on_passenger_turned`. The stale line 57
  marker was retired to stop this row re-queuing.
- Row 41 (bubble time): FIXED. `ID_BUBBLE_TIME` exists in generated tracked_data and is published
  on bubble-column contact (boat.rs L436, L446, L651); `ticks_underwater` stays a separate eject
  counter.
- Row 42 (eject counter / flowing water): VERIFIED, no change. Vanilla `AbstractBoat.tick`
  L243-249 counts both UNDER_WATER and UNDER_FLOWING_WATER toward the 60-tick eject; Pumpkin's
  `BoatStatus::is_underwater()` (physics.rs L22-24) covers both, so the suggested split would
  diverge from vanilla.
- Row 43 (leash): FIXED. `BoatEntity::interact` LEAD branch (L765-805), `tick` calls
  `entity.tick_leash` (L604), `support_quad_leash` true (L951), and leash NBT read/write (L531-544).
- Row 44 (custom name on drop): VERIFIED. `VehicleEntity.destroy` L94-100 copies CUSTOM_NAME onto
  the dropped item; `hurtServer` L59-80 discards on an instabuild hit with no drop, so
  `drops_item_when_destroyed(creative) = !creative` (boat.rs L117) is correct.
- Row 45 (push-mount cap): VERIFIED. `entity/mod.rs:1350-1357` and boat.rs `interact` share
  `boat::max_passengers`.
- Row 46 (seat offsets): FIXED. `passenger_seat_offset` is shared by `passenger_attachment_point`
  and `passengers.rs::get_passenger_attachment_point`, including the +0.2 animal shift.
- Row 47 (pure tests): present (rider yaw clamp/delta, clamp adjustment, body yaw, head clamp,
  bubble impulse, seat offsets, interact outcome, drop resolution).

All rows open=0. No generated-data, WIT, cross-crate change or source edit required. Worker
constraint: no cargo/git run; read-only verification plus tracker-doc update only.

## Result

Second pass against `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/`
(`AbstractBoat.java`, `AbstractChestBoat.java`, `Boat.java`) and `VehicleEntity.java`.

- h6 residual open marker (delta-rotation accumulation, line 57): still stale, no change.
  `AbstractBoat.positionRider` (L696-706) applies `deltaRotation` then `clampRotation`
  (L740-754); Pumpkin samples `yaw_delta` in `tick` (boat.rs L585-588) and applies it in
  `on_rider_positioned` / `on_passenger_turned`. Verified live at `entity/mod.rs:1686`
  (`position_rider` -> `on_rider_positioned`). Tests already cover it.
- Row 43 (leash) had one half missing and is now FIXED. `AbstractBoat.remove` (L790-805)
  drops the lead on a destroying removal (`reason.shouldDestroy()` -> `dropLeash()` /
  `removeLeash()`), which Pumpkin had no equivalent for: `BoatEntity::on_removed` only
  scattered chest contents, so a broken or `/kill`-ed leashed boat left `leashed_to`
  pointing at a gone holder with no lead refunded. `on_removed` now calls
  `drop_leash()` (unleash event + lead item, entity_drops independent like vanilla)
  before the chest scatter. No new test: the seam is entity/world state, not a pure
  function; the existing leash interact path is unchanged.
- Row 42 (eject counter): unchanged, vanilla `AbstractBoat.tick` L243-247 counts both
  `UNDER_WATER` and `UNDER_FLOWING_WATER`, matching `BoatStatus::is_underwater`.
- Rows 40, 41, 44, 45, 46, 47 remain as previously resolved; `max_passengers`,
  `passenger_seat_offset` and the bubble-column metadata are single-sourced and wired
  (`entity/mod.rs:1343` uses `boat::max_passengers`).

Edited `crates/pumpkin-core/src/entity/vehicle/boat.rs` only. No generated-data, WIT or
cross-crate change. No cargo/git run (worker constraint); syntax-reviewed only.
