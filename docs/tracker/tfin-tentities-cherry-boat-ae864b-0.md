# Final rows from docs/tracker/tentities-cherry-boat-ae864b.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: boat.rs dismount.rs mod.rs passengers.rs

- Row "Delta-rotation accumulation not added: STILL OPEN" (h6 status): VERIFIED ALREADY FIXED, no code change needed. Current code implements it: BoatEntity.yaw_delta (boat.rs:169, init :193) is recomputed every tick from consecutive yaw samples (boat.rs:436-439 wrap_degrees(yaw - last_yaw)), and on_rider_positioned (boat.rs:666-677) applies rider_yaw_with_delta(rider.yaw, boat.yaw, yaw_delta) to non-player riders, syncing body/head yaw. Pure-function unit test rider_yaw_follows_boat_yaw_delta_then_clamps exists (boat.rs:742-752). This matches decompiled AbstractBoat.positionRider (AbstractBoat.java:696-705: add deltaRotation to YRot/YHeadRot, then clampRotation) and clampRotation (AbstractBoat.java:740-754: body yaw = boat yaw, head clamped to +-105 deg). The doc h6 note is stale: the clamp test was renamed to rider_yaw_clamps_to_boat_heading_window (boat.rs:735) and the delta logic lives in boat.rs, not passengers.rs. position_rider (entity/mod.rs:1648) invokes the hook once per tick via ride_tick_riders (world/mod.rs:2248-2252).
All four STILL OPEN rows checked against decompiled vanilla at
- Row 1 / h6 "Delta-rotation accumulation not added: STILL OPEN": VERIFIED ALREADY FIXED.
Unit allowlist is `dismount.rs` only. No row in this doc is marked STILL OPEN against that

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
