# Final rows from docs/tracker/tblocks-fire.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- Gap 3 (TNT replaced before priming): STILL OPEN (deliberate ordering, identical effect while TNT is stateless; S).
- Gap 4 (waterlogged neighbour odds): STILL OPEN (unchecked; S).
Re-verification 2026-11-17 against `.vendor/vanilla-decompiled/net/minecraft/world/level/block/FireBlock.java` (Paper build). Both rows still flagged STILL OPEN in the Status section were already resolved in code; no new source edits were needed this pass.
Fresh verification 2026-11-18 against `.vendor/vanilla-decompiled/net/minecraft/world/level/block/FireBlock.java` (Paper/CraftBukkit build). Both rows flagged STILL OPEN in the Status section are already resolved in the current code; no source edits were needed this pass.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
