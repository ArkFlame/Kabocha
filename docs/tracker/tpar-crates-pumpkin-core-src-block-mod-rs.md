# Parity rows for crates/pumpkin-core/src/block/mod.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-bl-b11fae** [partial/OWN-OPEN] own audit: 7 open, 3 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-bl-b11fae.md) docs/tracker/taudit-crates-pumpkin-core-src-bl-b11fae.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)


## Result (third pass)

All seven rows re-verified against decompiled vanilla at `.vendor/vanilla-decompiled/net/minecraft/`. No source edits were needed: every row is already fixed in `crates/pumpkin-core/src/block/mod.rs` (plus its cross-file callers) and matches vanilla.

- Row 1 (get_inside_collision_shape): VERIFIED. The default `BoundingBox::full_block()` matches vanilla `BlockBehaviour.getEntityInsideCollisionShape`, which returns `Shapes.block()` (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/state/BlockBehaviour.java:336-337`). The audit's empty-collision-shape fallback is not present in this version; the blocks that override it (tripwire, end portal) already do so in `block/blocks/`. No edit.
- Row 2 (comparator output): FIXED. `calculate_comparator_output` divides by `min(inventory.get_max_count_per_stack(), stack.get_max_stack_size())`, matching vanilla `AbstractContainerMenu.getRedstoneSignalFromContainer` (`.vendor/vanilla-decompiled/net/minecraft/world/inventory/AbstractContainerMenu.java:905-920`). Covered by test `comparator_output_uses_smaller_container_or_item_max_stack`. The expected cross-crate getter already exists on `pumpkin_inventory::Inventory` as `get_max_count_per_stack`.
- Row 3 (pushEntitiesUp): FIXED. `added_collision_boxes` mirrors `Shapes.joinUnoptimized(..., BooleanOp.ONLY_SECOND)` and `collide_down_y` mirrors `Shapes.collide(Direction.Axis.Y, bbox.move(0,1,0), ..., -1.0)`, so entities are raised by `1.0 + offset` exactly as in vanilla `Block.pushEntitiesUp` (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/Block.java:158-170`). Covered by tests `added_collision_boxes_is_new_minus_old` and `collide_down_y_matches_vanilla_shape_collision`. No cross-crate VoxelShape API was required; `BoundingBox::subtract` and `minmax` reproduce the needed ops locally.
- Row 4 (BlockDropItemEvent player): FIXED. `drop_loot` takes `Option<&Arc<Player>>` and sets `player: player.cloned()` on the event. `World::break_block` (world/mod.rs:7171) passes the breaking `cause`; explosion.rs:974 and leaves.rs:121 pass `None` for non-player removals, matching Paper's event carrying the breaking player only for player breaks.
- Row 5 (BlockPlaceContext::at): VERIFIED (already fixed in an earlier pass). It gates on `can_replace_with_other_block` and sets `inside: false`, matching the vanilla constructor `replaceClicked = state.canBeReplaced(this)` and the false "inside" flag (`.vendor/vanilla-decompiled/net/minecraft/world/item/context/BlockPlaceContext.java:25-47`).
- Row 6 (consolidation): VERIFIED (already fixed). `has_placement_collision()` defaults to `!can_place_inside_entities()`; test `placement_collision_mirrors_can_place_inside_entities`.
- Row 7 (silk touch): VERIFIED (already fixed). Per-break name lookup replaced with the generated `Enchantment::SILK_TOUCH` constant.

No BLOCKED-DATA rows: no missing generated pumpkin-data field and no missing subsystem remain. No source edits or rustfmt were needed this pass; no compile/test run (worker constraint).
