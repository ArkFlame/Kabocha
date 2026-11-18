# Final rows from docs/tracker/tentities-cherry-boat-ae864b.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: dismount.rs

STILL OPEN note (h6 delta-rotation) is already covered by the two prior Result sections and
- Row 1 / h6 "Delta-rotation accumulation not added: STILL OPEN": VERIFIED ALREADY FIXED.
- `boat/dismount.rs` (the only file in this unit's allowlist) has no STILL OPEN row and was verified
pass, no STILL OPEN row. Matches `.vendor/.../AbstractBoat.java:709-738`,

## Result

FINAL PASS 2026-11-18: no STILL OPEN rows. The only source file in this unit's allowlist,
`crates/pumpkin-core/src/entity/vehicle/boat/dismount.rs`, was verified line-by-line against
decompiled vanilla and matches:
- `AbstractBoat.getDismountLocationForPassenger` (AbstractBoat.java:709-738): escape vector from
  `getBbWidth()*SQRT_OF_TWO` + passenger yaw, target X/Z, `BlockPos.containing(..., maxY, ...)`
  (`floored_v(fallback+direction)`), `belowBlockPos` water gate (`get_fluid_and_fluid_state`
  + MINECRAFT_WATER tag), valid-floor filter (`!isInfinite && <1.0` vs `isBlockFloorValid`),
  pose-outer/target-inner loop, and the `super` fallback (`Vec3(x, maxY, z)` = `fallback`).
- `Entity.getCollisionHorizontalEscapeVector` (Entity.java:6645-6651): `(cw+cw2+1e-5)/2`,
  x=-sin, z=cos, scale=max(|x|,|z|), matches `get_collision_horizontal_escape_vector`.
- `DismountHelper.canDismountTo` (DismountHelper.java:42-53): bounds from rider pose dimensions
  moved to location, empty block collisions, worldborder containment match `can_dismount_to`.
- `dismount_poses` matches `Player.getDismountPoses` (Standing/Crouching/Swimming) vs
  `LivingEntity` (Standing); two pure unit tests cover pose sets and escape vector.

No code change needed; no BLOCKED rows (no missing generated pumpkin-data field or subsystem).
No .rs file edited, so rustfmt not applicable.
