# entities / Bamboo Chest Raft

Coverage: RE-REVIEW-UNTRACKED (source changed under the previous assessment).
Audited: 2026-11-17.

Source: `crates/pumpkin-core/src/entity/vehicle/boat.rs` (`BoatEntity` serves all 22 boat and
chest boat types, including `BAMBOO_CHEST_RAFT`), submodules
`entity/vehicle/boat/{physics,passengers,dismount,block_effects}.rs`,
`entity/vehicle/vehicle.rs`, registration `entity/type.rs:350`, item placement
`item/items/boat.rs:43-44`.

Vanilla reference (decompiled): `.vendor` absent; verified against
`/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft/world/entity/vehicle/boat/`
(`AbstractBoat.java`, `AbstractChestBoat.java`, `VehicleEntity.java`, `Containers.java`).

## Verdict

No raft-specific code is missing: vanilla `Raft`/`ChestRaft` override only `rideHeight`
(0.5) and the single-rider x-offset (0.15), both already in generated data
(`entity_type.rs`). Everything below is shared `AbstractBoat`/`AbstractChestBoat` behaviour
reached through the chest raft.

## Covered (matches vanilla)

- 27-slot chest inventory, container title from the entity type with the Bedrock chest-raft
  fallback, spectator and occupied-by-other gates (`boat.rs:104-140`).
- Single passenger for chest rafts, two for plain rafts (`max_passengers`, `boat.rs:60-68`).
- `isInvulnerableTo` check, creative instabuild break without drops, entity-drops gamerule
  gate on the boat item (`boat.rs:344-380`).
- `canBeCollidedWith` = true, `isCollidable` = true, `isPickable` = `!isRemoved`
  (`boat.rs:333-340` + `Entity::is_alive` = `!is_removed`): all three match vanilla.
- `outOfControlTicks < 60` eject and boarding gate; bubble-time metadata now synced
  (`boat.rs:151-185`).
- Rider seat offsets, chest `getSinglePassengerXOffset` 0.15, chest-raft bedrock name.

## Fixed in this pass

1. Pushed-mob boarding ignored two vanilla predicates (`AbstractBoat.java:329-340`):
   `hasEnoughSpaceFor` (`entity.getBbWidth() < this.getBbWidth()`) and the
   `CANNOT_BE_PUSHED_ONTO_BOATS` entity tag. Added both to the boat pickup loop
   (`entity/mod.rs:1298-1312`).
2. Chest contents were dropped only inside the `doEntityDrops` gate, but vanilla
   `AbstractChestBoat.remove` scatters the 27 slots on any destroying removal
   (`AbstractChestBoat.java:71-77`); only the boat item is gamerule-gated
   (`VehicleEntity.destroy`). Contents are now scattered unconditionally (`boat.rs:362-380`).
3. `CONTAINER_OPEN` was never emitted when a chest boat opened; the old comment claimed
   vanilla emits it only for block containers, but `AbstractChestBoat.interact` and
   `openCustomInventoryScreen` both call `gameEvent(CONTAINER_OPEN, player)`
   (`AbstractChestBoat.java:80-98`). Emitted after a successful open (`boat.rs:130-140`).
4. `canAddPassenger` requires `!isEyeInFluid(WATER)` (`AbstractBoat.java:832`), which the
   old code comment explicitly denied. Added `boat::is_eye_in_water` and applied it to the
   player boarding path (`boat.rs:391-400`) and the pushed-mob path
   (`entity/mod.rs:1287`).

Rejected earlier claims (verified wrong this pass):

- `canBeCollidedWith` returning `true` unconditionally is vanilla (`AbstractBoat.java:129`);
  the previous tracker listed it as a gap.
- The previous "contents drop gated by `doEntityDrops`" analysis was right, the old suggested
  fix ("unconditional scatter") is now applied.

## Remaining gaps

| # | file:line | Vanilla class/method | Gap | Suggested fix | Size |
|---|---|---|---|---|---|
| 1 | `boat.rs:130-140` | `AbstractChestBoat.interact`, `openCustomInventoryScreen` | After a successful open vanilla also calls `PiglinAi.angerNearbyPiglins(level, player, true)`; Pumpkin has no anger-nearby helper | Add a piglin anger pass (radius 16, line of sight) and call it from the open path | M |
| 2 | `boat.rs:372-380` | `VehicleEntity.destroy` | The dropped boat item misses the entity custom name; vanilla copies `getCustomName()` into `DataComponents.CUSTOM_NAME` (`VehicleEntity.java:94-101`) | Copy the name when `ItemStack` components support it | S |
| 3 | `boat/passengers.rs:11-19` | `AbstractBoat.positionRider`, `clampRotation` | Rider position is set but vanilla also adds `deltaRotation` to yaw/yHeadRot for passengers without `CAN_TURN_IN_BOATS` and clamps the body rotation to the boat heading (`AbstractBoat.java:690-720`) | Port `delta_rotation` + `clamp_rotation` into `position_rider`/`synchronize_mount` | M |
| 4 | `boat/block_effects.rs` (none) | `AbstractBoat.tickBubbleColumn` | Bubble time metadata is synced, but a bubble column does not lift or sink the boat; vanilla sets vertical velocity from the column below | Read the bubble column state below the boat, set velocity, flag `velocity_dirty` | M |
| 5 | `entity/vehicle/` (none) | `AbstractBoat implements Leashable` | Boats and rafts cannot be leashed to a fence | Implement on `VehicleEntity` (shared with minecarts) | M |
| 6 | `vehicle.rs:190-215` | `VehicleEntity.destroy` | `kill_and_drop_self` rolls `minecraft:entities/<type>`; vanilla vehicles never use a loot table. Harmless today (no boat entity tables in generated data) but a datapack table would double the drop | Skip the loot roll for boat/raft entity types | S |
| 7 | `boat.rs:206-230` | `AbstractBoat.tick` -> fluid push | Boat fluid current push is skipped for client-predicted movement (engine-wide client authority difference) | Only if server-side divergence is observed | L |

Notes (checked, not gaps):

- Chest-raft/chest-boat entity loot tables do not exist in generated data (`loot_table.rs`
  has no `entities/*boat*`), so the code-level item drop is the only one.
- `AbstractBoat.hasEnoughSpaceFor` is also used by `AgeableMob` to eject oversized babies;
  Pumpkin's mob path has no equivalent call, tracked under the mob audits.
