# Parity rows for own-tentities-spruce-boat-8ca801

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/tentities-spruce-boat-8ca801** [partial/OWN-OPEN] own audit: 9 open, 3 fixed, 1 verified, 0 blocked-data (docs/tracker/tentities-spruce-boat-8ca801.md) docs/tracker/tentities-spruce-boat-8ca801.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)


## Result (2026-11-18, final pass)

Re-verified each row in `docs/tracker/tentities-spruce-boat-8ca801.md` against decompiled
vanilla at `.vendor/vanilla-decompiled/net/minecraft/`. All rows are already resolved in the
current checkout, so no source file needed changes and no test was added this pass.

- Row 1 / h6 rider rotation: VERIFIED. `crates/pumpkin-core/src/entity/vehicle/boat.rs` stores
  `yaw_delta` (:232/:258/:593), applies the delta before the +/-105 clamp in `on_rider_positioned`
  (:856-899), and `on_passenger_turned` (:901-949) folds the clamp adjustment into
  `rider.y_rot_o` (:948). Matches `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java`
  `positionRider` :696-705, `clampRotation` :740-754 (`Mth.clamp(delta, -105.0F, 105.0F)`) and
  `onPassengerTurned` :763-765 (`passenger.yRotO += yDelta`). Three pure unit tests cover the
  clamp/delta math (`rider_yaw_clamps_to_boat_heading_window`,
  `rider_yaw_follows_boat_yaw_delta_then_clamps`, `rider_clamp_adjustment_feeds_previous_yaw`).
  No action.
- Row 2 (dead `move_from_client` / `correct_client_position`): VERIFIED removed. `grep` for
  `move_from_client|correct_client_position|CMoveVehicle` in
  `crates/pumpkin-core/src/entity/vehicle/boat/physics.rs` returns no matches; the generic
  MoveVehicle packet supersedes the boat path. No action.
- Row 3 (boat gametest): VERIFIED present and registered. `test_chest_boat_scatters_slots_on_kill`
  exists in `server/builtin_game_tests/boat_block_effects.rs:386`, is in the id list at
  `builtin_game_tests/mod.rs:280`, and is registered with `register_function_test` at :1868-1874.
  It guards the `EntityBase::kill` container-scatter fix against
  `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractChestBoat.java`
  `remove` :71-76 (`Containers.dropContents` on `RemovalReason.KILLED`). No action.

Fixed: 0 (nothing new; all rows already fixed/verified). Open: 0. Files changed: this doc only.
No .rs file touched, so rustfmt not run.
