# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-entity-pas.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: panda.rs roll.rs sneeze.rs

Verified each STILL OPEN row against decompiled vanilla `net/minecraft/world/entity/animal/panda/Panda.java`
Fresh pass against decompiled vanilla at `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/panda/Panda.java` (protocol 777). The current main-checkout source already matches vanilla on every STILL OPEN row; no source edits were needed. The note's claimed roll fix (keep rollDelta.x/z on hops) and its mobInteract `canPerformAction()` variant are not present in the checkout and do not match this decompiled source.
Every STILL OPEN row was checked line by line; no source edit is warranted in this unit's files
Resumed pass: verified all STILL OPEN rows against decompiled vanilla at `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/panda/Panda.java` (protocol 777). No source edits made to `panda.rs`, `roll.rs`, or `sneeze.rs`.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
