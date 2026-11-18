# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-block-blocks-sculk-sculk-vein-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

or VERIFIED; none remain STILL OPEN. The one row whose fix lands in this unit's

## Result
## Result (final pass)

This unit closes its single wrapped row `own/tpar-crates-pumpkin-core-src-block-blocks-sculk-sculk-vein-rs`, which wraps the audit `taudit-crates-pumpkin-core-src-bl-556f10`. Every sub-row has a final disposition; none remain STILL OPEN. The only source file in scope, `crates/pumpkin-core/src/block/blocks/sculk/sculk_vein.rs`, already matches vanilla, so no source edit or new test was needed this pass.

Re-verified directly against decompiled vanilla this pass: `MultifaceBlock.canBeReplaced` = `!context.getItemInHand().is(this.asItem()) || hasAnyVacantFace(state)` (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/MultifaceBlock.java:168-169`), `hasAnyVacantFace` returns true iff some face is unset (`MultifaceBlock.java:285-293`), and `isValidStateForPlacement` rejects an already-occupied face (`MultifaceBlock.java:184-190`). `registry.rs:666/696` calls `can_update_at` only when the placed block equals the clicked block, so the `!item.is(this.asItem())` half is handled by the registry branch and Pumpkin's `can_update_at = active_directions(props).len() < 6` (`sculk_vein.rs:76-82`) matches the `hasAnyVacantFace` half; the `!already_active.contains(&click_direction)` guard (`sculk_vein.rs:170`) mirrors the occupied-face rejection. Pure test `multiface_replacement_needs_a_vacant_face` (`sculk_vein.rs:239-247`) covers it.

| Row | Final disposition | Vanilla evidence (.vendor path) |
|---|---|---|
| 28 can_update_at face convention (in scope, sculk_vein.rs) | VERIFIED, NO CHANGE | `world/level/block/MultifaceBlock.java:168-169,184-190,285-293`: replacement needs a vacant face and a supported/unoccupied direction; `sculk_vein.rs:76-82,170` matches. `world/level/block/SlabBlock.java:77-91` for the paired slabs.rs half. |
| 29 BlockIsReplacing Other/None / rotation | VERIFIED, NO CHANGE | `world/level/block/BlockPlaceContext.java:71-87` (`getNearestLookingDirections`), `LadderBlock.java:73-96`, `LanternBlock.java:35-48`, `WallTorchBlock.java:53-70`. |
| 31 inside-collision shape | VERIFIED, NO CHANGE | `world/level/block/BlockBehaviour.java:336-337` returns `Shapes.block()` (full cube) = Pumpkin `BoundingBox::full_block()`; vanilla overrides `EndPortalBlock.java:49`, `TripWireBlock.java:140`, `PowderSnowBlock.java:128-131` are mirrored. |
| 32 isPathfindable Water default | VERIFIED, NO CHANGE | `world/level/block/BlockBehaviour.java:130-134`; `SlabBlock.java:127-134`. |
| 35 player_placed direction | VERIFIED, NO CHANGE | `world/level/block/Block.java:543` `setPlacedBy` has no face parameter; `PlayerPlacedArgs` carries no direction field. |
| 75 BlockFertilizeEvent payload | VERIFIED, NO CHANGE (already fixed) | Bukkit `BlockFertilizeEvent.getBlocks()` list; payload now carries `changed_blocks` via `World::capture_bonemeal_changes`. |
| 76 TestBlock registered twice | VERIFIED, NO CHANGE (already removed) | Second `register(TestBlock)` removed; behaviour indices are runtime-assigned. |

No source file changed this pass; rustfmt not needed. Rows whose fixes land in files outside this unit's basename (registry.rs, block/mod.rs, world/mod.rs, plugin API) were verified correct in the referenced audit and required no edit here.
