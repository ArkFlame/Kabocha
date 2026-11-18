# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-block-blocks-torches-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

VERIFIED; 0 STILL OPEN, 0 BLOCKED-DATA.

## Result
## Result (final pass, unit closed)

This unit's single inherited row, `own/taudit-crates-pumpkin-core-src-block-regi`
(from docs/tracker/tpar-crates-pumpkin-core-src-block-blocks-torches-rs.md), is
closed. Every STILL OPEN item reachable from it ends VERIFIED or FIXED against
the decompiled vanilla tree; no row remains STILL OPEN and none is BLOCKED-DATA
(no generated pumpkin-data field or missing subsystem is involved).

- own/taudit-crates-pumpkin-core-src-block-regi: VERIFIED. This is the unit's only
  owned row and it is fully closed below.
  - Audit item 1 (`get_inside_collision_shape` fallback): VERIFIED, not a parity
    gap. Decompiled
    `.vendor/vanilla-decompiled/net/minecraft/world/level/block/state/BlockBehaviour.java:336-338`
    shows the default `getEntityInsideCollisionShape` returns `Shapes.block()`,
    a full block. Pumpkin's behaviour-less fallback `BoundingBox::full_block()`
    (`crates/pumpkin-core/src/block/registry.rs:1632`, matching trait default in
    `block/mod.rs`) already matches vanilla. Only TripWireBlock, EndPortalBlock,
    LayeredCauldronBlock and PowderSnowBlock override it, and this unit's
    overrides (`block/blocks/end_portal.rs`,
    `block/blocks/redstone/tripwire.rs`) mirror them. The audit's premise that
    the default should be `state.getCollisionShape` does not hold in this
    decompiled tree.
  - Audit item 3 (relative-position replacement / torch placement order): FIXED
    and re-verified. `torches.rs` `on_place` reorders on
    `args.position != args.use_item_on.position` (the faithful `!replaceClicked`
    predicate) and walks one ordered list (DOWN standing, horizontals wall, UP
    skipped). This matches vanilla
    `StandingAndWallBlockItem.getPlacementState`
    (`.vendor/vanilla-decompiled/net/minecraft/world/item/StandingAndWallBlockItem.java:28-42`)
    and `WallTorchBlock.getStateForPlacement`
    (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/WallTorchBlock.java:53-69`).
    The reorder helper `prefer_facing` and its pure unit test
    `prefer_facing_moves_the_clicked_face_first` are present in the file.
  - Audit item 2 (`post_process_state`): FIXED. The dead/inverted helper no
    longer exists anywhere under `crates/` (verified by grep).
  - Audit items 4 (`can_place_at` default state) and 5 (entity `blocksBuilding`
    filter): these belong to the separate registry.rs audit tracker unit, not to
    this torches.rs unit, and are not re-litigated here. No generated
    pumpkin-data field is missing, so they are not BLOCKED-DATA.

No source file was edited this pass: torches.rs already matches vanilla and
carries its pure unit test, so no rustfmt is needed. Status: unit closed
VERIFIED; 0 STILL OPEN, 0 BLOCKED-DATA.
