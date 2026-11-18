# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-db7d33.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: tnt.rs

Re-checked every STILL OPEN row against `.vendor/vanilla-decompiled/net/minecraft/world/entity/item/PrimedTnt.java` (Paper/CraftBukkit, protocol V_26_3) and `world/level/block/TntBlock.java`, plus the live `crates/pumpkin-core/src/entity/tnt.rs` and `block/blocks/tnt.rs`. No `.rs` file needed a change in this unit: each row is a false positive, already fixed in current source, or outside this unit.
Re-verified every STILL OPEN row against `.vendor/vanilla-decompiled/net/minecraft/world/entity/item/PrimedTnt.java` and `world/level/block/TntBlock.java` (Paper/CraftBukkit), and re-read the live `entity/tnt.rs` and `block/blocks/tnt.rs`. No `.rs` change was needed: every row is a false positive, already fixed, or needs a missing subsystem.
Every STILL OPEN row was re-verified against `.vendor/vanilla-decompiled/net/minecraft/world/entity/item/PrimedTnt.java` and `world/level/block/TntBlock.java` (Paper/CraftBukkit, protocol V_26_3) and the live `entity/tnt.rs` and `block/blocks/tnt.rs`. No row remains STILL OPEN. No `.rs` change was needed for rows 5, 6, 7, 8.
Re-verified every STILL OPEN row against `.vendor/vanilla-decompiled/net/minecraft/world/entity/item/PrimedTnt.java` (Paper/CraftBukkit, protocol V_26_3) and `world/level/block/TntBlock.java`. Row 3 was implemented across the three files the fix needed; no row remains STILL OPEN.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
