# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-block-blocks-shelf-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

All unresolved rows from the audit (`docs/tracker/taudit-crates-pumpkin-core-src-bl-4d09af.md`)

## Result

Final statuses (MC 26.3, verified against decompiled vanilla):

- Flowing-water waterlogging (shelf.rs:325): VERIFIED. `ShelfBlock.getStateForPlacement` (ShelfBlock.java:125-131) sets `WATERLOGGED = replacedFluidState.is(Fluids.WATER)` (source only); Pumpkin `args.replacing.is_water()` (mod.rs:793) matches. No code change.
- `BlockIsReplacing` water-source migration (mod.rs, 46 call sites): VERIFIED. `is_water()`/`is_any_water()`/`is_full_water()` exist (mod.rs:787-813) with tests (mod.rs:943-965); all call sites migrated, including shelf.rs:325. Cite mod.rs.
- Shelf collision/occlusion shape: VERIFIED. Generated collision shapes present (pumpkin-data block.rs states 4092-4155, `collision_shapes: &[77, 78, 79]`); light occlusion handled for `#minecraft:wooden_shelves` (pumpkin-world occlusion.rs:30-36). Cite occlusion.rs and generated block.rs.
- Piston push reaction for shelves: VERIFIED. Vanilla `PistonBaseBlock` refuses blocks with block entities (PistonBaseBlock.java:284 `return !state.hasBlockEntity();`); shelf has a block entity (generated `block_entity_type: 39`), and Pumpkin piston.rs:97-103 refuses via `!has_block_block_entity(block)`. Cite piston.rs and PistonBaseBlock.java:284.

No source changes; no new test seam beyond existing (shelf.rs:567/581, mod.rs:943-965).
