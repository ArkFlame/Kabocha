# Final rows from docs/tracker/tentities-cherry-boat.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

The stale "STILL OPEN" marker on the h6 delta-rotation note was removed: it had already been
Fresh audit of the current Pumpkin source. No row is marked STILL OPEN (the only textual
"STILL OPEN" hits are historical mentions inside earlier Result sections; the h6 note on
Every row ends FIXED, VERIFIED or NOT A GAP; none is STILL OPEN and none is BLOCKED-DATA (no missing

## Result
FINAL PASS close-out. All five rows re-verified against the real decompiled vanilla tree present
in this checkout (hidden .vendor path) plus the current Pumpkin source. No source file needed
editing; every row was already correct. Vanilla paths cited below are confirmed present.

1. Rider clamp + delta rotation: FIXED / VERIFIED.
   .vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java
   positionRider (696-706) adds deltaRotation then calls clampRotation (740-754); onPassengerTurned
   (762-766) folds yDelta into yRotO; calculatePassengerBodyYRot (756-760) puts a full-boat animal
   at +90/+270 by id parity. Pumpkin boat.rs: yaw_delta = wrap_degrees(yaw - last_yaw) (591-593);
   on_rider_positioned (856-899) applies it before clamp; on_passenger_turned (901-949) re-clamps
   and folds rider_clamp_adjustment into y_rot_o. Helpers clamp_rider_yaw (1009, +/-105),
   clamp_head_to_body (998, Mob.clampHeadRotationToBody), rider_body_yaw (982, full-boat animal
   +90/+270) and rider_clamp_adjustment (1018) match. Tests at 1045/1055/1065 cover the seam.

2. Buoyancy / drag / friction: FIXED / VERIFIED.
   .vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java
   floatBoat (607-648): gravity, IN_AIR snap, per-status friction 0.9/0.45/landFriction, buoyancy
   0.75 factor. Pumpkin boat/physics.rs: float_boat (184-239), get_water_level_above (113-139),
   called from boat.rs:625 for every server-authoritative boat; player-steered boats keep
   client-authoritative motion by design (AGENTS.md).

3. Boarding gate (ticks_underwater >= 60): VERIFIED (no change needed; premise was wrong).
   .vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java
   interact (779-787) returns PASS when player.isSecondaryUseActive() || !(outOfControlTicks < 60.0F)
   || ... . Pumpkin boat_interact_outcome (boat.rs:161-177) keeps the !out_of_control term and
   returns Nothing when it fails, with a test at 1267. Matches vanilla exactly.

4. Chest-boat passenger cap: VERIFIED (no change needed; matches vanilla).
   .vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractChestBoat.java
   getMaxPassengers() returns 1 (48-50) vs AbstractBoat 2 (AbstractBoat.java:840-842);
   canAddPassenger (AbstractBoat.java:836-838); interact (80-97) falls back to the container only
   when !canAddPassenger or isSecondaryUseActive. Pumpkin max_passengers (boat.rs:95-101) returns 1
   for chest boats; chest_opens_container (135-141) gates the container on sneak/no-free-seat.

5. Chest contents on every destroy route: FIXED / VERIFIED.
   .vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractChestBoat.java
   remove (71-77) drops the 27 slots when reason.shouldDestroy(); destroy (65-68). Pumpkin
   scatter_chest_contents (boat.rs:272-288) runs from on_removed (488-506) on a destroying removal
   and from the damage path (746), guarded by claim_drops so it fires once. is_collidable (643-645)
   returns true unconditionally, matching AbstractBoat.canBeCollidedWith (AbstractBoat.java:129-131).

Every row ends FIXED or VERIFIED; none is STILL OPEN and none is BLOCKED-DATA (no missing generated
pumpkin-data field or subsystem). No source file changed, so no rustfmt/build was required (cargo and
shell are forbidden for this worker). Status: PASS (verified against real decompiled vanilla source,
not compiled).
