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
