# Final rows from docs/tracker/tblocks-campfire.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- Row 12 (`isSmokeyPos` obstruction probe): STILL OPEN, needs decompiled source to confirm the stop rule.
Reviewed but not in the STILL OPEN set (no change made):
- Row 12 (STILL OPEN, `isSmokeyPos` obstruction probe): VERIFIED already fixed,
- Row 12 (the only row explicitly marked STILL OPEN, h6 status line 57):

## Result

Row 12 (`isSmokeyPos` obstruction probe): VERIFIED, already fixed in tree, no code
change needed. Decompiled `CampfireBlock.isSmokeyPos`
(`.vendor/vanilla-decompiled/net/minecraft/world/level/block/CampfireBlock.java:268-284`)
loops `i = 1..=5`: at the first cell whose collision shape intersects the virtual
post it checks the single cell directly below (`posToCheck.below()`) and returns
`isLitCampfire(belowState)`, otherwise returns true on a lit campfire and false
after the scan. `SHAPE_VIRTUAL_POST = Block.column(4.0, 0.0, 16.0)` (`:61`) is
x/z 0.375..0.625 and y 0.0..1.0. Rust
`CampfireBlock::is_smokey_pos_from` (`block/blocks/campfire.rs:64-77`) mirrors
this exactly: `for dy in 1..=5`, lit returns true, first `blocks_smoke`
(intersection with the 0.375..0.625 / y 0..1 virtual post at `:47-50,55-56`)
returns `state_at(dy + 1).0`, else false. `state_at(dy)` reads `pos.below(dy)`
and `state_at(dy+1)` reads `posToCheck.below()`, matching vanilla. The pure test
`smoke_scans_five_cells_and_stops_at_a_blocker` (`:277-288`) covers
lit-under-blocker true, unlit-under-blocker false, lit-cell self-block and the
range edge. The benign difference that vanilla passes the origin `pos` (Rust
the actual cell `check_pos`) to `getCollisionShape` is kept as-is: static
campfire-cell collision shapes are position independent.

No STILL OPEN rows remain. No source file was modified this pass.
