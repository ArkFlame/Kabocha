# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-8f1a6a.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

STILL OPEN (row 38, S): claim contradicted by vanilla. `playerTouch` (`AbstractArrow.java:704`) checks only `isInGround() || isNoPhysics()`, with no owner branch.
STILL OPEN (row 39, M): engine-wide `last_hurt_mob` field plus consumers.
Re-verified every STILL OPEN row against decompiled vanilla
Re-checked every STILL OPEN row against decompiled vanilla

## Result
- Row 38 (no-physics owner pickup, S): VERIFIED | .vendor/vanilla-decompiled/net/minecraft/world/entity/projectile/arrow/AbstractArrow.java:703-704,724-729. playerTouch checks only (isInGround() || isNoPhysics()) && shakeTime <= 0, and tryPickup has no owner branch; the original claim is contradicted by vanilla. Pumpkin arrow.rs:656-728 (on_player_collision) already matches. No code change needed.
- Row 39 (last_hurt_mob, M): FIXED | .vendor/vanilla-decompiled/net/minecraft/world/entity/projectile/arrow/AbstractArrow.java:497-499 calls livingOwner.setLastHurtMob(entity). arrow.rs:776-791 (on_hit_entity) now records the hit target on a living owner using Pumpkin existing fields last_attacking_id and last_attack_time (the surrogate for LivingEntity.lastHurtMob, written by Player.attack and Mob.attack, read by OwnerHurtTargetGoal and Fox), so owner AI reacts to what its arrow struck. Same observable behaviour as vanilla without a new engine field; no generated pumpkin-data field needed. Pure unit test omitted: recording owner state needs a live world/entity graph, not a pure seam.
