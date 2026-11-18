# Parity rows for crates/pumpkin-core/src/entity/decoration/painting.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Painting** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.
- **own/tentities-painting** [partial/OWN-OPEN] own audit: 4 open, 9 fixed, 7 verified, 0 blocked-data (docs/tracker/tentities-painting.md) docs/tracker/tentities-painting.md

## Result
Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field. See terminal results below.


## Result (2026-11-18 THIRD PASS, all rows terminal)

Re-checked every row directly against the vendored decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/painting/Painting.java`
and `.../decoration/HangingEntity.java`. No generated-data or WIT change; only
`painting.rs` is in this unit and no source edit was needed this pass (all actionable
fixes from prior passes are present and verified in the current file).

- `entities/Painting [partial/AUDITED]`: CLOSED (VERIFIED / FIXED / one BLOCKED-DATA
  residual). Six sub-rows, all terminal:
  - Row 1 (variant-sized hit box): FIXED. `painting.rs` `painting_box_size()` gives
    width on the axis perpendicular to the facing, height on Y, DEPTH (0.0625) on the
    facing axis, matching `Painting.calculateBoundingBoxStatic`
    (`.../decoration/painting/Painting.java:158-175`). `hanging_bounding_box()` /
    `recalculate_bounding_box()` store it on `set_variant`, `set_facing` and
    `read_custom_nbt`. Test `painting_box_size_matches_vanilla_axes`. Generated
    `entity_type.rs` `[0.5,0.5]` intentionally untouched.
  - Row 2 (yaw from facing): FIXED. `facing_to_yaw()` = horizontal value * 90 and
    `set_facing()` applies it; matches `HangingEntity.setDirection` ->
    `setYRot(direction.get2DDataValue() * 90)` (`.../decoration/HangingEntity.java:60-67`).
    Tests `facing_to_yaw_matches_vanilla_direction_yaw`, `horizontal_and_3d_facings_round_trip`.
  - Row 3 (front-space / support check): actionable part FIXED, residual BLOCKED-DATA.
    The original is-air front-cell premise is disproven: vanilla `survives()`
    (`.../decoration/HangingEntity.java:82-110`) uses `hasLevelCollision(getPopBox())`
    = `!noBlockCollision || !noBorderCollision`, a voxel collision-shape test, plus
    `isSupportingBlock` = `state.isSolid() || DiodeBlock.isDiode(state)`
    (HangingEntity.java:91-93) and `canCoexist`. `painting_fits` now uses
    `is_supporting_block` (test `diode_blocks_support_hanging_entities`) and `survives()`
    enforces `is_space_empty` + `no_border_collision` + overlap, matching the
    collision/border/support/coexist structure. Residual gap: non-solid blocks that
    carry a collision shape (slabs, stairs, fences) are ignored because the shared
    `World::check_collision` short-circuits `!state.is_solid()`
    (`pumpkin-core/src/world/mod.rs:3210`); faithful `noBlockCollision` parity needs
    per-block voxel collision-shape evaluation in that shared routine, which governs
    all entity movement/collision/spawning (whole missing subsystem). BLOCKED-DATA:
    missing subsystem = collision-shape evaluation for non-solid blocks in
    `World::check_collision` / `World::is_space_empty`.
  - Row 4 (build-limit check): FIXED. `hanging_entity.rs` painting branch calls
    `world.is_in_build_limit(target_pos)` (line 58) before `choose_variant` (line 72),
    mirroring the frame branch (line 106).
  - Row 5 (variant RNG): VERIFIED, no code change. `Painting.create` samples with
    `Util.getRandomSafe(potentialVariants, candidate.random)` (Painting.java:112);
    `Entity.random` is thread-local `SHARED_RANDOM`, so Rust `rand::rng()` (also
    thread-local) is functionally equivalent; no world-seeded seam exists in this unit.
  - Row 6 (support re-check interval): FIXED. `SUPPORT_CHECK_INTERVAL = 100` and the
    `fetch_add(1)+1 < 100` guard fires on ticks 100, 200, ..., matching vanilla
    `BlockAttachedEntity.CHECK_INTERVAL`.
- `own/tentities-painting [partial/OWN-OPEN]`: CLOSED by the above six terminal rows
  (docs/tracker/tentities-painting.md). No STILL OPEN row remains.

No source edit this pass; no shared file rewritten. UNVERIFIED by worker build
(no cargo by worker).


## Result (2026-11-18 FINAL PASS, all rows terminal)

Re-read `painting.rs`, `entity/decoration/hanging.rs` and
`item/items/hanging_entity.rs` in full and compared each row against the
vendored decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/painting/Painting.java`,
`.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/HangingEntity.java`,
`.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/BlockAttachedEntity.java`
and `.vendor/vanilla-decompiled/net/minecraft/world/item/HangingEntityItem.java`.
All six rows are terminal. No source edit was needed this pass: the actionable
fixes from earlier passes are present and match vanilla. No generated-data,
WIT or cross-crate change.

- `entities/Painting` and `own/tentities-painting`, Row 1 (variant-sized hit box):
  FIXED. `painting.rs` `painting_box_size()` sets width on the axis perpendicular
  to facing, height on Y and `DEPTH` (0.0625) on the facing axis, matching
  `Painting.calculateBoundingBoxStatic` (Painting.java:158-175).
  `calculate_center_pos()` matches the wall offset and size offsets
  (Painting.java:160-174, HangingEntityItem.java:36-48). Test
  `painting_box_size_matches_vanilla_axes`.
- Row 2 (yaw from facing): FIXED. `facing_to_yaw()` = horizontal value * 90 and
  `set_facing()` applies it; matches `HangingEntity.setDirection` calling
  `setYRot(direction.get2DDataValue() * 90)` (HangingEntity.java:60-67).
  Tests `facing_to_yaw_matches_vanilla_direction_yaw` and
  `horizontal_and_3d_facings_round_trip`.
- Row 3 (front-space / support check): actionable part FIXED, residual
  BLOCKED-DATA. Vanilla `survives()` uses `hasLevelCollision(popBox)` plus
  `isSupportingBlock` (`isSolid || DiodeBlock.isDiode`) plus `canCoexist`
  (HangingEntity.java:82-110). `painting_fits` applies `is_supporting_block`
  (test `diode_blocks_support_hanging_entities`) and the front-cell solid check;
  `survives()` also enforces `is_space_empty` plus `no_border_collision` plus
  overlap. Residual gap: non-solid blocks that carry a voxel collision shape
  (slabs, stairs, fences) are ignored because `World::check_collision`
  short-circuits on `!state.is_solid()`. Faithful `noBlockCollision` parity needs
  per-block voxel collision-shape evaluation in that shared routine, which
  governs all entity movement and spawning. BLOCKED-DATA, missing subsystem =
  collision-shape evaluation for non-solid blocks in `World::check_collision`
  / `World::is_space_empty`.
- Row 4 (build-limit check): FIXED. `item/items/hanging_entity.rs` painting
  branch calls `world.is_in_build_limit(target_pos)` (line 58) before
  `choose_variant` (line 72), mirroring the frame branch (line 106) and the
  build-limit gate behind `player.mayUseItemAt` in `HangingEntityItem.useOn`
  (HangingEntityItem.java:41).
- Row 5 (variant RNG): VERIFIED, no code change. `Painting.create` samples with
  `Util.getRandomSafe(potentialVariants, candidate.random)` (Painting.java:112);
  Rust `rand::rng()` is thread-local like the entity RandomSource, functionally
  equivalent. No entity-seeded seam in this unit.
- Row 6 (support re-check interval): FIXED. `SUPPORT_CHECK_INTERVAL = 100` and
  the `fetch_add(1)+1 < 100` guard fires the first check on tick 100, matching
  `BlockAttachedEntity.CHECK_INTERVAL` (BlockAttachedEntity.java:26,46).

No STILL OPEN row remains. UNVERIFIED by worker build (no cargo by worker).
