# Final rows from docs/tracker/tsmall-batch2.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: furnace_like_block_entity.rs mod.rs

| taudit-crates-pumpkin-core-src-bl-4d09af : 63 (`block/mod.rs:667-673`) | STILL OPEN | `BlockIsReplacing::water_source()` still has 45 call sites under `crates/pumpkin-core/src`. Migration to an `is_water()` helper is wider than this pass. |
| taudit-crates-pumpkin-core-src-bl-689b5a : 47 (`furnace_like_block_entity.rs:520-530`) | STILL OPEN | Same gap as bl-231130:46 (no relight rescale). |
| taudit-crates-pumpkin-core-src-bl-689b5a : 91 (`furnace_like_block_entity.rs:535-560`) | STILL OPEN | Same gap as bl-231130:46. |
| taudit-crates-pumpkin-core-src-bl-689b5a : 92 (`furnace_like_block_entity.rs:252-275`) | STILL OPEN | Same gap as bl-231130:50. |

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
