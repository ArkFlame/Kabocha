# Painting (entities) - re-review

Audit date: 2026-11-16. Scope: `crates/pumpkin-core/src/entity/decoration/painting.rs`,
`crates/pumpkin-core/src/item/items/hanging_entity.rs`, painting entry in
`crates/pumpkin-data/src/generated/entity_type.rs`. No decompiled vanilla source
available in this worktree (`.vendor/` absent), so class references are from the
vanilla/Paper call chain, not byte-diffed.

The facing rework (horizontal NBT value -> 3D data index) is correct and stays.
Two small parity gaps found and fixed on this branch:

- `damage_with_context` passed the damaged entity (`caller`) as the drop cause,
  so the creative check never matched a `Player` and creative hits dropped a
  painting item. Vanilla `HangingEntity#hurt` uses `dropItem(source.getEntity())`.
  Fixed: use `source`, and skip when already removed (vanilla `!isRemoved()`).
- `read_custom_nbt` indexed `block_pos[0..2]` directly, panicking on a short or
  malformed int array from a save file. Fixed with a length-checked conversion.

Remaining gaps (not fixed):

## 1. Painting hit box is 0.5 x 0.5 instead of variant sized (M)
- `crates/pumpkin-data/src/generated/entity_type.rs:7912` (`dimension: [0.5, 0.5]`)
  is the only box the entity ever gets; `painting.rs:66-96` and
  `painting.rs:259-283` never resize it.
- Vanilla: `Painting#setVariant`/`calculateBoundingBox` (via `HangingEntity`)
  builds an AABB from the variant width/height, thin along the facing axis.
- Effect: raycast/hit and client-side picking only cover the middle 0.5 block of
  a large painting; vanilla lets a hit anywhere on the artwork break it.
- Fix: add an `EntityBase` AABB override or a `set_dimensions` call that uses
  `variant.width()/height()` when the variant is set/loaded, oriented by facing
  (width runs along `facing.rotate_counter_clockwise()`, depth stays thin).
  Needs a per-entity AABB hook, so size M rather than S.

## 2. Painting yaw is never set from the facing (S)
- `crates/pumpkin-core/src/item/items/hanging_entity.rs:82-92` and
  `painting.rs:267-270` set the data index but leave yaw 0 for every painting.
- Vanilla `HangingEntity#setDirection` also calls `setYRot(direction.get2DDataValue() * 90)`
  (south 0, west 90, north 180, east 270), so spawn/save state carries it.
- Fix: set rotation (yaw, pitch 0) on placement and in `read_custom_nbt`, mapping
  the horizontal value with the same table as `facing_from_horizontal`.

## 3. Front-space check accepts any non-solid block (S, verify against 26.3 source)
- `painting.rs:172-175`: front cells must merely not be `is_solid()`.
- Vanilla `Painting#survives` requires the cells in front to be air/empty, so
  paintings cannot overlap torches, signs, water, etc.
- Fix: use `BlockState::is_air()` for the front cell once the vanilla check is
  confirmed for the current version.

## 4. Placement skips the build-limit check the item-frame branch has (S)
- `crates/pumpkin-core/src/item/items/hanging_entity.rs:55-99` (painting) never
  calls `world.is_in_build_limit(target_pos)`; the frame branch does at line 101.
- Vanilla rejects hanging entities outside the world build bounds (`mayUseItemAt`
  / world bounds), so a painting can currently be placed outside the build height.
- Fix: mirror the frame branch check before choosing the variant.

## 5. Variant choice uses thread RNG instead of the world RNG (S)
- `painting.rs:208-209` uses `rand::rng()`.
- Vanilla `Painting#create` draws from `level.getRandom()`, so placement is
  world-seeded and reproducible; the thread RNG is per-call and unseeded.
- Fix: take the world/level RNG (as other spawners do) and sample from it.

## 6. Support re-check is off by one tick (S)
- `painting.rs:295-298`: first support check happens at tick 101 and then every
  101 ticks because the counter is reset after the increment.
- Vanilla `HangingEntity#tick` checks on `tickCount % 100 == 0` (ticks 100, 200,
  ...). Fix: compare the counter modulo or reset before the increment.

## Gap status (h6 sweep)
- Row 2 yaw from facing: FIXED already in code (`set_facing` sets rotation, `read_custom_nbt` calls it). Verified by reading painting.rs:134, 308.
- Row 4 build-limit check: FIXED, painting branch of `hanging_entity.rs` now checks `is_in_build_limit`. UNVERIFIED (no cargo).
- Row 6 support re-check interval: FIXED, first check now on tick 100 (`painting.rs` tick). UNVERIFIED (no cargo).
- Rows 1, 3, 5: STILL OPEN.


## Result

Re-review on 2026-11-17 against decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/` (Paper/Folia 26.3 sources).

- Row 1 (hit box variant sized): FIXED in code and verified. `painting.rs`
  `hanging_bounding_box()`/`recalculate_bounding_box()` store a variant-sized AABB
  (width perpendicular to facing, height on Y, `DEPTH` along the facing axis) into
  `entity.bounding_box` on `set_variant`, `set_facing` and `read_custom_nbt`, which
  matches vanilla `Painting.calculateBoundingBoxStatic`. Extracted the pure
  `painting_box_size()` helper and added test `painting_box_size_matches_vanilla_axes`.
  The generated `entity_type.rs` `[0.5, 0.5]` dimension is intentionally left alone
  (generated file, runtime box overrides it).
- Row 2 (yaw from facing): already fixed by prior sweep; vanilla
  `HangingEntity.setDirection` (`get2DDataValue() * 90`) confirmed in `facing_to_yaw`.
- Row 3 (front-space check): LEFT OPEN - the doc's premise does not hold in 26.3.
  Vanilla `HangingEntity#survives` uses collision-shape `noBlockCollision` on the
  AABB (via `hasLevelCollision`), not an is-air cell test; torches/signs/water have
  no collision shape and are allowed. Pumpkin already enforces the real collision
  check in `survives()` through `World::is_space_empty(bounding_box)`, so switching
  the `painting_fits` front cell to `is_air()` would wrongly reject those blocks.
  True per-cell parity needs voxel-shape collision at filter time (cross-crate).
- Row 4 (build-limit check): already fixed by prior sweep; re-confirmed in
  `hanging_entity.rs` painting branch.
- Row 5 (variant RNG): LEFT OPEN - the doc's premise does not hold in 26.3.
  Both `Level.getRandom()` (`Level.random = ThreadLocalRandomSource.INSTANCE`) and
  `Entity.random` (`SHARED_RANDOM`) resolve to thread-local RNG in the bundled
  Paper/Folia source, so `rand::rng()` is functionally equivalent; there is no
  world-seeded RNG seam to adopt within this unit's files.

## Result (2026-11-18, re-verified against vendored decompiled vanilla Painting.java / HangingEntity.java / Entity.java)

- Row 1 (hit box variant sized): VERIFIED FIXED in this checkout. painting.rs painting_box_size()
  matches vanilla Painting.calculateBoundingBoxStatic: width along the axis perpendicular to the
  facing, height on Y, DEPTH (0.0625) along the facing axis. hanging_bounding_box()/recalculate_bounding_box()
  store the variant-sized AABB on set_variant, set_facing and read_custom_nbt, and test
  painting_box_size_matches_vanilla_axes covers it. Generated entity_type.rs [0.5, 0.5] left alone.
- Row 2 (yaw from facing): VERIFIED FIXED. HangingEntity.setDirection confirmed (get2DDataValue()*90);
  facing_to_yaw matches and set_facing applies rotation.
- Row 3 (front-space check): remains OPEN by design. Vanilla HangingEntity.survives() uses
  hasLevelCollision(getPopBox()) -> Level.noBlockCollision (collision shapes), a solid/diode support
  check, and canCoexist; there is no is-air cell test. Torches/signs/water have no collision and are
  allowed. Pumpkin already enforces the collision check in survives() via World::is_space_empty(), so
  switching painting_fits front cells to is_air() would wrongly reject those blocks. Per-cell voxel
  parity needs collision shape at filter time (cross-crate).
- Row 4 (build-limit check): VERIFIED FIXED. hanging_entity.rs painting branch checks is_in_build_limit.
- Row 5 (variant RNG): remains OPEN by design. Vanilla Painting.create uses candidate.random, and in
  the bundled Paper/Folia source Entity.SHARED_RANDOM and Level.random both resolve to
  ThreadLocalRandomSource.INSTANCE, so rand::rng() is functionally equivalent; no world-seeded seam.
- Row 6 (support re-check): VERIFIED FIXED. First check at tick 100 (SUPPORT_CHECK_INTERVAL = 100).
- No STILL OPEN row is actionable within this unit; no generated-data, WIT or cross-crate change made.



## Result (2026-11-18 worker re-verify, vendored vanilla available)

Both remaining STILL OPEN rows re-checked directly against decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/` (Paper/Folia 26.3). No code change was needed.

- Row 3 (front-space check): remains open by design, premise does not hold in this source.
  `HangingEntity.survives()` (HangingEntity.java:82-88) uses `hasLevelCollision(getPopBox())`,
  which is `!level.noBlockCollision(this, popBox)` (HangingEntity.java:108-110), i.e. a
  voxel collision-shape check on the thin painting AABB, not an is-air cell test.
  Support uses `state.isSolid() || DiodeBlock.isDiode(state)` (HangingEntity.java:91-92) and
  coexistence uses `canCoexist` (HangingEntity.java:99-105). Torches/signs/water have no
  collision shape and are allowed. `Painting.create()` filters candidates with the same
  `survives()`. Pumpkin already enforces the collision check in `survives()` via
  `World::is_space_empty`; switching `painting_fits` front cells to `is_air()` would wrongly
  reject those blocks. True per-cell parity needs collision shapes at filter time (cross-crate).
- Row 5 (variant RNG): remains open by design, premise does not hold in this source.
  `Painting.create()` samples with `Util.getRandomSafe(potentialVariants, candidate.random)`
  (Painting.java:112). `Entity.random` is `SHARED_RANDOM` (Entity.java:356), which in this
  bundled build is `ThreadLocalRandomSource.INSTANCE` (Entity.java:175). It is thread-local,
  not world-seeded, so Rust `rand::rng()` (also thread-local) is functionally equivalent;
  there is no world-seeded seam to adopt within this unit's files.
- Rows 1, 2, 4, 6 re-confirmed fixed in code (variant-sized box + `painting_box_size` test,
  yaw via `facing_to_yaw`/`set_facing`, `is_in_build_limit` check in the painting branch,
  support check on tick 100). No generated-data, WIT or cross-crate change made. No new test
  seam for rows 3/5.
