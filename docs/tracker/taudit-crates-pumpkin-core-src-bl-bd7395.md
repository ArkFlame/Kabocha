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
- 1. `get_updated_state` recomputes all faces per neighbour update (`vine.rs:69-112`): STILL OPEN (S).
- 2. Dead fallback branch (`vine.rs:80-106`): STILL OPEN (XS).
- 3. `is_air()` on default state (`vine.rs:377,421,431,486,521-522`): STILL OPEN (XS).
- 4. Placement direction order (`vine.rs:195-225,237-246`): STILL OPEN (S, unverified).
- 5. `can_place_at` duplicates placement loop (`vine.rs:257-319`): STILL OPEN (M).


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
