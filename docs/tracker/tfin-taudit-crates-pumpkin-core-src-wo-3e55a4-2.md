# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-wo-3e55a4.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: mod.rs player.rs

| 7 | XS | STILL OPEN - marker/comment-only cleanup in world/mod.rs, outside this unit. |
| h3 | M | STILL OPEN - Ender dragon TODO (world/mod.rs:5978), outside this unit; needs dragon-fight rewrite. |
Re-verified every STILL OPEN row against `.vendor/vanilla-decompiled/net/minecraft/` (`LevelChunk.java:389-430`, `ServerChunkCache.java:967-990`, `ServerPlayer.java:3003-3035`, `ChunkMap.java:766-800`) and the current Pumpkin `world/mod.rs` / `entity/player.rs`. This unit's editable basenames are `none`, so no source file was changed; each fix below is already present in the checkout (landed by sibling units) and needs no in-unit seam. This section supersedes the earlier Result below.
Re-verified every STILL OPEN row against `.vendor/vanilla-decompiled/net/minecraft/` and the current `world/mod.rs`. This unit has zero editable basenames (`none`), so no source file was changed; each fix is already present in the checkout (landed by sibling units) and needs no in-unit seam.

## Result
FINAL PASS: both rows verified against `.vendor/vanilla-decompiled/net/minecraft/` and the current `world/mod.rs`. Both rows are already resolved in this checkout; no source change or generated data is required.

| # | Size | Result |
|---|---|---|
| 7 | XS | VERIFIED (no behaviour gap) - row 7 is marker/comment-only. View/simulation distance in `CLogin::new` (world/mod.rs:4451-4475) come from config and match the `chunkRadius`/`simulationDistance` fields of vanilla `ClientboundLoginPacket` (`network/protocol/game/ClientboundLoginPacket.java:14-27`). The only surviving TODOs (:4962 structured logging, :5020 world spawn compass) are comment-only style notes; no behaviour gap. |
| h3 | M | FIXED/VERIFIED - the `TODO: Implement ender dragon handling` marker is gone. Dragon handling is wired through `dragon_fight::DragonFight::tick` (world/mod.rs:2421-2423), the `dragon_fight` field (:414) loads/saves NBT (:660-689, :9223-9227), and dragon parts are merged into entity queries (:5826-5917), matching vanilla `EnderDragonFight.tick()` (`world/level/dimension/end/EnderDragonFight.java:184`) and `EnderDragonPart` (`world/entity/boss/enderdragon/EnderDragonPart.java`). |

No `.rs` file was edited: both rows are already fixed / have no behaviour gap in the current checkout, so there is no in-unit seam to change and no unit test to add (deleting a marker is churn; the dragon path is existing engine code). rustfmt not run (no source file touched).
