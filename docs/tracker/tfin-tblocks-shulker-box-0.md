# Final rows from docs/tracker/tblocks-shulker-box.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: shulker_box.rs

All rows previously marked STILL OPEN (1, 2, 4) were re-checked against
Re-checked every row still flagged STILL OPEN in `## h6 gap status` (1, 2, 4) against
- Row 3 (useWithoutItem result): not STILL OPEN; remains Consume with a screen, Pass
Re-verified h6 STILL OPEN rows (1, 2, 4) against `.vendor/vanilla-decompiled/net/minecraft/world/level/block/ShulkerBoxBlock.java` and `.../block/entity/ShulkerBoxBlockEntity.java` and current `blocks/shulker_box.rs` / `block/entities/shulker_box.rs`. Smallest first (2, 4, 1). No source edits; all already match vanilla.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
