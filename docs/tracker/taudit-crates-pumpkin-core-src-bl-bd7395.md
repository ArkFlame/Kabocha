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
