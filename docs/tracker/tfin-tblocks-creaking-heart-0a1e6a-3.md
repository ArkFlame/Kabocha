# Final rows from docs/tracker/tblocks-creaking-heart-0a1e6a.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

| 5 hurt particle / `setChanged` | STILL OPEN (client-only, out of unit) | Vanilla `creakingHurt` only emits client particles and seeds an emitter; no server divergence. |
Re-checked the three STILL OPEN rows against `.vendor/vanilla-decompiled`. One small in-unit parity
Re-checked every previously STILL OPEN row against `.vendor/vanilla-decompiled/net/minecraft/`.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
