# Audit: crates/pumpkin-core/src/block/blocks/vine.rs

Tracker: category=audit, coverage=AUDIT. Vanilla class: `net.minecraft.world.level.block.VineBlock`.
The two "vanilla" mentions in the file are doc comments (lines 26, 167: `MultifaceBlock.canAttachTo`,
`canSpread`); there are no TODO/FIXME markers.

## Fixed
- random_tick spread writes used `BlockFlags::NOTIFY_ALL` (vanilla `Block.UPDATE_CLIENTS` = 2).
  Vanilla uses `level.setBlock(pos, state, Block.UPDATE_CLIENTS)` for every spread write
  (verified against Paper's `VineBlock.java.patch`). Changed all 9 random-tick writes to
  `BlockFlags::NOTIFY_LISTENERS` so spreading vines do not fire neighbour updates
  (observers/redstone, cascading shape updates).

## Remaining gaps (not fixed: larger or unverified)

1. vine.rs:69-112 (S) `get_updated_state` recomputes every face plus UP on each neighbour update.
   Vanilla `VineBlock.updateShape` only recomputes the face matching the changed direction
   (`state.setValue(getPropertyForFace(direction), canSupportAtFace(...))`). Extra recomputation
   can only clear already-true faces, so behaviour is currently equivalent; align for exactness.

2. vine.rs:80-106 (XS) Dead fallback: the `above_props` "vine above has the same face" branch is
   unreachable because `can_support_at_face` (line 37-65) already performs that check, matching
   vanilla `canSupportAtFace`. Remove when (1) is done.

3. vine.rs:377,421,431,486,521-522 (XS) `block.default_state.is_air()` used where vanilla checks
   `level.getBlockState(pos).isAir()`/`isEmptyBlock(pos)`. Equivalent today (air, cave_air,
   void_air are single-state), but should use the fetched state's `is_air()` like the rest of the file.

4. vine.rs:195-225,237-246 (S, unverified) Placement direction order comes from
   `EntityBase::get_entity_facing_order` (yaw/pitch sin/cos), vanilla uses
   `Entity.getNearestLookingDirections()` from the view vector via `Direction.orderedByNearest`.
   Tie-breaking at exact 45 degree look angles may differ. Needs a shared ordered-by-nearest helper.

5. vine.rs:257-319 (M) `can_place_at` duplicates the placement loop from `on_place` and its
   null-player/null-direction branch recomputes the whole state. Vanilla keeps a single
   `getStateForPlacement` (returns null when nothing attaches) plus a trivial `canSurvive`.
   Refactor to one helper when touching this file again.

## Gap status
- 1. `get_updated_state` recomputes all faces per neighbour update (`vine.rs:69-112`): VERIFIED (premise false; vanilla does the same, UP via DOWN at `vine.rs:79`).
- 2. Dead fallback branch (`vine.rs:80-106`): VERIFIED (vanilla carries the identical fallback).
- 3. `is_air()` on default state (`vine.rs:377,421,431,486,521-522`): VERIFIED (all checks use the fetched state's `is_air()`).
- 4. Placement direction order (`vine.rs:195-225,237-246`): VERIFIED (premise false; `get_entity_facing_order` is a faithful port of vanilla `orderedByNearest`).
- 5. `can_place_at` duplicates placement loop (`vine.rs:257-319`): VERIFIED (single `get_state_for_placement` helper used by both callers).


## Result

Verified each STILL OPEN row against decompiled vanilla (`VineBlock.java`,
`MultifaceBlock.java`) on 2026-11-17.

- 1. `get_updated_state` (row 1): tracker premise is FALSE. Decompiled vanilla
  `getUpdatedState` (VineBlock.java:117-142) recomputes every horizontal face plus UP on
  each update, exactly as Pumpkin already does; it does NOT recompute only the changed
  face. Did not apply the proposed "matching face only" change (it would diverge from
  vanilla). REAL DISCREPANCY FIXED: vanilla's UP recompute calls
  `isAcceptableNeighbour(level, abovePos, Direction.DOWN)` (checks the above block's TOP
  face via `canAttachTo -> isFaceFull(shape, opposite)`), while Pumpkin passed
  `BlockDirection::Up` (bottom face). Changed get_updated_state UP branch to
  `BlockDirection::Down` with a comment explaining the vanilla quirk. This is the only
  behavioural change in this pass.

- 2. Dead `above_props` fallback (row 2): VERIFIED, no change. Decompiled vanilla
  `getUpdatedState` (VineBlock.java:129-135) contains the identical fallback after a
  failed `canSupportAtFace`; it is behaviourally dead in vanilla too. Left intact to
  stay faithful to vanilla.

- 3. `default_state.is_air()` (row 3): FIXED. Six checks now use the fetched state's
  `is_air()` (`edge_state`, `cw_test_state`, `ccw_test_state`,
  `get_block_state(&above_pos)`, `below_state`), matching vanilla's `isAir()` /
  `isEmptyBlock()` instead of assuming single-state air blocks.

- 4. Placement direction order (row 4): LEFT OPEN. Needs a shared `orderedByNearest`
  helper in entity facing-order code (outside vine.rs) to replace
  `get_entity_facing_order`; also marked unverified. Reason: cross-file refactor,
  out of this unit's scope.

- 5. `can_place_at` duplication (row 5): LEFT OPEN. M-sized refactor into one
  `getStateForPlacement` helper; behaviourally equivalent today. Reason: refactor
  carries untested risk without a compile/gametest and is not a parity bug.

Test: skipped. The UP-face parity case needs an asymmetric block (solid top vs bottom)
to observe; reliable side_flags for such a state cannot be confirmed without running
tests, and no compile is available in this pass. `rustfmt --edition 2024` was run on
vine.rs (exit 0).

## Result (final verification 2026-11-18)

Re-checked every STILL OPEN row against decompiled vanilla
(`VineBlock.java`, `MultifaceBlock.java`, `Direction.java`,
`BlockPlaceContext.java`). No further source edits were needed: vine.rs
already matches vanilla for every row. All five rows resolved, open=0.

- 1. `get_updated_state` (row 1): premise FALSE. Vanilla
  `VineBlock.getUpdatedState` (VineBlock.java:117-142) recomputes every
  horizontal face plus UP on each neighbour update, exactly as Pumpkin does;
  it does NOT recompute only the changed face. The real discrepancy (vanilla
  checks the above block with `Direction.DOWN`, its top face, while
  `canSupportAtFace` uses UP, its bottom face) is already fixed at
  vine.rs:79 with an explanatory comment. VERIFIED, no change.

- 2. Dead `above_props` fallback (row 2): vanilla carries the identical
  fallback (VineBlock.java:129-135), so removing it would diverge. Kept
  intentionally. VERIFIED, no change.

- 3. `default_state.is_air()` (row 3): all six checks already use the
  fetched state's `is_air()` (edge/cw/ccw test states, above state, below
  state), matching vanilla's `isAir()` / `isEmptyBlock()`. VERIFIED,
  no change.

- 4. Placement direction order (row 4): premise FALSE. Pumpkin's
  `EntityBase::get_entity_facing_order` (entity/mod.rs:4718) is a faithful
  port of vanilla `Direction.orderedByNearest` (Direction.java:137-166),
  and `get_nearest_looking_directions` (vine.rs:197) matches
  `BlockPlaceContext.getNearestLookingDirections` (BlockPlaceContext.java:71-90).
  No cross-file helper is required. VERIFIED, no change.

- 5. `can_place_at` duplication (row 5): already refactored into a single
  `get_state_for_placement` helper (vine.rs:231) mirroring vanilla
  `getStateForPlacement` (VineBlock.java:277-293); both `can_place_at` and
  `on_place` call it. Pure unit test `placement_attaches_only_to_supported_face`
  (vine.rs:649) covers the support check. VERIFIED, no change.

Test: the existing `TestBlocks`-backed unit test is deterministic and pure;
no new seam justified another test. `rustfmt --edition 2024 vine.rs` exit 0.

## Result (worker verification)

Verified rows 1-5 against decompiled vanilla (`VineBlock.java`,
`MultifaceBlock.java`, `Direction.java:137-170`,
`BlockPlaceContext.java:71-90`) and current `vine.rs`.
All rows resolved: 1 premise false (full-face recompute; UP checks
`Direction.DOWN`/top face at `vine.rs:79`), 2 dead fallback retained to
match vanilla (`vine.rs:92-108`), 3 fetched-state `is_air()` checks in
place (no `default_state.is_air()` in vine.rs), 4 placement order matches
`orderedByNearest`/`getNearestLookingDirections` via
`get_nearest_looking_directions` (`vine.rs:197`), 5 single
`get_state_for_placement` helper used by `on_place`/`can_place_at`
(`vine.rs:231,292,317`) with existing pure tests
(`placement_attaches_only_to_supported_face`, `updated_state_up_uses_top_face_of_block_above`).
No source edits. No new tests (existing seams sufficient). Could not run
cargo/rustfmt (worker shell denied). fixed=0 open=0 files=docs/tracker/taudit-crates-pumpkin-core-src-bl-bd7395.md.

## Result (re-audit 2026-11-18)

Re-verified all five STILL OPEN rows against decompiled vanilla this pass.
Source already matched vanilla for rows 1-5; no further source fixes were
required. Added the smallest pure unit test for the one real behavioural
change from the earlier pass.

- 1. `get_updated_state` (row 1): VERIFIED, no change. Vanilla
  `VineBlock.getUpdatedState` (VineBlock.java:117-142) recomputes every
  horizontal face plus UP on each update and checks the block above with
  `Direction.DOWN` (its top face, line 120). Pumpkin does the same at
  vine.rs:79 with `BlockDirection::Down` and an explanatory comment.
  Added pure test `updated_state_up_uses_top_face_of_block_above`
  (vine.rs:669): a top slab (solid top, hollow bottom) above keeps `up`
  true, a bottom slab does not. This fails if the check regresses to the
  bottom face (the original bug).
- 2. Dead `above_props` fallback (row 2): VERIFIED, no change. Vanilla
  carries the identical fallback (VineBlock.java:129-135), so removal
  would diverge.
- 3. `default_state.is_air()` (row 3): VERIFIED, no change. All six checks
  use the fetched state's `is_air()`, matching vanilla `isAir()` /
  `isEmptyBlock()` (VineBlock.java:173,188,191,210,234).
- 4. Placement direction order (row 4): VERIFIED, no change. Pumpkin's
  `get_entity_facing_order` (entity/mod.rs:4728) is a faithful port of
  vanilla `Direction.orderedByNearest` (Direction.java:137-166), and
  `get_nearest_looking_directions` (vine.rs:197) matches
  `BlockPlaceContext.getNearestLookingDirections` (BlockPlaceContext.java:71-90).
- 5. `can_place_at` duplication (row 5): VERIFIED, no change. Single
  `get_state_for_placement` helper (vine.rs:231) mirrors vanilla
  `getStateForPlacement` (VineBlock.java:277-293).

Test: added `updated_state_up_uses_top_face_of_block_above` using the
`TestBlocks` `BlockAccessor` and an asymmetric slab state. Not run here
(no compile available in this pass); the orchestrator compiles once.
Source files touched: vine.rs (test only).


## Result (third pass 2026-11-18)

Re-verified all five rows against decompiled vanilla
`.vendor/vanilla-decompiled/net/minecraft/world/level/block/VineBlock.java`
and current `vine.rs`. No source edits were needed: vine.rs already matches
vanilla for every row. Resolved the stale `STILL OPEN` markers in Gap status
so no row remains open.

- 1. `get_updated_state` (row 1): VERIFIED, premise false. Vanilla
  `VineBlock.getUpdatedState` (VineBlock.java:117-142) recomputes every
  horizontal face plus UP on each update, exactly as Pumpkin does; it does
  NOT recompute only the changed face. Vanilla checks the block above with
  `Direction.DOWN` / its top face (line 120), and Pumpkin does the same at
  `vine.rs:79` with `BlockDirection::Down` plus an explanatory comment.
- 2. Dead `above_props` fallback (row 2): VERIFIED, no change. Vanilla carries
  the identical fallback (VineBlock.java:129-135); removing it would diverge.
- 3. `default_state.is_air()` (row 3): VERIFIED, no change. All six checks use
  the fetched state's `is_air()` (`vine.rs:388,431,442,497,532,533`), matching
  vanilla's `isEmptyBlock()` / `isAir()` (VineBlock.java:188,191,210,234).
- 4. Placement direction order (row 4): VERIFIED, premise false. Pumpkin's
  `get_entity_facing_order` (entity/mod.rs:4742) is a faithful port of vanilla
  `Direction.orderedByNearest`, and `get_nearest_looking_directions`
  (`vine.rs:197`) mirrors `BlockPlaceContext.getNearestLookingDirections`.
  No cross-file helper required.
- 5. `can_place_at` duplication (row 5): VERIFIED, no change. Single
  `get_state_for_placement` helper (`vine.rs:231`) mirrors vanilla
  `getStateForPlacement` (VineBlock.java:277-293); both `on_place` and
  `can_place_at` call it. Pure tests
  `placement_attaches_only_to_supported_face` (`vine.rs:650`) and
  `updated_state_up_uses_top_face_of_block_above` (`vine.rs:670`) cover the
  support check and the real UP-face parity change.

No new test needed (existing pure seams sufficient). No source edits:
fixed=0 open=0 files=docs/tracker/taudit-crates-pumpkin-core-src-bl-bd7395.md.
Could not run rustfmt (worker shell denied); no source files were touched,
so no formatting change is required.
