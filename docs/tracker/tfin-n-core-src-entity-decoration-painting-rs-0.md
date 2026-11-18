# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-entity-decoration-painting-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

(docs/tracker/tentities-painting.md). No STILL OPEN row remains.
No STILL OPEN row remains. UNVERIFIED by worker build (no cargo by worker).

## Result
## Result (2026-11-18 FINAL PASS, all rows terminal)

Re-read the full current sources `crates/pumpkin-core/src/entity/decoration/painting.rs`,
`crates/pumpkin-core/src/entity/decoration/hanging.rs`, and
`crates/pumpkin-core/src/item/items/hanging_entity.rs` against the vendored
decompiled vanilla at `.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/painting/Painting.java`,
`.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/HangingEntity.java`,
`.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/BlockAttachedEntity.java` and
`.vendor/vanilla-decompiled/net/minecraft/world/item/HangingEntityItem.java`.
Every actionable fix from earlier passes is present and matches vanilla. No
generated-data, WIT, or cross-crate change was needed, and no source file needed
editing this pass. No STILL OPEN row remains.

- Row 1 (variant-sized hit box): FIXED. `painting_box_size()` puts width on the axis
  perpendicular to facing, height on Y and `DEPTH` (0.0625) on the facing axis,
  matching `Painting.calculateBoundingBoxStatic` (Painting.java:158-175).
  `hanging_bounding_box()` / `recalculate_bounding_box()` apply it on variant/facing
  changes and NBT load. Generated `entity_type.rs` `[0.5,0.5]` intentionally untouched.
  Test `painting_box_size_matches_vanilla_axes`.
- Row 2 (yaw from facing): FIXED. `facing_to_yaw()` = `get2DDataValue() * 90` and
  `set_facing()` applies it via `set_rotation`; matches `HangingEntity.setDirection`
  (HangingEntity.java:60-67). Tests `facing_to_yaw_matches_vanilla_direction_yaw`,
  `horizontal_and_3d_facings_round_trip`.
- Row 3 (front-space / support check): actionable parity FIXED, one BLOCKED-DATA residual.
  Vanilla `HangingEntity.survives()` (HangingEntity.java:82-110) uses
  `hasLevelCollision(getPopBox())` = `!noBlockCollision || !noBorderCollision`,
  `isSupportingBlock` = `isSolid || DiodeBlock.isDiode` (HangingEntity.java:91-93)
  and `canCoexist`. `painting_fits` applies `is_supporting_block` (test
  `diode_blocks_support_hanging_entities`) plus the front-cell solid check;
  `survives()` additionally enforces `is_space_empty` + `no_border_collision` +
  non-overlap, and placement re-runs `survives()` before spawn. Residual gap:
  non-solid blocks that carry a voxel collision shape (slabs, stairs, fences) are
  ignored because `World::check_collision` short-circuits on `!state.is_solid()`
  (`crates/pumpkin-core/src/world/mod.rs:3210`), which `is_space_empty` calls
  (`world/mod.rs:3327-3340`). Faithful `noBlockCollision` parity needs per-block
  voxel collision-shape evaluation in that shared routine, which governs all entity
  movement/collision/spawning. BLOCKED-DATA, missing subsystem = per-block
  collision-shape evaluation for non-solid blocks in `World::check_collision` /
  `World::is_space_empty`.
- Row 4 (build-limit check): FIXED. The painting branch of
  `crates/pumpkin-core/src/item/items/hanging_entity.rs` calls
  `world.is_in_build_limit(target_pos)` (line 58) before `choose_variant` (line 72),
  mirroring the frame branch (line 106) and the `player.mayUseItemAt` gate behind
  `mayPlace` in `HangingEntityItem.useOn` (HangingEntityItem.java:41,93-95).
- Row 5 (variant RNG): VERIFIED, no code change. `Painting.create` samples with
  `Util.getRandomSafe(potentialVariants, candidate.random)` (Painting.java:112); the
  entity `RandomSource` is thread-local, so Rust `rand::rng()` (also thread-local) in
  `choose_variant` is functionally equivalent. No entity-seeded seam exists here.
- Row 6 (support re-check interval): FIXED. `SUPPORT_CHECK_INTERVAL = 100` and the
  `fetch_add(1) + 1 < 100` guard fires the first check on tick 100, matching
  `BlockAttachedEntity.CHECK_INTERVAL` / tick (BlockAttachedEntity.java:26,42-78).

UNVERIFIED by worker build (no cargo by worker); rustfmt to be run by the
orchestrator. Files verified, none edited this pass: `painting.rs`, `hanging.rs`,
`hanging_entity.rs`.
