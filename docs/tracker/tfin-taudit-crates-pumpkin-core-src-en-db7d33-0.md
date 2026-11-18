# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-db7d33.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: tnt.rs

Verified against decompiled `.vendor/vanilla-decompiled/net/minecraft/world/entity/item/PrimedTnt.java` and `world/level/block/TntBlock.java` (Paper/CraftBukkit, protocol V_26_3). No source files changed: every STILL OPEN row is either a stale-premise false positive or outside this unit.
Out-of-scope finding (row 4 is marked FIXED, not STILL OPEN, so left untouched): decompiled `readAdditionalSaveData` reads `fuse` as `input.getShortOr("fuse", (short)80)`, i.e. a missing tag defaults to 80; `fuse_from_nbt` in `tnt.rs` currently returns 0. Flagged for orchestrator decision.
Re-checked every STILL OPEN row against the decompiled `.vendor/vanilla-decompiled/net/minecraft/world/entity/item/PrimedTnt.java` and `world/level/block/TntBlock.java` (Paper/CraftBukkit). No source changes were needed: each applicable fix is already present in `tnt.rs`.
Fresh verification against `.vendor/vanilla-decompiled/net/minecraft/world/entity/item/PrimedTnt.java` (Paper/CraftBukkit) and the live sources. No `.rs` file needed a change: each STILL OPEN row is a false positive, already fixed in current source, or outside this unit.

## Result
VERIFIED (PrimedTnt/TntBlock vanilla paths above): each row is a false positive, already fixed, or out of unit. No BLOCKED-DATA. Flag for orchestrator: fuse_from_nbt in tnt.rs returns 0 for a missing tag, vanilla default is 80 (not fixed here).
