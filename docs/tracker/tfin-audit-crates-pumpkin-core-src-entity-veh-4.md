# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-entity-veh.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- block_effects 2 (endpoint double dispatch): STILL OPEN, M. Same root cause as row 1; needs
- block_effects 6 (gametest): STILL OPEN, M. Needs a `pumpkin-gametest` scenario (boat over
- minecart 7 (passenger repositioning): STILL OPEN, S, behavioural. Needs vanilla
- minecart 8 (slope tests): STILL OPEN, S. Test-only, but expected positions must come from

## Result

All four rows are already fixed/verified in the current tree (no source edit needed this pass;
verified against decompiled vanilla and the current source).

- block_effects 2 (endpoint double dispatch): FIXED. The sweep in
  `entity/vehicle/boat/block_effects.rs:71-118` skips every position inside the final standing
  box (`standing_min`/`standing_max` guard at :106-114), leaving those blocks to the generic
  `tick_block_collisions` endpoint pass, so a block is dispatched at most once per tick. Mirrors
  vanilla `Entity.visitedBlocks` dedupe in
  `.vendor/vanilla-decompiled/net/minecraft/world/entity/Entity.java:1951-1977,2041` (the set is
  filled in `checkInsideBlocks` and cleared at :1977). The `visited` HashSet at :71/:103 dedupes
  within the sweep itself.

- block_effects 6 (gametest): FIXED. `server/builtin_game_tests/boat_block_effects.rs` provides
  `test_boat_breaks_lily_pad_on_landing` (:10) and `test_boat_sweep_ignites_on_fire` (:341), which
  drive a boat over lily pads and over a fire row respectively. Both are registered in
  `server/builtin_game_tests/mod.rs:1817,1824` (module declared at :1). The fire scenario pins
  the non-lily-pad sweep dispatch of row 1 and the no-double-dispatch behaviour of row 2.
  Vanilla reference for fire/lily-pad `entityInside`:
  `.vendor/vanilla-decompiled/net/minecraft/world/entity/Entity.java:2041-2047`.

- minecart 7 (passenger repositioning): VERIFIED faithful, no change. `entity/vehicle/minecart.rs:757-761`
  calls `crate::entity::position_riders(self)` after a successful move
  (`attempted_to != from`), which places each rider at its seat via
  `position_rider` (`entity/mod.rs:1664-1690`: `vehicle.pos + seat - offset`, per-entity attachment
  point). Vanilla `Minecart.positionRider`
  (`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/minecart/Minecart.java:77-85`)
  calls `super.positionRider(passenger, moveFunction)` for this server-side placement; the only
  extra override logic is client-side player rotation (:80-84), a client-only effect that does not
  apply on the server.

- minecart 8 (slope tests): FIXED. `entity/vehicle/minecart.rs` pins `get_pos_on_rail` with
  `rail_position_follows_ascending_slope` (:1190), `all_ascending_rails_snap_center_to_vanilla_height`
  (:1215, all four Ascending* shapes) and `rail_position_snap_covers_north_south_and_curve` (:1235).
  The implementation at :147-186 is a 1:1 port of vanilla `OldMinecartBehavior.getPos`
  (`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/minecart/OldMinecartBehavior.java:304-347`),
  including the `+0.0625` rail offset, `yD = (y1 - y0) * 2.0`, and the `yD < 0 -> y++`,
  `yD > 0 -> y += 0.5` slope correction the tests assert.

No source file was edited this pass (all four rows already satisfied); only this Result section was
updated, so there are no compile-pitfall or rustfmt concerns.
