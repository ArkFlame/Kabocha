# Final rows from docs/tracker/tsmall-batch2.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: furnace_like_block_entity.rs

| taudit-crates-pumpkin-core-src-bl-231130 : 46 (`furnace_like_block_entity.rs:511-535`) | STILL OPEN | Vanilla `AbstractFurnaceBlockEntity:278-281` rescales `cookingTimer` to `ceil(ratio * newTotal)` on relight. Rust has no rescale (no `ceil` rescale in `furnace_like_block_entity.rs`). |
| taudit-crates-pumpkin-core-src-bl-231130 : 47 (`furnace_like_block_entity.rs:177-198`) | STILL OPEN | Not re-checked in this pass; row claim not contradicted. |
| taudit-crates-pumpkin-core-src-bl-231130 : 49 (`furnace_like_block_entity.rs:867-876`) | STILL OPEN | Bedrock writer `write_bedrock_nbt` takes `stored_xp` (line ~918); caller not read, so the hardcoded `0` is not confirmed in this pass. |
| taudit-crates-pumpkin-core-src-bl-231130 : 50 (`furnace_like_block_entity.rs:235-274`) | STILL OPEN | Verified: output stack is still `ItemStack::new(recipe.result.count, output_item)` (line ~259), so `recipe.result.components` is dropped. |

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
