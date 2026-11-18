# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-6aac9c.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

## Result

Final pass. Both rows were already fixed in earlier passes; no source edit was needed here. The
fixes in `crates/pumpkin-core/src/entity/vehicle/minecart.rs` are present and verified below.

- Gap 9 (S) FIXED / VERIFIED. `minecart.rs:757-761` calls
  `crate::entity::position_riders(self)` when the cart moves, and the `passenger_attachment_point`
  override (`:342-350`) supplies the lowered villager/wandering-trader seat offset. `position_rider`
  in `crates/pumpkin-core/src/entity/mod.rs:1664-1690` places each rider at
  `vehicle.pos + seat - offset`, matching vanilla `Entity.positionRider(passenger, moveFunction)`
  at `.vendor/vanilla-decompiled/net/minecraft/world/entity/Entity.java:3540-3544`
  (`getPassengerRidingPosition` minus `passenger.getVehicleAttachmentPoint`). Riders are no longer
  teleported to the cart centre, so the per-entity attachment offsets are preserved.
- Gap 10 (S) FIXED / VERIFIED. Table-driven `rail_position_snap_covers_north_south_and_curve` test
  at `minecart.rs:1263-1290` covers flat north-south and a south-east curve. Expected values were
  derived from decompiled `OldMinecartBehavior.getPos` at
  `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/minecart/OldMinecartBehavior.java:304-351`:
  `y = floor(y) + 0.0625` (Pumpkin `RAIL_HEIGHT_OFFSET = 0.0625`, `minecart.rs:189`) and, for the
  diagonal branch, `progress = (xx*xD + zz*zD)*2`, which yields `(0.75, ., 0.75)` for the
  south-east curve case asserted in the test.

No row remains open. No BLOCKED-DATA: no generated `pumpkin-data` field or subsystem is
missing. No code change this pass; rustfmt, cargo and git were not run by this worker (orchestrator
handles them).
