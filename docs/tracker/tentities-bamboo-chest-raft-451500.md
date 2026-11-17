# entities / Bamboo Chest Raft

Coverage: RE-REVIEW-UNTRACKED (source changed under the previous assessment).
Audited: 2026-11-17 (re-review after 4c40845b3).

Source: `crates/pumpkin-core/src/entity/vehicle/boat.rs` (`BoatEntity` serves all 22 boat and
chest boat types, including `BAMBOO_CHEST_RAFT`), submodules
`entity/vehicle/boat/{physics,passengers,dismount,block_effects}.rs`,
`entity/vehicle/vehicle.rs`, container glue `entity/vehicle/minecart/container.rs`,
registration `entity/type.rs:350`, item placement `item/items/boat.rs:43-44`.

Vanilla reference (decompiled): `.vendor` absent; verified against
`/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft/world/entity/vehicle/boat/`
(`AbstractBoat.java`, `AbstractChestBoat.java`, `Raft.java`, `ChestRaft.java`) and
`.../vehicle/VehicleEntity.java`.

## Verdict

No raft-specific code is missing: vanilla `Raft`/`ChestRaft` override only `rideHeight`
(`dimensions.height() * 0.8888889F`), and `AbstractChestBoat.getSinglePassengerXOffset` is
0.15. Both are already in generated data (`entity_type.rs`: `ride_height: 0.5`,
`single_passenger_x_offset: 0.15` for `BAMBOO_CHEST_RAFT`/`BAMBOO_RAFT`). Everything below is
shared `AbstractBoat`/`AbstractChestBoat` behaviour reached through the chest raft.

## Changed since the previous audit

1. `4c40845b3 fix(entity): restore boat destroy drop parity` re-added the custom name on the
   dropped boat item (`boat.rs:407-411`). That is correct: `VehicleEntity.destroy(level, dropItem)`
   sets `DataComponents.CUSTOM_NAME` to `getCustomName()` before `spawnAtLocation`, inside the
   `ENTITY_DROPS` gate. Previous gap 2 is closed.
2. Previous gap 6 (boat entity loot table would double-drop) is dropped as unreachable:
   vehicle loot tables are generated static data with no `entities/*boat*` entry, and the boat
   path uses its own drop code, never `VehicleEntity::kill_and_drop_self`.

## Covered (matches vanilla, re-verified this pass)

- 27-slot chest inventory, container title from the entity type with the Bedrock chest-raft
  fallback, spectator and occupied-by-other gates (`boat.rs:115-160`).
- Single passenger for chest rafts, two for plain rafts (`max_passengers`, `boat.rs:60-68`),
  matching `AbstractChestBoat.getMaxPassengers` and `AbstractBoat.getMaxPassengers`.
- `damage_with_context`: `isInvulnerableTo` check, wobble (`animateHurt`: hurt dir negation,
  hurt time 10, damage *11 path via `apply_damage_wobble`), `ENTITY_DAMAGE` game event,
  creative instabuild break without drops, chest contents scatter, custom-named boat item
  gated by `doEntityDrops` and skipped for explosions (`boat.rs:361-419`).
- `canBeCollidedWith` = true, `isCollidable` = true, `isPickable` = `!isRemoved`
  (`boat.rs:353-359`): all match `AbstractBoat.java:129-137,225-227`.
- `outOfControlTicks < 60` eject and boarding gate; bubble-time metadata synced
  (`boat.rs:179-211`).
- Pushed-mob boarding honours `hasEnoughSpaceFor` and `CANNOT_BE_PUSHED_ONTO_BOATS`
  (`entity/mod.rs:1293-1313`, `AbstractBoat.java:329-340`).
- `canAddPassenger` = seats left and `!isEyeInFluid(WATER)` (`boat.rs:421-434`,
  `AbstractBoat.java:831-833`).
- `CONTAINER_OPEN` emitted after a successful open (`boat.rs:152-159`), matching
  `AbstractChestBoat.interact`/`openCustomInventoryScreen`.
- Rider seat offsets incl. animal shift and 0.15 chest offset (`boat.rs:232-264`,
  `AbstractBoat.java:146-162`).

## Remaining gaps

| # | file:line | Vanilla class/method | Gap | Suggested fix | Size |
|---|---|---|---|---|---|
| 1 | `boat.rs:152-159` | `AbstractChestBoat.interact`, `openCustomInventoryScreen` | After a successful open vanilla also calls `PiglinAi.angerNearbyPiglins(level, player, true)`; Pumpkin has no anger-nearby helper | Add a piglin anger pass (radius 16, line of sight) and call it from the open path | M |
| 2 | `boat/passengers.rs:12-16` | `AbstractBoat.positionRider`, `clampRotation`, `onPassengerTurned` (`AbstractBoat.java:690-761`) | Rust only teleports the rider. Vanilla adds `deltaRotation` to yaw/yHeadRot for riders without `CAN_TURN_IN_BOATS`, clamps body yaw to the boat heading (animal offset 90/270 when full), and clamps yaw to +-105 degrees | Port `delta_rotation`, `clamp_rotation` and `on_passenger_turned` into `position_rider`/`synchronize_mount` | M |
| 3 | `boat/block_effects.rs` (none) | `AbstractBoat.onAboveBubbleColumn`, `tickBubbleColumn`, `handleBubbleColumnEffect` (`AbstractBoat.java:165-185,346-403`) | Bubble time only tracks eye-in-water; a bubble column never sets it to 60, launches (velocity y 0.6) or sinks (-0.7) and ejects riders at the end | Add bubble-column state (time, direction, is-above flag), feed it from the bubble column block entity-inside path, apply the velocity in `tick` | M |
| 4 | `entity/vehicle/` (none) | `AbstractBoat implements Leashable` (`AbstractBoat.java:423-445,763-771`) | Boats and rafts cannot be leashed: lead use only handles mobs (`mob/mod.rs:1276`), and boat NBT writes no leash data | Wire lead-on-entity interaction for vehicles and `write_leash_data`/`read_leash_data` in the boat save path | M |
| 5 | `boat.rs:385-399` | `AbstractChestBoat.remove` (`AbstractChestBoat.java:71-77`) | Chest contents are scattered only in the damage-break path. Vanilla drops contents for any destroying removal (`/kill`, out-of-world) | Add a destroy-removal hook for chest vehicles (scatter once, honour `claim_drops`) | M |
| 6 | `minecart/container.rs:206-237` | `AbstractChestBoat.stopOpen` (`AbstractChestBoat.java:196-199`) | No `CONTAINER_CLOSE` game event when a boat container closes; vanilla emits it with the container position and user context | Emit `CONTAINER_CLOSE` from the vehicle screen handler close path (shared with chest minecarts) | S |
| 7 | `boat.rs:266-274` | `AbstractBoat.getControllingPassenger` (`AbstractBoat.java:839-842`) | Rust returns the first living passenger; vanilla returns the first passenger only if it is a `Player`, otherwise `super.getControllingPassenger()` (null for `VehicleEntity`) | Return `None` unless the first passenger is a player; check generic callers at `entity/mod.rs:1650,2664,3867,3882` first | S |
| 8 | `boat.rs:317-347` | `AbstractBoat.tick` -> `Entity.baseTick` fluid push | Boat fluid current push is skipped for client-predicted movement (engine-wide client authority difference) | Only if server-side divergence is observed | L |

Notes (checked, not gaps):

- Chest-raft/chest-boat entity loot tables do not exist in generated data, so the code-level
  item drop is the only one.
- `AbstractBoat.hasEnoughSpaceFor` is also used by `AgeableMob` to eject oversized babies;
  Pumpkin mob path has no equivalent call, tracked under the mob audits.
- `AbstractBoat.checkFallDamage` only maintains `lastYd`/`fallDistance`; `last_yd` is already
  fed by `physics.rs:250,281` and boats take no fall damage.
- `getPickResult` returns a plain item; Pumpkin has no entity pick-block path.

## h6 status
- Rider yaw clamp (AbstractBoat.clampRotation, +/-105 deg, body and head sync): FIXED bd8b9bf97 in entity/vehicle/boat/passengers.rs. Unit test: rider_yaw_is_clamped_to_105_degrees_around_boat_yaw. Delta-rotation accumulation not added: UNVERIFIED, STILL OPEN.
