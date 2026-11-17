# blocks/Chests (RE-REVIEW)

Audited 2026-11-16 against vanilla `ChestBlock`, `ChestBlockEntity`,
`TrappedChestBlock`, `CopperChestBlock`. No decompiled sources present
(`.vendor/` empty), so vanilla behaviour is from the class definitions.

Files: `crates/pumpkin-core/src/block/blocks/chests.rs`,
`crates/pumpkin-core/src/block/entities/chest_like_block_entity.rs`,
`crates/pumpkin-core/src/block/entities/{chest,trapped_chest}.rs`.

## Fixed in this pass

- **Open statistic awarded even when the chest cannot open.** `normal_use_chest_impl`
  raised `OPEN_CHEST`/`TRIGGER_TRAPPED_CHEST` before asking for a screen factory, so a
  chest blocked by a solid block above (or a cat sitting on it) still counted as opened.
  Vanilla `ChestBlock.useWithoutItem` awards the stat only inside
  `if (menuProvider != null)`. Moved the increment inside the `Some(factory)` branch
  (`chests.rs:341`).

## Open gaps

1. **Neighbour half not detached on non-player removal.** `break`/explosion/`/setblock`
   removal of one half leaves the other half with `type=left|right`. `broken_chest_impl`
   (`chests.rs:368`) is only reached through `BlockRegistry::broken` (`registry.rs:1362`),
   which the player-break paths call (`net/java/play/player_action.rs:94`,
   `entity/player.rs:6363`); neither `ChestBlock` nor `TrappedChestBlock` implements
   `on_state_replaced` (`block/mod.rs:226`, dispatched at `world/mod.rs:6202`, `7402`).
   Vanilla `ChestBlock.affectNeighborsAfterRemoval` (`onRemove` before 1.21.5) splits the
   partner on every state change. Fix: implement `on_state_replaced` that reuses the
   `broken_chest_impl` logic. Size S. (Double-run on player break is a no-op: the partner
   is already `single`.)
2. **Blocked-chest test uses the wrong predicate.** `is_chest_blocked` ->
   `has_block_on_top` (`chests.rs:726-733`) uses `is_solid_block()`, i.e. the generic
   "solid" state flag. Vanilla `ChestBlock.isBlockedChestByBlock` tests the block above
   with `isRedstoneConductor`/`isCollisionShapeFullBlock`. Non-full solid blocks (bottom
   slab, stairs, walls, leaves) therefore block opening and hopper extraction here while
   they do not in vanilla. Fix: use `state.is_full_cube()` (already used for the same
   purpose in `block/entities/hopper.rs:528`). Size S; confirm the glass case before
   changing.
3. **Lid event parameter truncated to `u8`.** `on_viewer_count_update` sends
   `new as u8` (`chest_like_block_entity.rs:343`). Vanilla `ChestBlockEntity` block event
   (`EVENT_SET_OPEN_COUNT = 1`) carries the viewer count as an `int`. With more than 255
   simultaneous viewers the count wraps and the client lid animation closes early.
   Fix: widen the synced block event payload to `i32`. Size S (API change, check every
   `on_synced_block_event` implementation).
4. **Double-chest title may differ from vanilla.** `ChestScreenFactory.get_display_name`
   (`chests.rs:67-79`) returns `container.chestDouble` ("Large Chest") above 27 slots.
   Java vanilla builds the menu title from the first half's display name
   (`container.chest` or the custom name); "Large Chest" is Bedrock. Verify against a
   client before changing. Size S.

## Checked and already correct

- Placement `type` computation incl. sneak-click and clicked-face cases; partner state
  update on place (`chests.rs:85-134`, `629-700`).
- Double-chest combine order (`right` half = first) in both the menu factory and the
  hopper/dropper path; obstruction checked on both halves (`chests.rs:138-214`).
- Comparator reads 0 for a blocked chest but still reports output if a block entity
  exists (`chests.rs:217-230`).
- Trapped chest weak signal = clamped viewer count, strong signal only upward
  (`chests.rs:593-618`).
- Loot table deferred unpack: items skipped in NBT while `LootTable` is pending,
  spectator open does not generate loot, both halves unpacked together
  (`chest_like_block_entity.rs:57-79`, `chests.rs:251-318`).
- Hinge sound selection per copper oxidation stage, LEFT half silent, RIGHT half offset
  by the connected direction (`chests.rs:438-459`, `chest_like_block_entity.rs:393-448`).
- Container open/close game events and lid block event on viewer-count transitions
  (`block/viewer.rs:20-35`).
- Waterlogged handling through the generic state property; no per-block fluid hook needed.
