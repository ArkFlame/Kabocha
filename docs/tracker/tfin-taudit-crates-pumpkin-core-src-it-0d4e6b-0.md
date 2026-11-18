# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-it-0d4e6b.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: bucket.rs

- 1 waterlogged pickup can_survive (bucket.rs:309-318): CLOSED, see Result.
- 2 sneak waterlogging (bucket.rs:423): CLOSED, see Result (proposed removal rejected).
- 3 ultrawarm mob bucket spawns mob (bucket.rs:540-577): CLOSED, see Result.
- 4 no mayInteract / spawn protection (bucket.rs:291-338): CLOSED, see Result.

## Result
Final pass (2026-11-18). All four rows re-verified against the decompiled 1.21 source under .vendor/vanilla-decompiled/net/minecraft/ and the current Pumpkin code in crates/pumpkin-core/src/item/items/bucket.rs. Every row was already fixed by earlier passes; no new source edits were needed this pass. No cargo/git run (forbidden for this worker); rustfmt --edition 2024 on bucket.rs was run by the orchestrator in the prior pass. No generated pumpkin-data field or whole subsystem is missing, so no BLOCKED-DATA rows remain.

- Row 1 (waterlogged pickup can_survive, bucket.rs:311-334): VERIFIED. Vanilla .vendor/vanilla-decompiled/net/minecraft/world/level/block/SimpleWaterloggedBlock.java:39-50 (pickupBlock) sets WATERLOGGED=false, then destroyBlock(pos, true) when the state can no longer survive. Pumpkin's set_waterlogged(false) + block_registry.can_place_at on the dried state + break_block with drops (no SKIP_DROPS) reproduces this. Correct.

- Row 2 (sneak waterlogging, bucket.rs:442-449): VERIFIED, no change. Vanilla .vendor/vanilla-decompiled/net/minecraft/world/item/BucketItem.java:146 gates canPlaceFluidInsideBlock on (!shiftKeyDown || hitResult == null), so sneaking recurses to the adjacent position (emptyContents:190) instead of waterlogging the clicked block. Pumpkin's `is_water && !is_sneaking && is_liquid_block_container(block)` already matches; the tracker's proposed removal of !is_sneaking would be WRONG. Keep.

- Row 3 (ultrawarm mob bucket, bucket.rs:554-599): VERIFIED. Vanilla .vendor/vanilla-decompiled/net/minecraft/world/item/BucketItem.java:158-170 takes the fire-extinguish branch and returns true WITHOUT playEmptySound, then use():64 still runs checkExtraContent and .vendor/vanilla-decompiled/net/minecraft/world/item/MobBucketItem.java:33-39 spawns the mob. Pumpkin skips the extra empty sound on evaporation (bucket.rs:555-568) and still spawns the mob (bucket.rs:570-599), with the evaporation sound coming from try_place_liquid. Correct; the doc's "no spawn" premise was wrong.

- Row 4 (mayInteract / spawn protection, bucket.rs:661-664 and 750-753): VERIFIED. Vanilla .vendor/vanilla-decompiled/net/minecraft/world/item/BucketItem.java:60 gates use() on level.mayInteract(player, pos). Pumpkin's world.is_in_spawn_protection gates (helper crates/pumpkin-core/src/world/mod.rs:1027: overworld + permission level + radius + has-ops checks) match. player.mayUseItemAt is region/spectator-only and is covered upstream/by the plugin events. Correct.

Compile-pitfall re-check: this pass touched no Rust, so no new imports, helper placements, macro paths, struct initializers, clones, or clippy-pedantic issues were introduced. No new pure unit-test seam exists: all four behaviours require a live World; the existing block_face_names_match_bukkit test (bucket.rs:918) is the file's only pure test and is unchanged.

All four rows end FIXED/VERIFIED. Nothing open remains in this unit.
