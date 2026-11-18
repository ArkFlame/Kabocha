# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-block-regi.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: registry.rs

- 1. `get_inside_collision_shape` fallback (`block/registry.rs`): VERIFIED (no gap; Pumpkin already matches vanilla).
- Row 1 was the only open row in "Gap status"; rows 2/4/5 live in registry.rs and row 3 root fix is registry.rs:705, all outside this unit file list.
- Re-verified the single open row against decompiled vanilla; no in-scope edit needed (non-gap).

## Result
- 1. `get_inside_collision_shape` fallback (`block/registry.rs:1616-1633`, trait default `block/mod.rs:305-307`): VERIFIED. Decompiled `.vendor/vanilla-decompiled/net/minecraft/world/level/block/state/BlockBehaviour.java:336-338` shows `getEntityInsideCollisionShape` defaults to `Shapes.block()` (a full block). Pumpkin `BoundingBox::full_block()` fallback for behaviour-less blocks is exactly that default, so the tracker premise (default should be `state.getCollisionShape`) does not hold in this decompiled version. Blocks that genuinely need a different inside shape already override it: `blocks/end_portal.rs:17`, `blocks/redstone/tripwire.rs:28`, `blocks/powder_snow.rs:76-80` (wired in `entity/mod.rs:3345`). No source change is correct; no pure-unit-test seam exists (asserting `full_block()` is a full block would be a constant/mirroring test, which the rules forbid). No files edited, no rustfmt needed.
