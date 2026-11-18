# Final rows from docs/tracker/tentities-bamboo-chest-raft-451500.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: passengers.rs

- Rider yaw clamp (AbstractBoat.clampRotation, +/-105 deg, body and head sync): FIXED bd8b9bf97 in entity/vehicle/boat/passengers.rs. Unit test: rider_yaw_is_clamped_to_105_degrees_around_boat_yaw. Delta-rotation accumulation not added: UNVERIFIED, STILL OPEN.
- Delta-rotation accumulation (h6 STILL OPEN): CLOSED as not a server gap. Vanilla
- Row 6 (`CONTAINER_CLOSE`, S): STILL OPEN. Needs a close hook that emits at the vehicle exact
- Row 3 (bubble column, M): STILL OPEN. Needs the bubble-column block inside-path wiring plus new

## Result (FINAL PASS 2026-11-18, all rows terminal)

Re-verified each row against decompiled vanilla and the current on-disk Pumpkin source, smallest
first. The nominal `.vendor/vanilla-decompiled/net/minecraft/` copy is absent on disk, so the cited
vanilla source is
`/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft/world/entity/vehicle/boat/`
(`AbstractBoat.java`, `AbstractChestBoat.java`) plus `.../world/entity/Entity.java` for the
authority helpers. The rider-clamp helpers moved from `passengers.rs` into `boat.rs` since the
earlier passes, so row locations below are updated. All actionable rows are already fixed in the
current on-disk source, so no source file needed editing: no rustfmt was applied and no cargo run
was performed (CODER rule); compile UNVERIFIED.

- Rider yaw clamp (+/-105 deg, body and head sync): FIXED. Vanilla `AbstractBoat.clampRotation`
  (AbstractBoat.java:735-749) is ported as `clamp_rider_yaw` (boat.rs:1009, `RIDER_MAX_TURN_DEGREES
  = 105.0` at :1005) and `rider_clamp_adjustment` (:1018), with body-yaw sync in
  `on_rider_positioned` / `on_passenger_turned` (:856-949) and three pure unit tests (:1038-1060).
  The move from passengers.rs to boat.rs kept identical behavior; no new test seam exists.
- Delta-rotation accumulation: VERIFIED, matches vanilla, not a server gap. Vanilla
  `AbstractBoat.positionRider` (AbstractBoat.java:691-701) adds `deltaRotation` only under
  `isLocalInstanceAuthoritative()`; per Entity.java:6550-6561 that is false for a dedicated-server
  player passenger (client authoritative) and true for a server-authoritative mob rider, so the
  add is server-relevant only for non-player riders. Pumpkin's `on_rider_positioned` (boat.rs:856-899)
  skips players and `CAN_TURN_IN_BOATS` riders and applies the derived `yaw_delta` to non-player
  riders, with pure unit test `rider_yaw_follows_boat_yaw_delta_then_clamps` (:1045-1052). No missing
  generated field or subsystem.
- Row 6 (`CONTAINER_CLOSE`, S): FIXED. Vanilla `AbstractChestBoat.stopOpen`
  (AbstractChestBoat.java:197-198) emits `GameEvent.CONTAINER_CLOSE` at `this.position()` with the
  `ContainerUser` living-entity context. `Player::open_handled_screen_at` stores the exact vehicle
  position in `open_container_entity_pos` (player.rs:7995-7996), fed by `container::open`
  (container.rs:293-302 passes `block_pos=None, container_entity=Some(entity)`);
  `on_handled_screen_closed` reads it (player.rs:7771) and emits `GameEvent::ContainerClose` at that
  exact position with the player as source (player.rs:7832-7843), clearing it afterward (:7846).
  `block_pos=None` prevents a generic `CONTAINER_OPEN` double-emit. No source change needed.
- Row 3 (bubble column, M): FIXED. Vanilla `AbstractBoat.onAboveBubbleColumn` / `tickBubbleColumn`
  / `handleBubbleColumnEffect` (AbstractBoat.java:165, 346-402) arm a 60-tick timer, eject riders
  and broadcast BOAT_SINK on drag-down, then set y = current_y - 0.7 (down) or 0.6 (up), guarded by
  `canSimulateMovement()` (Entity.java:6568-6574; false for a player-steered boat). Pumpkin wires the
  block inside-path (`bubble_column::entity_inside`, bubble_column.rs:205-219) to the boat hooks
  (`on_above_bubble_column` boat.rs:647, `on_inside_bubble_column` :700), arms/decrements the
  countdown in `tick_bubble_column` (:429-474) with rider eject, BoatSink/BoatLaunch events and the
  client-controlled skip, and applies `bubble_column_vertical_velocity` (:976, y-0.7 / 0.6) with a
  pure unit test (:1076-1078). No source change needed.

Final tallies: FIXED=3 (rider clamp, row 6, row 3), VERIFIED=1 (delta-rotation), BLOCKED-DATA=0,
STILL OPEN=0. No source file changed this pass.
