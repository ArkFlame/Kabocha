# Final rows from docs/tracker/tsmall-batch2.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: furnace_like_block_entity.rs mod.rs

| taudit-crates-pumpkin-core-src-bl-4d09af : 63 (`block/mod.rs:667-673`) | VERIFIED | Zero `water_source()` call sites remain under `crates/pumpkin-core/src` (grep). Source-only `is_water()` at `block/mod.rs:793` with test `is_water_matches_only_source_water`. Vanilla `world/level/material/Fluids.java:9-10`. |
| taudit-crates-pumpkin-core-src-bl-689b5a : 47 (`furnace_like_block_entity.rs:520-530`) | FIXED (already at HEAD) | Relight rescale `rescale_cooking_after_relight` (:46-62), called at :622. Vanilla `AbstractFurnaceBlockEntity.java:278-281`. |
| taudit-crates-pumpkin-core-src-bl-689b5a : 91 (`furnace_like_block_entity.rs:535-560`) | FIXED (already at HEAD) | Same helper and call as bl-689b5a:47. Vanilla `AbstractFurnaceBlockEntity.java:278-281`. |
| taudit-crates-pumpkin-core-src-bl-689b5a : 92 (`furnace_like_block_entity.rs:252-275`) | FIXED (already at HEAD) | `assemble_cooking_result` (:86-100) applies result components (:298, :327). Vanilla `AbstractFurnaceBlockEntity.java:358-371`. |

## Result
All four rows closed: 63 VERIFIED, 47, 91 and 92 FIXED at HEAD. No row remains open. No `.rs` file was edited this pass.
