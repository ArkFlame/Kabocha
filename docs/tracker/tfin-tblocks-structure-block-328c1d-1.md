# Final rows from docs/tracker/tblocks-structure-block-328c1d.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: set_structure_block.rs

| net/java/play/set_structure_block.rs:134-137 | structure.setChanged(); level.sendBlockUpdated(pos, state, state, UPDATE_ALL) (ServerGamePacketListenerImpl.java:1317-1318) | Only mark_dirty + register_block_change (client notify); no neighbour update | Use an update-all helper when the world API offers one | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
- Structure block rows: STILL OPEN, not attempted this pass (see tblocks-structure-block.md h6 status).
Re-verified every STILL OPEN row against .vendor/vanilla-decompiled/net/minecraft/
Re-verified every STILL OPEN row against .vendor/vanilla-decompiled/net/minecraft (1.21.x Paper/Canvas tree).

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
