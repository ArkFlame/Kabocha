# Parity rows for crates/pumpkin-core/src/block/blocks/piston/piston_head.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **blocks/Piston Head** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.

## Result
- blocks/Piston Head [partial/AUDITED] -> VERIFIED, no code change needed. Re-audited every server-side method against the decompiled vanilla at .vendor/vanilla-decompiled/net/minecraft/world/level/block/piston/PistonHeadBlock.java (plus PistonBaseBlock.java for the head call chain in piston.rs). All implementable behaviour matches:
  - is_fitting_base (:19-34) = isFittingBase (:57-60): TYPE Normal/Sticky maps to PISTON/STICKY_PISTON base, EXTENDED true, same FACING.
  - on_state_replaced (:38-54) = playerWillDestroy (:63-71) + affectNeighborsAfterRemoval (:75-79): removes the base only when it fits; forwarded SKIP_DROPS gives the creative/silk-touch dropless path, survival drops the base once. moved is deliberately ignored, matching the vanilla movedByPiston-ignoring affectNeighborsAfterRemoval; safe because piston.rs replaces the base with MOVING_PISTON (lines 252-256) before clearing the head in remove_arm_if_head (326-332), so a retract never drops a piston item.
  - get_state_for_neighbor_update (:56-78) = updateShape (:83-96): only the neighbour opposite FACING can invalidate the head, via can_place_at.
  - can_place_at (:80-89) = canSurvive (:99-102): fitting base or MOVING_PISTON with the same FACING.
  - on_neighbor_update (:91-114) = neighborChanged (:105-115): survives, then forwards the update to the base position.
  - is_pathfindable (:116-118) = isPathfindable (:137-140): false.
  - rotate/mirror/state definition come from generated block data; light occlusion is handled in pumpkin-world/src/lighting/occlusion.rs (useShapeForLightOcclusion, :48-50).
  - Vanilla SHORT is read only by the client getShape model (:53-54); no server behaviour depends on it, so not writing it is correct.
- Known gaps requiring generated data / new hooks (out of scope for this file; not left as STILL OPEN): (1) per-state collision AABB for the plate/stem needs asset-extractor generated data plus a get_shape hook (Pumpkin stores only side/centre-solidity masks); (2) vanilla getCloneItemStack (:117-120) has no BlockBehaviour clone-item hook, so pick-block on a head cannot return a piston/sticky-piston item without a cross-crate trait + dispatch change. Neither affects server-side survival/creative behaviour.
- No source file changed, so no rustfmt needed. No unit test added: no pure seam exists (is_fitting_base needs generated state IDs and a test would only mirror the formula).


## Result (final pass, 2026-11-18)

Independent terminal re-audit of the single row `blocks/Piston Head` against the
decompiled vanilla
`.vendor/vanilla-decompiled/net/minecraft/world/level/block/piston/PistonHeadBlock.java`
and the head call chain in
`.vendor/vanilla-decompiled/net/minecraft/world/level/block/piston/PistonBaseBlock.java`,
plus the current `crates/pumpkin-core/src/block/blocks/piston/piston_head.rs`,
`.../piston/piston.rs` and `crates/pumpkin-world/src/lighting/occlusion.rs`.

- **blocks/Piston Head [partial/AUDITED]: VERIFIED.** No source change needed this
  pass; the line references in the earlier Result still match the current file
  exactly. Every server-side method matches vanilla:
  - `is_fitting_base` (:19-34) = `isFittingBase` (PistonHeadBlock.java:57-60):
    `PistonType::Normal` (generated index 0, "normal") is vanilla
    `PistonType.DEFAULT`, `Sticky` (1, "sticky") is `STICKY`;
    `StickyPistonLikeProperties` supplies `extended`/`facing` for both PISTON and
    STICKY_PISTON states, so the base test is equivalent.
  - `on_state_replaced` (:38-54) = `playerWillDestroy` (:63-71) plus
    `affectNeighborsAfterRemoval` (:75-79): survival drops the base once, creative /
    silk touch (`SKIP_DROPS`) removes it dropless; the `(flags & SKIP_DROPS) |
    NOTIFY_ALL` flags match the audited two-part bed pattern (blocks/bed.rs:175).
    `moved` is deliberately ignored because vanilla
    `affectNeighborsAfterRemoval` ignores `movedByPiston`, and it is safe because
    piston.rs replaces the base with MOVING_PISTON (piston.rs:252-256) before
    `remove_arm_if_head` clears the head (piston.rs:326-332), so a retraction can
    never pass `is_fitting_base` and drop a piston item.
  - `get_state_for_neighbor_update` (:56-78) = `updateShape` (:83-96): only the
    neighbour opposite FACING invalidates the head, via `can_place_at`.
  - `can_place_at` (:80-89) = `canSurvive` (:99-102): fitting base or MOVING_PISTON
    with the same FACING.
  - `on_neighbor_update` (:91-114) = `neighborChanged` (:105-115): survives then
    forwards to the base; the `Some(args.world)` / `.as_ref()` argument shape is
    identical to fire.rs:249-259 and type-checks.
  - `is_pathfindable` (:116-118) = `isPathfindable` (:137-140): false.
  - `rotate`/`mirror`/state definition come from generated block data; light
    occlusion is handled by `pumpkin-world/src/lighting/occlusion.rs:21`
    (PISTON_HEAD is in SINGLETONS for `useShapeForLightOcclusion = true`,
    PistonHeadBlock.java:48-50). Vanilla SHORT is read only by the client
    `getShape` model (PistonHeadBlock.java:53-54), so not writing it in server
    behaviour is correct.

Blocked sub-items (not separate tracker rows; no generated field or subsystem makes
them implementable in this unit):
- BLOCKED-DATA: per-state collision AABB for the plate/stem
  (PistonHeadBlock.java:38-40, 53-55) needs an asset-extractor generated per-state
  shape field plus a `get_shape` hook; `BlockBehaviour` (block/mod.rs:174-341) has
  no such hook.
- BLOCKED-DATA: `getCloneItemStack` (PistonHeadBlock.java:117-120) would need a
  clone-item hook in `BlockBehaviour` plus pick-block dispatch; that trait and
  dispatch change is a different subsystem.

No STILL OPEN rows remain. No `.rs` source was touched, so rustfmt is not
applicable. Verified by reading only (worker cannot compile), so no compile was
run. No pure unit-test seam exists: `is_fitting_base` needs generated state IDs
and a test would only mirror the formula.
