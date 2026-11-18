# Final rows from docs/tracker/tblocks-chests.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

worker. Every row now has a terminal status; none remain STILL OPEN.
All four rows resolved; no row remains STILL OPEN. Only documenting comments were
run (per unit rules). Every row has a terminal status; no row remains STILL OPEN.
All four rows closed; none STILL OPEN. No generated pumpkin-data field or

## Result
FINAL PASS 2026-11-18. Re-verified all four rows against decompiled vanilla present
under .vendor/vanilla-decompiled/net/minecraft/. No cargo/git/build command was run.
Every row has a terminal status; none remain STILL OPEN. No source file needed a
change this pass: all four were already resolved in the tree by earlier passes.

- Row 1 (partner half detached on non-player removal): FIXED, in tree.
  Vanilla ChestBlock.updateShape
  (.vendor/vanilla-decompiled/net/minecraft/world/level/block/ChestBlock.java:187-199)
  flips a lone half back to ChestType.SINGLE via the else-if at :195-196;
  affectNeighborsAfterRemoval (:265-267) only calls
  Containers.updateNeighboursAfterDestroy (comparator updates). Pumpkin's
  chest_state_for_neighbor_update (chests.rs:623-656), wired as
  get_state_for_neighbor_update on all three chest blocks (:670/854/926), is a
  faithful port that runs on every state change, covering break/explosion/setblock.
  No on_state_replaced needed. No code change.
- Row 2 (blocked-chest predicate): FIXED, in tree.
  Vanilla isBlockedChestByBlock (ChestBlock.java:368-370) tests the block above
  with isRedstoneConductor (a collision-full block). Pumpkin's has_block_on_top
  uses state.is_full_cube() (chests.rs:1104-1110). No code change.
- Row 3 (lid event viewer count truncated to u8): VERIFIED (matches vanilla).
  Vanilla ChestBlockEntity.signalOpenCount passes the int current
  (.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/ChestBlockEntity.java:232-234;
  EVENT_SET_OPEN_COUNT=1 at :29), but ClientboundBlockEventPacket.write emits
  output.writeByte(this.b1) and decodes with readUnsignedByte
  (.vendor/vanilla-decompiled/net/minecraft/network/protocol/game/ClientboundBlockEventPacket.java:28-39),
  so only the low 8 bits reach the wire. Pumpkin's `new as u8`
  (chest_like_block_entity.rs:346, ender_chest.rs:86) plus the protocol u8
  action_parameter writes byte-identical output; widening to i32 would change
  nothing on the wire and needs a protocol change outside this unit. The
  documenting comments at both cast sites are present. No code change.
- Row 4 (double-chest menu title): FIXED, in tree.
  Vanilla MENU_PROVIDER_COMBINER.getDisplayName
  (.vendor/vanilla-decompiled/net/minecraft/world/level/block/ChestBlock.java:130-135)
  uses the first (RIGHT) half custom name, else the second (LEFT) half, else
  container.chestDouble. Pumpkin resolves both halves (chests.rs:491-493) and
  chest_display_name (chests.rs:81-94) picks the default key by double_chest;
  pure unit test chest_menu_title_matches_vanilla (chests.rs:1143-1166) present.
  No code change.

All four rows terminal: FIXED (1,2,4) and VERIFIED (3). No BLOCKED-DATA row; no
generated pumpkin-data field or subsystem is missing. No rustfmt needed.
