# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-449897.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- Resolved against decompiled `Witch.java`: item 1 (performRangedAttack speed) and item 2 (Raider goal priorities) verified as already-correct, item 3 (`can_be_leader`) override restored to false. No open rows remain; see Result.

## Result
Item 1 VERIFIED (Witch.java:69 RangedAttackGoal 1.0/60/10.0 matches witch.rs:139). Item 2 VERIFIED in the earlier pass (Raider goal priorities). Item 3 FIXED: witch.rs:488 can_be_leader false matches Witch.java:299-301.
