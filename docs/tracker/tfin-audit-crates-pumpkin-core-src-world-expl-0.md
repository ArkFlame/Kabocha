# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-world-expl.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: block_explode.rs explosion.rs

| 2 | explosion.rs:451-470 | `level.getBlockState` loads/forces the chunk, so explosions still destroy blocks in unloaded areas; here `read_chunk_sync` returning `None` is treated as air | Fall back to `world.get_block_state` (which loads) or force-load the chunk before sampling | M. STILL OPEN (design recorded, UNVERIFIED test): unloaded chunks count as air via the sync read, documented at the sampler (`explosion.rs`); vanilla force-load needs an async prefetch before the sync tick. Prior note: M. STILL OPEN: `explode()` is sync inside the tick and both the sampler and `World::get_block_state` use the non-loading `read_chunk_sync`; a force-load needs an async prefetch before the sync tick, a cross-cutting change (not attempted) |
Re-verification 2026-11-18 (block_explode.rs slice). Checked STILL OPEN rows against
STILL OPEN rows against `.vendor/vanilla-decompiled/net/minecraft/world/level/ServerExplosion.java`
Checked the STILL OPEN rows of this unit against

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
