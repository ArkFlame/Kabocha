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
