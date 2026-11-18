# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-f12c04.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: block_effects.rs

- row 22 (S, STILL OPEN): verified against decompiled Entity.addMovementThisTick (Entity.java:1538-1547) and confirmed already FIXED by a prior pass in block_effects.rs (merge_oldest_movement at :136-144 plus test at :580), outside this unit. No change here.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
