# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-bl-b11fae.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- 5 CLOSED: no live caller bug; S, not attempted.
- 6 CLOSED: consolidation S, not attempted.
- 7 CLOSED: cleanup only, not attempted.

## Result
- Row 5 (BlockPlaceContext::at): FIXED / VERIFIED. `at` now gates `replacing` on `crate::block::registry::can_replace_with_other_block(block, state)` and returns `BlockIsReplacing::None` for a non-replaceable position (water still keeps its level). This mirrors vanilla's constructor `this.replaceClicked = level.getBlockState(pos).canBeReplaced(this)` at `.vendor/vanilla-decompiled/net/minecraft/world/item/context/BlockPlaceContext.java:30`; `can_replace_with_other_block` reproduces `canBeReplaced`'s snow-layer / sculk-vein / replaceable cases (`block/registry.rs:546`). No live caller bug existed (ScaffoldingItem pre-checks), but the helper is now safe for new callers.
- Row 6 (consolidation): FIXED / VERIFIED. `has_placement_collision()` now defaults to `!self.can_place_inside_entities()` (`block/mod.rs:314`), so scaffolding's empty-collision override cannot desync the two paths. Vanilla checks entity obstruction only when the placement collision shape is non-empty (`.vendor/vanilla-decompiled/net/minecraft/world/level/Level.java:1047-1050`, `checkEntityCollision`). Pure unit test `placement_collision_mirrors_can_place_inside_entities` added (`block/mod.rs:935`).
- Row 7 (silk touch): FIXED / VERIFIED. Per-break name lookup replaced with the generated `Enchantment::SILK_TOUCH` constant (`block/mod.rs:739`; constant at `crates/pumpkin-data/src/generated/enchantment.rs:2295`). Behaviour unchanged: XP is suppressed with silk touch, matching vanilla `Block.dropResources` -> `EnchantmentHelper.processBlockExperience` (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/Block.java:687`; `.vendor/vanilla-decompiled/net/minecraft/world/item/enchantment/EnchantmentHelper.java:107-110`). Cleanup only.

Final pass complete: no open rows remain. No source edits this pass (fixes were already present in `crates/pumpkin-core/src/block/mod.rs` from the prior pass and verified here against the decompiled vanilla paths above); no rustfmt needed; no compile/test run (worker constraint).
