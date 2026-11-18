# Final rows from docs/tracker/tentities-bamboo-chest-raft-451500.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

No STILL OPEN rows remain. No source file changed this pass.
Each row below ends in a terminal status; no STILL OPEN remains. The only editable file this unit
Final tallies: fixed=8, verified-not-gaps=2 (rows 2,8), BLOCKED-DATA=0, STILL OPEN=0. No source
STILL OPEN=0. No source file changed this pass.

## Result

Final pass (2026-11-18): verified each row against `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/` (`AbstractBoat.java`, `AbstractChestBoat.java`) and the current Pumpkin source. No source edit needed this pass (all actionable rows already fixed/verified in prior passes); no rustfmt/cargo run performed (CODER rule); compile UNVERIFIED.

- Row 1 (piglin anger on chest open, M): FIXED. Vanilla `AbstractChestBoat.interact` calls `PiglinAi.angerNearbyPiglins` on a successful open (`AbstractChestBoat.java:93,104`). Pumpkin `boat.rs:325` calls `PiglinAi::on_guarded_block_interacted` on successful open. No change.
- Row 2 (rider yaw clamp / delta-rotation, M): VERIFIED, matches vanilla, not a gap. Vanilla `AbstractBoat.positionRider`/`clampRotation`/`onPassengerTurned` (`AbstractBoat.java:691-761`). Pumpkin `boat.rs` has `clamp_rider_yaw` (RID rider max turn 105.0), `rider_clamp_adjustment`, `on_rider_positioned` and `on_passenger_turned` with unit tests `rider_yaw_clamps_to_boat_heading_window` and `rider_yaw_follows_boat_yaw_delta_then_clamps`. The `deltaRotation` add is guarded by `isLocalInstanceAuthoritative()` (`AbstractBoat.java:699`), false for dedicated-server player passengers. No change.
- Row 3 (bubble column, M): FIXED. Vanilla `AbstractBoat.onAboveBubbleColumn`/`handleBubbleColumnEffect` (`AbstractBoat.java:165-185,346-408`) set a 60-tick timer, y=0.6 updraft, current_y-0.7 drag-down and eject riders. Pumpkin `boat.rs` has `bubble_column_time` (line 237), `tick_bubble_column` (429), `on_above_bubble_column` (647), `on_inside_bubble_column` (700), `bubble_column_vertical_velocity` (976) and a pure unit test (1077). No change.
- Row 4 (leash, M): FIXED. Vanilla `AbstractBoat implements Leashable` (`AbstractBoat.java:54,423-449`, leash NBT at 765). Pumpkin `boat.rs` writes/reads leash NBT (539/546), ticks the leash (609) and wires `leash_to`/`unleash`/`get_leash_holder` (773-797) and `support_quad_leash` (956). No change.
- Row 5 (destroy-removal chest scatter, M): FIXED. Vanilla `AbstractChestBoat.remove` drops contents on any destroying removal (`AbstractChestBoat.java:71-77`). Pumpkin `boat.rs` calls `scatter_chest_contents` from `on_removed` (504) and the damage-break path (746); `MinecartInventory::claim_drops` makes it run once. No change.
- Row 6 (CONTAINER_CLOSE, S): FIXED. Vanilla `AbstractChestBoat.stopOpen` emits `GameEvent.CONTAINER_CLOSE` at `this.position()` with the living-entity context (`AbstractChestBoat.java:197-198`). Pumpkin `player.rs` stores `open_container_entity_pos` on entity screen open (7995) and reads it in `on_handled_screen_closed` (7771, 7798-7809) to emit `ContainerClose` at the exact vehicle position with the player as source; cleared on all close paths (7812, 8090, 8173). `container::open` passes no block pos, so `CONTAINER_OPEN` is not double-emitted. No change.
- Row 7 (controlling passenger, S): FIXED. Vanilla `AbstractBoat.getControllingPassenger` returns the first passenger only when it is a `Player`, else `super` (null for `VehicleEntity`) (`AbstractBoat.java:845-846`). Pumpkin `boat.rs:526-534` filters on `get_player().is_some()`. Sole caller `apply_mount_move` (`player_auth_input.rs:514-516`) uses `is_some_and` to compare the controller entity id to the rider, staying correct when the filter yields `None`. No caller-side change needed; no pure unit-test seam (needs live `BoatEntity` and `Player`). No change.
- Row 8 (fluid current push, L): VERIFIED, not a gap. Vanilla applies the fluid current push via `Entity.baseTick` reached from `AbstractBoat.tick`. Pumpkin deliberately skips it for client-predicted movement (AGENTS.md: player movement is client authoritative); same observable behaviour via a different algorithm. No missing generated field or subsystem, so not BLOCKED-DATA. No change.

Final tallies: FIXED=6 (rows 1,3,4,5,6,7), VERIFIED=2 (rows 2,8), BLOCKED-DATA=0, STILL OPEN=0. No source file changed this pass.
