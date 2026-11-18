# Final rows from docs/tracker/tblocks-chests.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- Other S rows (partner on state change, synced viewer count width, custom-name labels): CLOSED in the Result below (rows 1, 3, 4).
2026-11-17. The h2-pass rows (1, 3, 4) were each checked; all resolved in the final Result below.
- **Row 3 (lid event viewer count truncated to u8): VERIFIED (matches vanilla) - not a gap.** See final Result.
Re-verified every row against decompiled vanilla present this pass.

## Result

Final pass, 2026-11-18. Re-verified all four rows against decompiled vanilla under
.vendor/vanilla-decompiled/net/minecraft/. No cargo/git/shell build was run (per unit
rules). Every row has a terminal status; none remain STILL OPEN. No source file needed a
change this pass: all four were already resolved in the tree by earlier passes, and the
one formerly-open row (3) is confirmed byte-for-byte identical to vanilla on the wire.

- **Row 1 (partner half detached on non-player removal): FIXED, in tree.**
  Vanilla `ChestBlock.updateShape`
  (.vendor/vanilla-decompiled/net/minecraft/world/level/block/ChestBlock.java:172-200)
  flips a lone half back to `ChestType.SINGLE` via the else-if at :195-197 when its
  connected neighbour changes, while `affectNeighborsAfterRemoval` (:265-267) only calls
  `Containers.updateNeighboursAfterDestroy` (comparator updates). Pumpkin's
  `chest_state_for_neighbor_update` (chests.rs:623) is wired as
  `get_state_for_neighbor_update` on all three chest blocks (chests.rs:670/854/926) and
  runs on every state change, covering break/explosion/setblock. No `on_state_replaced`
  needed. No change.
- **Row 2 (blocked-chest predicate): FIXED, in tree.**
  Vanilla `ChestBlock.isBlockedChestByBlock`
  (ChestBlock.java:368-370) tests the block above with `isRedstoneConductor` (a
  collision-full block), so slabs/stairs/walls/leaves do not block. Pumpkin's
  `has_block_on_top` uses `state.is_full_cube()` (chests.rs:1109), matching it. No change.
- **Row 3 (lid event viewer count truncated to u8): VERIFIED (matches vanilla), not a gap.**
  Vanilla `ChestBlockEntity.signalOpenCount` passes the int `current`
  (.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/ChestBlockEntity.java:232-234;
  `EVENT_SET_OPEN_COUNT = 1` at :29), but `ClientboundBlockEventPacket.write` emits it with
  `output.writeByte(this.b1)` and decodes with `readUnsignedByte`
  (.vendor/vanilla-decompiled/net/minecraft/network/protocol/game/ClientboundBlockEventPacket.java:28-39),
  so only the low 8 bits reach the wire. Pumpkin's `new as u8`
  (chest_like_block_entity.rs:346, with a documenting comment at :343-345) plus the
  protocol's u8 `action_parameter` (block_event.rs:21,52, `write_u8`) produces
  byte-identical output. Widening to `i32` would change nothing on the wire and would need
  a cross-crate/protocol refactor outside this unit, so leaving the cast is correct. No change.
- **Row 4 (double-chest menu title): FIXED, in tree.**
  Vanilla `ChestBlock.MENU_PROVIDER_COMBINER.getDisplayName`
  (ChestBlock.java:129-136) uses the first (RIGHT) half's custom name, else the second
  (LEFT) half's, else `container.chestDouble` ("Large Chest"); single chests use
  `container.chest`. Pumpkin's `chest_display_name` (chests.rs:81) resolves the custom name
  and picks the default key by `double_chest`; pure unit test
  `chest_menu_title_matches_vanilla` (chests.rs:1144) present. No change.

All four rows terminal: FIXED (1, 2, 4) and VERIFIED (3). No BLOCKED-DATA row; no
generated pumpkin-data field or missing subsystem blocks any row. No source file or
rustfmt needed; only this Result section was closed.
