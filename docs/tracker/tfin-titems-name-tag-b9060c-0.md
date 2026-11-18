# Final rows from docs/tracker/titems-name-tag-b9060c.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: armor_stand.rs mod.rs name_tag.rs

- Row 1 (ItemBehaviour use_on_entity result): STILL OPEN, not closed this pass. Returning PASS/SUCCESS needs a framework-wide change to the `ItemBehaviour::use_on_entity` return type in `crates/pumpkin-core/src/item/mod.rs` and every item impl. That is cross-cutting, not a generated-data gap, so it was not attempted. Vanilla reference: `.vendor/vanilla-decompiled/net/minecraft/world/item/` interaction results.

## Result
Result: 0 rows closed this pass. Row 1 remains open with the reason above.
