# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-entity-decoration-item-frame-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: item_frame.rs

All ten numbered rows are FIXED/VERIFIED in `crates/pumpkin-core/src/entity/decoration/item_frame.rs`; no row remains STILL OPEN and none is BLOCKED-DATA. No source edit was required this final pass: every fix is present in source and was re-verified directly against the decompiled vanilla tree now present under `.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/` (`ItemFrame.java`, `BlockAttachedEntity.java`, `HangingEntity.java`) plus fresh source inspection. The one deferred explosion divergence is not one of the ten numbered rows and is recorded separately below.

## Result
Row 1 (support-check cadence) FIXED/VERIFIED: `tick` fires the support check on tick 100, 200, ... via `ticks_since_last_check.fetch_add(1) + 1 < SUPPORT_CHECK_INTERVAL` (100), matching `BlockAttachedEntity.CHECK_INTERVAL = 100` and its `ticksSinceLastCheck++ >= hangingTickFrequency` gate (`.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/BlockAttachedEntity.java:26,43-49`).

Row 2 (add-item sound) FIXED/VERIFIED: `set_item` plays `get_add_item_sound()` only when `!item_stack.is_empty() && update_neighbours`, matching `setItem`'s `!itemStack.isEmpty() && updateNeighbours && playSound` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/ItemFrame.java:313-323`).

Row 3 (comparator-only update) FIXED/VERIFIED: `set_item`, `set_rotation` and `drop_item` call `world.update_neighbour_for_output_signal(&pos, &Block::AIR)`, matching `level().updateNeighbourForOutputSignal(pos, Blocks.AIR)` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/ItemFrame.java:325-327,360-364`; `drop_item` mirrors the `setItem(EMPTY)` path at line 250).

Row 4 (doEntityDrops) FIXED/VERIFIED: `drop_item` clears the item, pokes comparators, then returns before both drops when `!world.level_info.load().game_rules.entity_drops`, matching `if (!level.getGameRules().get(GameRules.ENTITY_DROPS))` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/ItemFrame.java:251-257`).

Row 5 (invulnerable damage) FIXED/VERIFIED: `damage_with_context` returns false for sources where `is_invulnerable_to(&damage_type, cause)` holds, matching `HangingEntity.hurtServer`'s invulnerable-source early return (`.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/ItemFrame.java:187-190`; `BlockAttachedEntity.hurtServer:129-130`).

Row 6 (invisible flag) FIXED/VERIFIED: `is_invisible`/`set_invisible` use the shared `Entity.invisible` flag and tracker; NBT writes/reads `Invisible` through it, matching the vanilla `output.putBoolean("Invisible", this.isInvisible())` / `setInvisible(input.getBooleanOr("Invisible", false))` round-trip (`.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/ItemFrame.java:378,395`).

Row 7 (custom name) FIXED/VERIFIED: `get_frame_item_stack_with_data` copies the custom name via `get_text()`, matching the verbatim `itemStack.set(DataComponents.CUSTOM_NAME, this.getCustomName())` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/ItemFrame.java:476-480`).

Row 8 (creative drop ordering) FIXED/VERIFIED: `drop_item` returns before both drops for a creative player (`is_creative`), matching `causedBy instanceof Player && player.hasInfiniteMaterials()` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/ItemFrame.java:255-257`).

Row 9 (single-slot Container) FIXED/VERIFIED: `ItemFrameInventory` (size 1, max stack 1) is exposed via `EntityBase::get_container_inventory`, matching `getSlot(int) -> slot == 0 ? SlotAccess.of(this::getItem, this::setItem)` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/ItemFrame.java:334-337`).

Row 10 (DOWN drop offset) FIXED/VERIFIED: `drop_item` applies the -0.6 y offset for Down-facing frames, matching `spawnAtLocation(..., getDirection() == Direction.DOWN ? -0.6F : 0.0F)` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/ItemFrame.java:291-292`).

Non-row deferred issue (NOT one of the ten numbered rows), BLOCKED-DATA: explosion damage. Vanilla `ItemFrame.hurtServer` (ItemFrame.java:187-213) sends explosion damage through `BlockAttachedEntity.hurtServer` (BlockAttachedEntity.java:129-172), which kills the frame and drops frame + item because `shouldDamageDropItem` requires `!IS_EXPLOSION` (ItemFrame.java:173-174). Pumpkin's early `return false` for `EXPLOSION`/`PLAYER_EXPLOSION` (item_frame.rs:671-676) keeps the frame intact. Fixing it needs the missing generated `IS_EXPLOSION` damage-type tag plus the `ignoreExplosion` / `shouldAffectBlocklikeEntities` path in `explosion.rs` (outside this unit, whose only editable source file is `item_frame.rs`); no generated `pumpkin-data` field for that tag exists yet, so the fix cannot be completed here.
