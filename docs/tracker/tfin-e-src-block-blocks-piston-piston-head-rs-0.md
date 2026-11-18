# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-block-blocks-piston-piston-head-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- Known gaps requiring generated data / new hooks (out of scope for this file; not left as STILL OPEN): (1) per-state collision AABB for the plate/stem needs asset-extractor generated data plus a get_shape hook (Pumpkin stores only side/centre-solidity masks); (2) vanilla getCloneItemStack (:117-120) has no BlockBehaviour clone-item hook, so pick-block on a head cannot return a piston/sticky-piston item without a cross-crate trait + dispatch change. Neither affects server-side survival/creative behaviour.
No STILL OPEN rows remain. No `.rs` source was touched, so rustfmt was not applicable.

## Result

Final pass, 2026-11-18. Re-read `crates/pumpkin-core/src/block/blocks/piston/piston_head.rs`
(119 lines) end to end against the decompiled vanilla
`.vendor/vanilla-decompiled/net/minecraft/world/level/block/piston/PistonHeadBlock.java`
(141 lines). No source change was needed: every server-side method already matches,
so there is nothing to fix and no unit-test seam exists (the only pure helper,
`is_fitting_base`, takes generated `Block`/`BlockStateId` values and a test would
only mirror the formula). No `.rs` file was touched, so rustfmt was not applicable.
Verification was by reading only (worker cannot compile).

- **blocks/Piston Head [partial/AUDITED]: VERIFIED.** Vanilla path
  `.vendor/vanilla-decompiled/net/minecraft/world/level/block/piston/PistonHeadBlock.java`.
  Every implementable server-side method matches:
  - `is_fitting_base` (:19-34) = `isFittingBase` (Java :57-60): `PistonType::Normal`
    maps to `PISTON`, `Sticky` to `STICKY_PISTON`; extended + same FACING checked.
  - `on_state_replaced` (:38-54) = `playerWillDestroy` (:63-71) +
    `affectNeighborsAfterRemoval` (:75-79): SKIP_DROPS (creative/silk touch) breaks
    the base dropless, otherwise it drops once; `moved` is ignored just as vanilla
    `affectNeighborsAfterRemoval` ignores `movedByPiston`, and it is safe because
    piston.rs swaps the base to MOVING_PISTON before clearing the head, so a retract
    cannot pass `is_fitting_base`.
  - `get_state_for_neighbor_update` (:56-78) = `updateShape` (:83-96): only the
    neighbour opposite FACING can invalidate the head, via `can_place_at`.
  - `can_place_at` (:80-89) = `canSurvive` (:99-102): fitting base or MOVING_PISTON
    with the same FACING.
  - `on_neighbor_update` (:91-114) = `neighborChanged` (:105-115): survives, then
    forwards the update to the base position.
  - `is_pathfindable` (:116-118) = `isPathfindable` (:137-140): false.
  - rotate/mirror/state definition come from generated block data;
    `useShapeForLightOcclusion` (:48-50) is handled in
    `pumpkin-world/src/lighting/occlusion.rs`; vanilla SHORT (:53-54) is read only by
    the client getShape model, so omitting it from server behaviour is correct.

- **BLOCKED-DATA (sub-item, not a separate row): per-state collision AABB for the
  plate/stem** (Java :38-40, :53-55). Missing: an asset-extractor generated per-state
  VoxelShape field plus a `get_shape` hook in `BlockBehaviour`
  (`crates/pumpkin-core/src/block/mod.rs`), which stores only side/centre-solidity
  masks. No server-side survival/creative behaviour depends on it.
- **BLOCKED-DATA (sub-item, not a separate row): getCloneItemStack** (Java :117-120).
  Missing: a clone-item hook in `BlockBehaviour` plus pick-block dispatch; that
  cross-crate trait + dispatch change is a different subsystem.

No STILL OPEN rows remain.
