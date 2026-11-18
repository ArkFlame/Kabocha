# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-1730b6.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

implemented. Gap status above is updated so no numbered row remains STILL OPEN.
This is the terminal Result: no numbered row remains STILL OPEN and none is BLOCKED-DATA.

## Result
Terminal closure for `docs/tracker/taudit-crates-pumpkin-core-src-en-1730b6.md` (item_frame.rs). Source re-inspected in `crates/pumpkin-core/src/entity/decoration/item_frame.rs`; no additional code edit required this pass. No pure unit-test seam exists (affected behaviour requires a `World`/fixture; `calculate_bounding_box` is a pure formula and formula-mirroring tests are disallowed). No numbered row remains STILL OPEN and none is BLOCKED-DATA.

- Row 1 (support-check cadence) VERIFIED: `tick` uses `SUPPORT_CHECK_INTERVAL = 100` and `fetch_add(1, ..) + 1 < 100` so the support check runs on ticks 100, 200, ... matching vanilla `BlockAttachedEntity.CHECK_INTERVAL = 100` / `ticksSinceLastCheck++ >= hangingTickFrequency`. Cite: `.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/BlockAttachedEntity.java`.
- Row 2 (add-item sound) VERIFIED: `set_item` plays `get_add_item_sound()` only when `!item_stack.is_empty() && update_neighbours`, matching vanilla `if (!itemStack.isEmpty() && updateNeighbours && playSound(...))`. Cite: `.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/ItemFrame.java`.
- Row 3 (comparator-only update) VERIFIED: `set_item`, `set_rotation` and `drop_item` call `world.update_neighbour_for_output_signal(&pos, &Block::AIR)`, matching vanilla `level().updateNeighbourForOutputSignal(this.pos, Blocks.AIR)`. Cite: `.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/ItemFrame.java`.
- Row 4 (doEntityDrops) VERIFIED: `drop_item` clears the item and comparator signal then returns before both drops when `world.level_info.load().game_rules.entity_drops` is false, matching vanilla `if (!level.getGameRules().get(GameRules.ENTITY_DROPS)) ...`. Cite: `.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/ItemFrame.java`.
- Row 5 (invulnerable damage) VERIFIED: `damage_with_context` returns false for `is_invulnerable_to(&damage_type, cause)` sources, matching vanilla `if (this.isInvulnerableToBase(source)) return false;`. Cite: `.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/ItemFrame.java`, `.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/BlockAttachedEntity.java`.
- Row 6 (invisible flag) VERIFIED: `is_invisible`/`set_invisible` use the shared `Entity.invisible` flag and NBT reads/writes `Invisible` through it, matching vanilla `setInvisible(...)` / `putBoolean("Invisible", ...)` / `getBooleanOr("Invisible", ...)`. Cite: `.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/ItemFrame.java`.
- Row 7 (custom name) VERIFIED: `get_frame_item_stack_with_data` copies the custom name via `custom_name.get_text()`, matching vanilla `itemStack.set(DataComponents.CUSTOM_NAME, this.getCustomName())` (Pumpkin stores a plain String, so component styles are not preserved). Cite: `.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/ItemFrame.java`.
- Row 8 (creative drop ordering) VERIFIED: `drop_item` returns before both drops for a creative causing player, matching vanilla `causedBy instanceof Player player && player.hasInfiniteMaterials()`. Cite: `.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/ItemFrame.java`.
- Row 9 (single-slot Container) VERIFIED: `ItemFrameInventory` (size 1, max stack 1) is exposed via `has_container_inventory`/`get_container_inventory`, matching vanilla `slot == 0 ? SlotAccess.of(this::getItem, this::setItem) : SlotAccess.NULL`. Cite: `.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/ItemFrame.java`.
- Row 10 (DOWN drop offset) VERIFIED: `drop_item` applies a -0.6 y offset for Down-facing frames, matching the Paper MC-123848 override `this.getDirection() == Direction.DOWN ? -0.6F : 0.0F`. Cite: `.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/ItemFrame.java`.

Non-row note (not a numbered row): explosion handling differences outside these ten rows are not marked STILL OPEN in this unit.
