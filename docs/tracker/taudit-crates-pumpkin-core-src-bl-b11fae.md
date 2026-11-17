# Audit: crates/pumpkin-core/src/block/mod.rs

Coverage: AUDIT. No TODO/FIXME markers; the two "vanilla" hits are documentation comments
(push_entities_up, BlockDropItemContext) and are correct.

Fixed in this audit:
- `drop_loot` + `World::break_block`: the block entity is captured before removal and passed
  through `LootContextParameters`, matching vanilla `Block.dropResources(state, level, pos,
  blockEntity, entity, tool)` / `ServerPlayerGameMode.destroyBlock`. Before the fix a
  player-broken banner/sign/shulker lost its components because `set_block_state` had already
  removed the block entity when loot ran.

Remaining gaps:

1. `crates/pumpkin-core/src/block/mod.rs:252` - `get_inside_collision_shape` default returns
   `BoundingBox::full_block()`. Vanilla `BlockBehaviour.getInsideCollisionShape` returns the
   block's collision shape and only falls back to `Shapes.block()` when that shape is empty.
   Vanilla class: `net.minecraft.world.level.block.state.BlockBehaviour`.
   Fix: fold `args.state.get_block_collision_shapes()` into one box and fall back to full block
   when empty; the trait returns a single box although vanilla can return a multi-box
   `VoxelShape`, so a shape-list API would be needed for exact parity. Audit pressure plates and
   `Entity::check_inside_blocks` users before changing. Size: M.

2. `crates/pumpkin-core/src/block/mod.rs:809` - `calculate_comparator_output` divides by
   `stack.get_max_stack_size()` only. Vanilla
   `AbstractContainerMenu.getRedstoneSignalFromContainer` divides by
   `Math.min(container.getMaxStackSize(), itemStack.getMaxStackSize())`.
   Vanilla class: `net.minecraft.world.inventory.AbstractContainerMenu`.
   Fix: add `get_max_stack_size()` (default 64) to `pumpkin_inventory::Inventory`, use the min.
   Size: S.

3. `crates/pumpkin-core/src/block/mod.rs:45` - `push_entities_up` pushes by
   `new_top - old_top` and searches entities in the plain block cell. Vanilla
   `Block.pushEntitiesUp` early-returns unless the new collision top is >= 1.0 and inflates the
   search AABB upward by that top. Confirm against the decompiled class before changing; with a
   partial-height replacement the current code may skip entities that vanilla moves.
   Size: S.

4. `crates/pumpkin-core/src/block/mod.rs:663` - `BlockDropItemEvent` is always fired with
   `player: None`, even for player breaks. Paper's event carries the breaking player (null only
   for non-player causes). `drop_loot` has no player parameter; thread `Option<&Player>` from
   `World::break_block` / `break_block_naturally`. Size: M.

5. `crates/pumpkin-core/src/block/mod.rs:754` - `BlockPlaceContext::at` only special-cases
   water, while vanilla's `BlockPlaceContext` constructor sets `replaceClicked` from
   `state.canBeReplaced(context)`. Current callers (`ScaffoldingItem`) pre-check replaceability,
   so no live bug, but the helper is unsafe for new callers. Size: S.

6. `crates/pumpkin-core/src/block/mod.rs:185` and `:260` - `can_place_inside_entities` and
   `has_placement_collision` both encode scaffolding's empty placement collision, while vanilla
   checks `state.getCollisionShape(...).isEmpty()` once. Consolidating avoids drift between the
   two paths. Size: S.

7. `crates/pumpkin-core/src/block/mod.rs:677` - silk touch is looked up by name on every break;
   use the generated `Enchantment` constant if one exists. Cleanup only. Size: S.
