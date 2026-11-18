# Final rows from docs/tracker/tblocks-chests.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

run. Every row has a terminal status; none remain STILL OPEN. No source file

## Result
Final pass, 2026-11-18, definitive. All four rows from
docs/tracker/tblocks-chests.md are terminal; none remain STILL OPEN. No
decompiled .java files were present under .vendor/vanilla-decompiled/ this
pass, so verification used the vanilla paths and line numbers recorded in
earlier passes plus a re-read of the current Pumpkin tree. No cargo/git/shell
build was run. No source file needed a change: every row was already resolved
in the tree.

- Row 1 (partner half detached on non-player removal): FIXED, in tree.
  Vanilla ChestBlock.updateShape
  (.vendor/vanilla-decompiled/net/minecraft/world/level/block/ChestBlock.java:172-200)
  flips a lone half back to ChestType.SINGLE when its connected neighbour
  changes, while affectNeighborsAfterRemoval (:265-267) only updates
  comparators. Pumpkin's chest_state_for_neighbor_update (chests.rs:623) is
  wired as get_state_for_neighbor_update on all three chest blocks
  (chests.rs:670/854/926), so break/explosion/setblock are covered. No
  on_state_replaced needed. No change.
- Row 2 (blocked-chest predicate): FIXED, in tree. Vanilla
  ChestBlock.isBlockedChestByBlock (ChestBlock.java:368-370) tests the block
  above with isRedstoneConductor (a collision-full block). Pumpkin's
  has_block_on_top uses state.is_full_cube() (chests.rs:1104-1109). No change.
- Row 3 (lid event viewer count truncated to u8): VERIFIED (matches vanilla),
  not a gap. Vanilla ChestBlockEntity.signalOpenCount passes the int current
  (.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/ChestBlockEntity.java:232-234),
  but ClientboundBlockEventPacket.write emits it with output.writeByte(this.b1)
  (.vendor/vanilla-decompiled/net/minecraft/network/protocol/game/ClientboundBlockEventPacket.java:28-39),
  so only the low 8 bits reach the wire. Pumpkin's `new as u8`
  (chest_like_block_entity.rs:346, ender_chest.rs:86) plus the protocol's u8
  action_parameter (block_event.rs:21,52, write_u8) produce byte-identical
  output; widening to i32 would change nothing on the wire. Documenting
  comments present at both cast sites. No change.
- Row 4 (double-chest menu title): FIXED, in tree. Vanilla
  MENU_PROVIDER_COMBINER.getDisplayName (ChestBlock.java:129-136) uses the
  first (RIGHT) half's custom name, else the second (LEFT) half's, else
  container.chestDouble. Pumpkin's chest_display_name (chests.rs:81) resolves
  both halves (chests.rs:487-493) and picks the default key by double_chest;
  pure unit test chest_menu_title_matches_vanilla (chests.rs:1144) present.
  No change.

No BLOCKED-DATA row: no generated pumpkin-data field or subsystem is missing.
No source file changed and no rustfmt needed this pass; only this Result
section was written.
