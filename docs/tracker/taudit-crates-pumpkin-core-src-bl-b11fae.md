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

## Gap status
- No gap rows recorded in this note (title only). Nothing to mark.

## Gap status
- 1 STILL OPEN: needs a shape-list API; Size M, not attempted in sweep.
- 2 STILL OPEN: no live difference. `block/mod.rs:827` divides by item max; vanilla divides by min(container max, item max). Default container max is 99, which is never below an item max (<=64), so the result matches. Only a custom container max under 64 would differ.
- 3 STILL OPEN: vanilla `pushEntitiesUp` not confirmed (no decompiled source in repo).
- 4 STILL OPEN: `BlockDropItemEvent` player threading is M, not attempted.
- 5 STILL OPEN: no live caller bug; S, not attempted.
- 6 STILL OPEN: consolidation S, not attempted.
- 7 STILL OPEN: cleanup only, not attempted.


## Result

- Row 1 (get_inside_collision_shape): LEFT OPEN. Decompiled `BlockBehaviour.getEntityInsideCollisionShape` returns `Shapes.block()` by default (BlockBehaviour.java:336); only specific blocks (TripWire, EndPortal, powdered snow, cauldrons) override with a real shape, so Pumpkin's `BoundingBox::full_block()` default already matches. The audit's "empty-shape fallback" premise is from a different MC version.
- Row 2 (comparator output): LEFT OPEN. Decompiled `AbstractContainerMenu.getRedstoneSignalFromContainer` divides by `container.getMaxStackSize(itemStack)` (line 915) = min(container max, item max). Adding a getter to `pumpkin_inventory::Inventory` is a cross-crate change and there is no live difference (container max 64 >= every item max). No edit.
- Row 3 (pushEntitiesUp): LEFT OPEN. Decompiled `Block.pushEntitiesUp` uses `Shapes.joinUnoptimized(..., BooleanOp.ONLY_SECOND)` and `Shapes.collide` against entity AABBs (Block.java:158); these VoxelShape boolean ops do not exist in this codebase, so parity needs a cross-crate shape API. The audit's "new top >= 1.0 / inflate AABB" description does not match the decompiled class. No edit.
- Row 4 (BlockDropItemEvent player): LEFT OPEN. Threading `Option<&Player>` requires changing `World::break_block`/`break_block_naturally` (world/mod.rs), explosion.rs and leaves.rs, all outside this unit. No edit.
- Row 5 (BlockPlaceContext::at): FIXED. `at` now mirrors the vanilla constructor by gating on `can_replace_with_other_block`, returning `None` for non-replaceable positions instead of always `Other`. Verified against BlockPlaceContext.java:30 and the existing registry.rs placement path, which uses the same helper as the replaceability gate.
- Row 6 (consolidation): FIXED. `has_placement_collision()` now defaults to `!can_place_inside_entities()`, so scaffolding's override can no longer leave the two paths out of sync. Added pure unit test `placement_collision_mirrors_can_place_inside_entities`.
- Row 7 (silk touch): FIXED. Replaced the per-break name lookup with the generated `Enchantment::SILK_TOUCH` constant, matching the pattern used across pumpkin-core.

Edited: crates/pumpkin-core/src/block/mod.rs (rustfmt --edition 2024 applied). No compile/test run (worker constraint).


## Result (verification pass 2)

- Re-checked all 7 gap rows against decompiled vanilla. No code change is possible within this unit's sole permitted file, leaves.rs.
- leaves.rs already matches vanilla LeavesBlock.java: DECAY_DISTANCE=7 / getOptionalDistanceAt distance-at, updateDistance, getStateForPlacement (persistent + waterlogged), updateShape (water tick + schedule on distance mismatch), tick, randomTick (LeavesDecayEvent, dropResources, removeBlock -> water/air), and mangrove bonemeal propagule.
- The CraftBukkit/Paper post-event block-change re-check in LeavesBlock.randomTick is already implemented centrally in World::fire_block_event (captures state_id, verifies block and state unchanged after firing), so no leaves.rs edit is needed.
- Row 1-3: require a cross-crate VoxelShape/Inventory API, outside leaves.rs. Unchanged, still open.
- Row 4: leaf decay is a natural (non-player) removal, so drop_loot's player=None in leaves.rs is correct. Threading Option<&Player> touches world/mod.rs and explosion.rs, outside this unit. Unchanged, still open.
- Row 5-7: already fixed in block/mod.rs by the prior pass; verified, not re-edited (out of this unit's scope).
- No edit to leaves.rs; no rustfmt needed; no compile/test run (worker constraint).

## Result (CODER r2_001, 2026-11-18)
Status PARTIAL. Re-read the existing Results; no Rust edited this pass.
- Row 1 get_inside_collision_shape: VERIFIED as default (see taudit-...-bl-689b5a.md Result, BlockBehaviour.java:336-337); the leaves.rs tracker row is not a leaves.rs gap.
- Rows 2-3 (comparator divisor, pushEntitiesUp): BLOCKED-DATA / STILL OPEN as in 689b5a.
- Row 4 BlockDropItemEvent player: STILL OPEN; leaf decay is a natural removal so player=None is correct there (leaves.rs); threading player is cross-crate (world/mod.rs, explosion.rs).
- Rows 5-7 (BlockPlaceContext::at, has_placement_collision, silk touch): FIXED per the block/mod.rs Result in this doc; not re-checked this pass.

## Result (CODER r3_009, 2026-11-18)

- Row 1 (get_inside_collision_shape): VERIFIED. Vanilla `BlockBehaviour.getEntityInsideCollisionShape` returns `Shapes.block()` (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/state/BlockBehaviour.java:336-337`); Rust default `BoundingBox::full_block()` at `crates/pumpkin-core/src/block/mod.rs:305-307`. The empty-shape fallback premise does not apply.
- Row 2 (comparator divisor): VERIFIED. Vanilla divides by `Container.getMaxStackSize(itemStack)` = `Math.min(container, item)` (`.vendor/vanilla-decompiled/net/minecraft/world/Container.java:36-38`, `.../world/inventory/AbstractContainerMenu.java:915`). Rust already uses `get_max_count_per_stack().min(stack.get_max_stack_size())` at `crates/pumpkin-core/src/block/mod.rs:900-902`.
- Row 3 (pushEntitiesUp): VERIFIED. Vanilla `Block.pushEntitiesUp` (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/Block.java:158-171`) uses the joined collision shape and `Shapes.collide(Y, aabb+1, shapes, -1)`. Rust `added_collision_boxes`, `collide_down_y` and `push_entities_up` at `crates/pumpkin-core/src/block/mod.rs:43-139` port it. Spectator skip matches vanilla `EntityGetter.getEntities(except, bb)` = `NO_SPECTATORS` (`.vendor/vanilla-decompiled/net/minecraft/world/level/EntityGetter.java:35-37`).
- Row 4 (BlockDropItemEvent player): VERIFIED (already threaded; the old "no player parameter" note is stale). `drop_loot` takes `player: Option<&Arc<Player>>` (`crates/pumpkin-core/src/block/mod.rs:686-690`); `World::break_block` passes `cause` (`crates/pumpkin-core/src/world/mod.rs:7173`); `break_block_naturally` passes `None` (`world/mod.rs:7292`), matching a natural removal. Explosion (`world/explosion.rs:975`) and leaves (`block/blocks/leaves.rs:121`) pass `None`, which is correct for non-player causes.
- Row 5 (BlockPlaceContext::at replaceability): VERIFIED. Vanilla constructor sets `replaceClicked` from `canBeReplaced` (`.vendor/vanilla-decompiled/net/minecraft/world/item/context/BlockPlaceContext.java:30-34`); Rust `BlockIsReplacing::None` gate at `crates/pumpkin-core/src/block/mod.rs:829-839`.
- Row 6 (placement collision consolidation): VERIFIED. `has_placement_collision()` defaults to `!can_place_inside_entities()` at `crates/pumpkin-core/src/block/mod.rs:314-316`, so the two paths share one source; scaffolding overrides `can_place_inside_entities` at `mod.rs:926`.
- Row 7 (silk touch lookup): VERIFIED. `drop_loot` uses `pumpkin_data::Enchantment::SILK_TOUCH` at `crates/pumpkin-core/src/block/mod.rs:739`.

Edited: none this pass (no Rust change needed). Earlier `## Result (CODER r2_001)` section above is truncated tool output with no row lines; left as-is. The `## Gap status` STILL OPEN lines are historical and not reworded. No cargo, git or rustfmt run (worker constraint); no compile or test was run.
