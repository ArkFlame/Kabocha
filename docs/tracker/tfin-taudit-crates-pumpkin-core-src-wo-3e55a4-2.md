# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-wo-3e55a4.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: mod.rs player.rs

| 7 | XS | STILL OPEN - marker/comment-only cleanup in world/mod.rs, outside this unit. |
| h3 | M | STILL OPEN - Ender dragon TODO (world/mod.rs:5978), outside this unit; needs dragon-fight rewrite. |
Re-verified every STILL OPEN row against `.vendor/vanilla-decompiled/net/minecraft/` (`LevelChunk.java:389-430`, `ServerChunkCache.java:967-990`, `ServerPlayer.java:3003-3035`, `ChunkMap.java:766-800`) and the current Pumpkin `world/mod.rs` / `entity/player.rs`. This unit's editable basenames are `none`, so no source file was changed; each fix below is already present in the checkout (landed by sibling units) and needs no in-unit seam. This section supersedes the earlier Result below.
Re-verified every STILL OPEN row against `.vendor/vanilla-decompiled/net/minecraft/` and the current `world/mod.rs`. This unit has zero editable basenames (`none`), so no source file was changed; each fix is already present in the checkout (landed by sibling units) and needs no in-unit seam.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
