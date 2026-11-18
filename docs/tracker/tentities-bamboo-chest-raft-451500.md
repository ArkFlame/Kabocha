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

## Result

Re-reviewed each gap against `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/`
(`AbstractBoat.java`, `AbstractChestBoat.java`) and `VehicleEntity.java`, smallest rows first.

- Row 7 (`get_controlling_passenger`, S): FIXED. `AbstractBoat.java:845-847` returns the first
  passenger only when it is a `Player`, else `super.getControllingPassenger()` (null for
  `VehicleEntity`). `boat.rs` filtered on `get_living_entity`, so a non-player mob rider was
  wrongly treated as the controller. Now filters on `get_player().is_some()` and the
  `is_client_controlled` comment was updated to match. All callers (`entity/mod.rs`, `boat.rs`,
  `player_auth_input.rs`) already expect a player, so no caller change was needed. No pure unit
  test seam: the filter needs a live `BoatEntity` with passengers plus a `Player`, and a
  getter/filter mirror test is forbidden by the test rules.
- Delta-rotation accumulation (h6 STILL OPEN): CLOSED as not a server gap. Vanilla
  `AbstractBoat.positionRider` (`AbstractBoat.java:696-706`) adds `deltaRotation` to rider
  yaw/yHeadRot only under `passenger.isLocalInstanceAuthoritative()`, which is false on a dedicated
  server for player passengers (`Player.isClientAuthoritative()` returns true and
  `Entity.isLocalInstanceAuthoritative()` is `!isClientAuthoritative()` when `!level.isClientSide`).
  The server-relevant `clampRotation` half is already ported (bd8b9bf97). No code change.
- Row 5 (destroy-removal chest scatter, M): CLOSED, already fixed. `on_removed` and the destroy
  branch in `damage_with_context` both call `scatter_chest_contents`, with
  `MinecartInventory::claim_drops` making it run once; matches `AbstractChestBoat.remove`.
- Row 1 (piglin anger, M): CLOSED, already wired. `open_custom_inventory_screen` calls
  `PiglinAi::on_guarded_block_interacted` on a successful open.
- Row 6 (`CONTAINER_CLOSE`, S): STILL OPEN. Needs a close hook that emits at the vehicle exact
  position; `Player::on_handled_screen_closed` emits from a `BlockPos` (`player.rs`, outside this
  unit). Passing a block pos from `container::open` would double-emit `CONTAINER_OPEN` and run
  block lock checks, so it is not a safe local fix.
- Row 3 (bubble column, M): STILL OPEN. Needs the bubble-column block inside-path wiring plus new
  boat state fields and tick velocity; cross-cutting, outside the editable files.
- Row 4 (leash, M): STILL OPEN. Needs lead-on-entity interaction (currently mobs only) and
  `write_leash_data`/`read_leash_data` in the boat NBT path; cross-crate/cross-cutting.
- Row 8 (fluid push, L): STILL OPEN. Engine-wide client-authority difference; skip unless server
  divergence is observed, per the vanilla-port guide.

Verification: `rustfmt --edition 2024 crates/pumpkin-core/src/entity/vehicle/boat.rs` succeeded.
No cargo run (CODER rule); compile UNVERIFIED.


## Result (re-review 2026-11-18, scope player_auth_input.rs)

Re-verified every STILL OPEN row against .vendor/vanilla-decompiled, smallest first. This unit may
edit only player_auth_input.rs, and none of the open rows live in that file, so no source change was
made and no unit-test seam exists here.

- Row 6 (CONTAINER_CLOSE, S): CONFIRMED OPEN. Vanilla AbstractChestBoat.stopOpen
  (AbstractChestBoat.java:197-199) emits GameEvent.CONTAINER_CLOSE at this.position() with the user
  context. Pumpkin's shared vehicle screen close (entity/vehicle/minecart/container.rs:210-212
  on_closed) only delegates to inner and emits no CONTAINER_CLOSE. One-line reason: the close hook
  lives in container.rs/player.rs, outside the editable file, and passing a block pos from
  container::open would double-emit CONTAINER_OPEN.
- Row 3 (bubble column, M): verified vanilla AbstractBoat.onAboveBubbleColumn /
  handleBubbleColumnEffect (updraft 0.6, whirlpool current_y - 0.7, 60-tick timer, rider eject).
  Current boat.rs already has the bubble fields, tick_bubble_column, on_above_bubble_column,
  on_inside_bubble_column, bubble_column_vertical_velocity (0.6 / y-0.7) and a pure unit test.
  One-line reason: implemented in boat.rs/block wiring, outside the editable file.
- Row 4 (leash, M): verified vanilla AbstractBoat Leashable. Current boat.rs already wires
  write_leash_data/read_leash_data, tick_leash, the lead/unleash interaction and support_quad_leash.
  One-line reason: lead interaction and NBT live in boat.rs/mob interaction, outside the editable file.
- Row 8 (fluid push, L): engine-wide client-authority difference; per the vanilla-port guide it is
  skipped unless server-side divergence is observed. One-line reason: not an input-codec bug.

Caller check for the row 7 fix: apply_mount_move (player_auth_input.rs:512-514) reads
get_controlling_passenger().is_some_and(|c| c.entity_id == rider.entity_id). That already returns
false when the new player-only filter yields None, and the controller is required to be the player
rider, so no caller-side change is needed. No pure seam for a unit test (needs live BoatEntity and
Player instances), so none was added.

No source file changed. rustfmt not applicable (no .rs edit). No cargo run (CODER rule).


## Result (re-verification 2026-11-18, scope player_auth_input.rs)

Re-verified every STILL OPEN row against .vendor/vanilla-decompiled, smallest first, and against
the current on-disk Pumpkin source. This unit may edit only player_auth_input.rs; none of the open
rows live in that file, so no source change was possible and no unit-test seam exists here.

- Row 6 (CONTAINER_CLOSE, S): CONFIRMED OPEN. Vanilla AbstractChestBoat.stopOpen
  (AbstractChestBoat.java:197-198) emits GameEvent.CONTAINER_CLOSE at this.position() with the
  living-entity context. Current Pumpkin entity/vehicle/minecart/container.rs:210-211 on_closed
  only calls self.inner.on_closed(player) and emits no CONTAINER_CLOSE. One-line reason: the close
  hook lives in container.rs/player.rs, outside the editable file, and player_auth_input.rs handles
  only auth-input movement/inventory actions, not screen close.
- Row 3 (bubble column, M): CONFIRMED ALREADY FIXED in boat.rs. Current source has bubble_column_time
  / bubble_column_drag_down / bubble_column_above fields, tick_bubble_column (velocity y 0.6 /
  y-0.7 via bubble_column_vertical_velocity), on_above_bubble_column and a pure unit test. One-line
  reason: implemented in boat.rs/block wiring, outside the editable file.
- Row 4 (leash, M): CONFIRMED ALREADY FIXED in boat.rs. Current source wires write_leash_data /
  read_leash_data (boat.rs:521-530), tick_leash (:592-593), leash_to / unleash / get_leash_holder
  (:757-781) and support_quad_leash (:940). One-line reason: lead interaction and NBT live in
  boat.rs, outside the editable file.
- Row 8 (fluid push, L): engine-wide client-authority difference; per the vanilla-port guide it is
  skipped unless server divergence is observed. One-line reason: not an input-codec bug.

Row 7 caller recheck: apply_mount_move (player_auth_input.rs:514-516) reads
get_controlling_passenger().is_some_and(|c| c.entity_id == rider.entity_id), which stays correct
under the player-only controller filter (returns false when the filter yields None). No caller-side
change needed. No pure seam for a unit test (would need live BoatEntity and Player), so none added.

No source file changed. rustfmt not applicable (no .rs edit). No cargo run (CODER rule).


## Result (re-review 2026-11-18, scope player_auth_input.rs)

Re-verified every row still marked STILL OPEN against .vendor/vanilla-decompiled, smallest first,
and against the current on-disk Pumpkin source. This unit may edit only player_auth_input.rs; none
of the open rows live in that file, so no source change was possible and no pure unit-test seam
exists here.

- Row 6 (CONTAINER_CLOSE, S): CONFIRMED OPEN. Vanilla AbstractChestBoat.stopOpen
  (AbstractChestBoat.java:197-198) emits GameEvent.CONTAINER_CLOSE at this.position() with the
  living-entity context. Current Pumpkin entity/vehicle/minecart/container.rs:210-211 on_closed
  only calls self.inner.on_closed(player) and emits no CONTAINER_CLOSE. One-line reason: the close
  hook lives in container.rs/player.rs, outside the editable file, and player_auth_input.rs handles
  auth-input movement/inventory only, not screen-close; passing a block pos from container::open
  would double-emit CONTAINER_OPEN.
- Row 3 (bubble column, M): CONFIRMED ALREADY FIXED in boat.rs. Vanilla
  handleBubbleColumnEffect (AbstractBoat.java:399-408) applies -0.7 add to current y on drag-down
  and sets y to 0.6 on updraft; Pumpkin bubble_column_vertical_velocity (boat.rs:971-973) returns
  current_y - 0.7 / 0.6 with a pure unit test, plus bubble fields, tick_bubble_column and
  on_above_bubble_column. One-line reason: implemented in boat.rs/block wiring, outside the file.
- Row 4 (leash, M): CONFIRMED ALREADY FIXED in boat.rs. Vanilla AbstractBoat implements Leashable
  (AbstractBoat.java:54,428-449). Pumpkin wires write_leash_data/read_leash_data (:534-541),
  tick_leash (:603-604), leash_to/unleash/get_leash_holder (:763-792) and support_quad_leash (:951).
  One-line reason: lead interaction and NBT live in boat.rs, outside the editable file.
- Row 8 (fluid push, L): engine-wide client-authority difference; per the vanilla-port guide it is
  skipped unless server-side divergence is observed. One-line reason: not an input-codec bug.

Row 7 caller recheck: apply_mount_move (player_auth_input.rs:514-516) reads
get_controlling_passenger().is_some_and(|c| c.entity_id == rider.entity_id), which stays correct
under the player-only controller filter (returns false when the filter yields None). No caller-side
change needed. No pure seam for a unit test (would need live BoatEntity and Player), so none added.

No source file changed. rustfmt not applicable (no .rs edit). No cargo run (CODER rule); compile
UNVERIFIED.


## Result (SECOND PASS 2026-11-18, expanded scope)

SECOND PASS grants edits to any source file a row needs. Re-verified every STILL OPEN row against
.vendor/vanilla-decompiled and the current on-disk Pumpkin source, smallest first.

- Row 6 (CONTAINER_CLOSE, S): FIXED in current source. Player::open_container_entity_pos
  (player.rs:762) is set from container_entity.pos in open_handled_screen_at (player.rs:7945-7946)
  and read in on_handled_screen_closed (player.rs:7721, 7782-7793), which emits
  GameEvent::ContainerClose at the vehicle exact position with the player as source. This matches
  vanilla AbstractChestBoat.stopOpen (AbstractChestBoat.java:197-198). The field is cleared on every
  close path (:7796, 8074, 8123). container::open passes block_pos=None, so CONTAINER_OPEN is not
  double-emitted. No source change needed this pass.
- Row 3 (bubble column, M): ALREADY FIXED in boat.rs (bubble_column_time/drag_down/above fields,
  tick_bubble_column :429, on_above_bubble_column :642, on_inside_bubble_column :695,
  bubble_column_vertical_velocity :971 with updraft 0.6 and drag-down current_y-0.7, pure unit test
  :1071). Vanilla AbstractBoat.onAboveBubbleColumn/handleBubbleColumnEffect matched. No change.
- Row 4 (leash, M): ALREADY FIXED in boat.rs (write_leash_data :534, read_leash_data :541,
  tick_leash :604, leash_to/unleash :773/792, support_quad_leash :951). Vanilla AbstractBoat
  implements Leashable. No change.
- Row 8 (fluid push, L): engine-wide client-authority difference; skipped per the vanilla-port guide
  unless server divergence is observed. No change.

No source file changed (all STILL OPEN rows already resolved in current source). No pure unit-test
seam exists for the close event (needs a live Player, world and screen handler). No cargo run
(CODER rule); compile UNVERIFIED.
