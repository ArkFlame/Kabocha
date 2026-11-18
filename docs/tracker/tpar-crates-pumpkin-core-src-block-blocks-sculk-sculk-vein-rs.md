# Parity rows for crates/pumpkin-core/src/block/blocks/sculk/sculk_vein.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-bl-556f10** [verified] own audit: 10 open, 8 fixed, 8 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-bl-556f10.md) docs/tracker/taudit-crates-pumpkin-core-src-bl-556f10.md

## Result (final resolution)

Closed the single row `own/taudit-crates-pumpkin-core-src-bl-556f10`, which wraps
the audit in `docs/tracker/taudit-crates-pumpkin-core-src-bl-556f10.md`. That
audit's final Result sections resolve every originally-STILL-OPEN row to FIXED
or VERIFIED; none remain STILL OPEN. The one row whose fix lands in this unit's
editable basename (`sculk_vein.rs`) is re-verified below against vanilla.

| Row | Final disposition | Evidence |
|---|---|---|
| own audit row 28 (in scope, sculk_vein.rs) can_update_at face convention | VERIFIED, NO NEW EDIT | `MultifaceBlock.canBeReplaced` = `!is(this.asItem()) || hasAnyVacantFace(state)` (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/MultifaceBlock.java:168-169`); `hasAnyVacantFace` returns true iff some face is unset (MultifaceBlock.java:285-293). Pumpkin `can_update_at` = `active_directions(props).len() < 6` (sculk_vein.rs:76-82) is identical. `isValidStateForPlacement` (MultifaceBlock.java:184-190) rejects an occupied face; Pumpkin guards step 1 with `!already_active.contains(&click_direction)` (sculk_vein.rs:169-172). Pure test `multiface_replacement_needs_a_vacant_face` (sculk_vein.rs:239-247) covers the fix. |
| own audit rows 29/31/32/35/75/76 (out of scope basenames) | VERIFIED / FIXED per referenced audit | Resolved with .vendor evidence in the referenced audit's final Result sections: 29 BlockIsReplacing/rotation VERIFIED (BlockPlaceContext.java:71-87, LadderBlock/LanternBlock/WallTorchBlock); 31 inside-collision VERIFIED (BlockBehaviour.java:336-337 returns `Shapes.block()`, so Pumpkin's `BoundingBox::full_block()` default is correct, premise was wrong); 32 isPathfindable Water VERIFIED (BlockBehaviour.java:130-134); 35 setPlacedBy has no face parameter (Block.java:543), `PlayerPlacedArgs.direction` removed; 75 BlockFertilizeEvent payload already fixed (plugin API `changed_blocks`); 76 duplicate `TestBlock` registration already removed. |

No source edits were needed this pass: the in-scope fix is already present and
matches vanilla, so there is no new seam requiring a pure unit test. No source
file changed; rustfmt not needed.
