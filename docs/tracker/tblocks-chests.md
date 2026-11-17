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

## h2 parity sweep (status)
- FIXED (existing tree) - `has_block_on_top` already uses `is_full_cube()` (chests.rs:1085-1091).
- Other S rows (partner on state change, synced viewer count width, custom-name labels): STILL OPEN, not attempted this pass.


## Result

Re-reviewed against decompiled vanilla (`ChestBlock.java`, `ChestBlockEntity.java`)
2026-11-17. The STILL OPEN rows from the h2 pass (1, 3, 4) were each checked.
No `cargo` was run by this worker (per unit rules). `rustfmt --edition 2024` was
run on `chests.rs`.

- **Row 1 (partner half detached on non-player removal): FIXED, already in tree.**
  All three chest blocks implement `get_state_for_neighbor_update` ->
  `chest_state_for_neighbor_update` (chests.rs), a faithful port of vanilla
  `ChestBlock.updateShape` (decompiled lines 172-200): a lone half flips back to
  `ChestType.SINGLE` when its connected neighbour changes. This hook is reached on
  every state change via `World::on_block_state_set` ->
  `BlockRegistry::update_neighbors` -> `replace_with_state_for_neighbor_update`,
  so break/explosion//setblock are all covered; no `on_state_replaced` needed.
  No code change made.
- **Row 2 (blocked-chest predicate): FIXED, already in tree.** `has_block_on_top`
  uses `state.is_full_cube()` (chests.rs:1109). Confirmed vanilla
  `isBlockedChestByBlock` tests the block above with `isRedstoneConductor` (a
  collision-full block), so slabs/stairs/walls/leaves do not block. No change.
- **Row 3 (lid event viewer count truncated to u8): STILL OPEN (out of unit).**
  Vanilla `ChestBlockEntity` sends the viewer count as an `int`
  (`EVENT_SET_OPEN_COUNT = 1`), but `add_synced_block_event` takes `data: u8`
  (world/mod.rs:1331) and `OnSyncedBlockEventArgs.data` is `u8` (block/mod.rs),
  so `on_viewer_count_update` must cast `new as u8` (chest_like_block_entity.rs:344).
  Widening to `i32` requires edits to world/mod.rs (BlockEvent + signature +
  `CBlockEvent` broadcast), block/mod.rs and the protocol block-event packet, all
  outside this unit's files. Reason: cross-crate/protocol API change required.
- **Row 4 (double-chest menu title): FIXED.** Verified against vanilla
  `ChestBlock.MENU_PROVIDER_COMBINER.getDisplayName` (decompiled lines 130-135):
  first (RIGHT) half custom name, else second (LEFT) half custom name, else
  `container.chestDouble` ("Large Chest") for double chests; single chests use
  `container.chest`. The old tree dropped the second half's name and always
  fell back to `container.chest`. `chests.rs` now resolves both halves' custom
  names and selects the default key by `double_chest`; added pure unit test
  `chest_menu_title_matches_vanilla`.

Files changed: `crates/pumpkin-core/src/block/blocks/chests.rs` (row 4 + test).
Other unit files (chest_like_block_entity.rs, hopper.rs, player_action.rs,
viewer.rs) needed no change.
