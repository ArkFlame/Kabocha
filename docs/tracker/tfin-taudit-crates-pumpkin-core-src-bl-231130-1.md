# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-bl-231130.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: furnace_like_block_entity.rs

| `furnace_like_block_entity.rs:867-876` | Bedrock furnace actor stores `StoredXPInt` | hardcoded `0` | pass the accumulated `RecipesUsed` XP | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `furnace_like_block_entity.rs:235-274` | `recipe.assemble` includes result data components | result stack is built from item id only | apply `recipe.result.components` to the output stack | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `furnace_like_block_entity.rs:225` | `ItemStack.isSameItemSameComponents` | only item ids are compared | compare item plus components once result components are supported | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
Verified every STILL OPEN row from the "Remaining gaps" table against decompiled vanilla

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
