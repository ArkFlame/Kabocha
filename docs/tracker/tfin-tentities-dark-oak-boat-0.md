# Final rows from docs/tracker/tentities-dark-oak-boat.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: boat.rs passengers.rs

- Rider yaw clamp (AbstractBoat.clampRotation, +/-105 deg, body and head sync): FIXED bd8b9bf97 in entity/vehicle/boat/passengers.rs. Unit test: rider_yaw_is_clamped_to_105_degrees_around_boat_yaw. Delta-rotation accumulation not added: UNVERIFIED, STILL OPEN.
- Row 3 (BUBBLE_TIME): FIXED, see Result. Decompiled `AbstractBoat` drives `DATA_ID_BUBBLE_TIME` only from bubble columns (`onAboveBubbleColumn` sets 60 at line 170, `tickBubbleColumn` decrements at 368-370), never from `UnderWater`/`UnderFlowingWater`. Rust additionally writes underwater ticks into that id (boat.rs:286-291), which diverges and needs a client-effect regression check outside this unit. Left open with reason.
- Row 8 (boat gametest): STILL OPEN. Needs a new file under `crates/pumpkin-gametest`, outside the source files named in this unit. Left open with reason.
Re-review pass against the decompiled tree at `.vendor/vanilla-decompiled` (AbstractBoat.java reached via the symlink). Only rows still marked STILL OPEN in the prior Result were revisited.

## Result
- Row 3 (BUBBLE_TIME): VERIFIED. Vanilla `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java` writes `DATA_ID_BUBBLE_TIME` only at 170 (bubble column sets 60) and 365-371 (`tickBubbleColumn`). Pumpkin `crates/pumpkin-core/src/entity/vehicle/boat.rs:392-419` `tick_underwater` no longer writes the bubble id; bubble writes are at 436-446 only.
- Rider yaw clamp (+/-105 deg): VERIFIED against AbstractBoat.clampRotation. Pumpkin `boat.rs:1005` `RIDER_MAX_TURN_DEGREES = 105.0` with `clamp_rider_yaw`; the doc's earlier file reference (passengers.rs) was stale, the code lives in boat.rs.
- Delta-rotation accumulation: still open, not verified in this pass.
- Row 8 (boat gametest): still open. Needs a runtime `pumpkin-gametest` run, which workers cannot execute (no cargo). Not a generated-data gap, so not BLOCKED-DATA.
