# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-entity-decoration-cushion-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files:

Final pass (2026-11-18). Independently re-read the current `crates/pumpkin-core/src/entity/decoration/cushion.rs`, both sub-audits (`tentities-cushion.md`, `tentities-cushion-2527f1.md`) and the decompiled vanilla classes (`Cushion.java`, `BlockAttachedEntity.java`, `Entity.java`). Every fixable gap is already implemented at HEAD; no `.rs` file needed editing this pass. Per row:

## Result

- **entities/Cushion** [AUDITED]: VERIFIED. `cushion.rs` matches vanilla `.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/Cushion.java` and `BlockAttachedEntity.java`: placement/support (`can_be_placed_at`/`would_survive_at`/`has_anchor_below`/`is_anchor_buried`/`is_covered_by_suffocating_blocks`, Cushion.java:173-214), tracked colour and spawn metadata (Cushion.java:43-55), `drop_and_remove` (Cushion.java:58-69: break sound before the `entity_drops` gate, creative-attacker no-drop, custom-name copy from Cushion.java:264-268), damage guards (`damage_with_context`: invulnerable / mobGriefing / mayBuild / mayInteract, BlockAttachedEntity.java:129-171 + Cushion.java:71-82), removed-hit returns handled (BlockAttachedEntity.java:162-164), cancelled-dismount still breaks (BlockAttachedEntity.java:166-168), dye clamp on NBT read (`DyeColor::by_id_or_white`, Cushion.java:238-241) and seat via `position_rider` (Cushion.java:90-99, Entity.java:3540-3544).

- **own/tentities-cushion-2527f1** [OWN-OPEN]: all five sub-rows closed.
  - Mount/dismount game events: VERIFIED (NOT A GAP). `GameEvent::EntityMount` at `entity/mod.rs:6261` (`add_passenger`) and `GameEvent::EntityDismount` at `:6516` (`remove_passenger_internal`), emitted at the vehicle position, matching vanilla `Entity.startRiding`/`stopRiding` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/Entity.java:3625,3672`).
  - `interact` seat (`position_rider`): VERIFIED (FIXED at HEAD, cushion.rs:411). Vanilla `Entity.positionRider` (Entity.java:3540-3544).
  - `drop_and_remove`: VERIFIED (FIXED at HEAD, cushion.rs:177-218). Creative/drops gate, sound order, cancelled dismount. Vanilla Cushion.java:58-69, BlockAttachedEntity.java:166-168.
  - `damage_with_context`: VERIFIED (FIXED at HEAD, cushion.rs:432-472). Invulnerable/mobGriefing/mayBuild/mayInteract, removed-hit handled. Vanilla BlockAttachedEntity.java:129-171, Cushion.java:71-82.
  - Placement/support `pumpkin-gametest` cases: BLOCKED-DATA. Behaviour VERIFIED against BlockAttachedEntity.java:42-78 and Cushion.java:216-219 (`survives()` == `wouldSurviveAt`; fire on tick 100 when `!isRemoved() && !survives()`). Pure unit seams already cover anchor required, slab anchor, suffocating-cover rejection, buried-anchor subtraction and the occupied-cushion drop. The remaining case (100-tick post-support-removal item drop) needs the running-server gametest harness (`builtin_game_tests` / `server_test_manager`): `tick()` requires a full `&Server` with no test-only constructor, a whole missing subsystem, so no pure unit seam exists.

- **own/tentities-cushion** [OWN-OPEN]: all five sub-rows closed with the same outcomes as above. Drop gating, colour clamp, mount/dismount events, seat and removed-hit are VERIFIED/FIXED at HEAD; the post-support-removal item-drop gametest is BLOCKED-DATA (needs the `pumpkin-gametest` running-server harness / `builtin_game_tests` / `server_test_manager`).

No `.rs` file was edited this pass (all fixable gaps are already present at HEAD), so no rustfmt run was needed. Re-read the diff against the worker compile-pitfall table: no new imports, helpers, fields, clones or panics, and no source file touched.
