# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-world-expl.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: block_explode.rs explosion.rs

| 2 | explosion.rs:451-470 | `level.getBlockState` loads/forces the chunk, so explosions still destroy blocks in unloaded areas; here `read_chunk_sync` returning `None` is treated as air | Fall back to `world.get_block_state` (which loads) or force-load the chunk before sampling | FIXED / VERIFIED (matches decompiled vanilla force-load semantics; see Result below) |

## Result
Row 2 (explosion.rs block-destruction sampler): FIXED / VERIFIED.
Vanilla `.vendor/vanilla-decompiled/net/minecraft/world/level/ServerExplosion.java`
method `getOrCacheExplosionBlock` (lines 100-137) resolves each ray block's chunk
through `this.level.getChunk(x >> 4, z >> 4)` at line 119, force-loading a chunk the
explosion ray enters before reading the block state at line 122; `calculateExplodedPositions`
(line 371) uses that cache for block destruction. Pumpkin now mirrors this in
`get_blocks_to_destroy`: each ray block position resolves its chunk through the
`load_chunk_for_explosion` helper, which calls `Level::get_or_fetch_chunk` (the
force-loading path) via the runtime handle using `block_in_place` + `block_on`,
cached per chunk in `chunk_cache`. `read_chunk_sync` returning `None` is no longer
accepted as air for destruction sampling. Added unit test
`explosion_force_loads_unloaded_chunks`. No generated pumpkin-data fields, WIT changes or
cross-crate refactors required; no rows remain open.

Verification (final pass, 2026-11-18): re-read
`.vendor/vanilla-decompiled/net/minecraft/world/level/ServerExplosion.java` lines
100-137 and confirmed the `getChunk` force-load at line 119. Re-read the Pumpkin
implementation and checked every touched signature: `read_chunk_sync`/`get_or_fetch_chunk`
both take `F: Fn(&SyncChunk) -> R` with `SyncChunk = Arc<ChunkData>`, so
`Clone::clone` yields `Arc<ChunkData>` as required; `get_block_absolute_y(usize, i32, usize)`
returns `Option<BlockStateId>`; `rt_handle: OnceLock<Handle>`; and the test's
`ChunkData::empty_sync` returns `Arc<ChunkData>` matching `process_new_chunk` and
`loaded_chunks`. Helper lives at module scope, uses `tracing::debug!` (no stdout),
avoids `unwrap`/`expect`/`panic`, and the test returns `Result`. No new compile-pitfall
exposures. No source edits were needed this pass: the fix landed in an earlier pass.
