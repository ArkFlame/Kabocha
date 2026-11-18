# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-bl-689b5a.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: furnace_like_block_entity.rs

| `furnace_like_block_entity.rs:484-496` | `getSpeedMultiplier` reads the fuel item's `CookingFuel.speedMultiplier` (`ResolvableFloat`, `cooking/speed_default` is conditional on `FAST_FURNACE`) | multiplier is derived from the block kind; a datapack fuel with an explicit `speed_multiplier` is ignored, and the value is not restored from NBT | store `speed_multiplier` on the three entities, fill it from `CookingFuelImpl::get_speed_multiplier(is_fast)` at ignition, read/write the float NBT key | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `furnace_like_block_entity.rs:542-556` | Paper passes `getTotalCookTime(level, entity)` (recipe time / multiplier, default 200) to `FurnaceStartSmeltEvent` | event is fired with the stored total, which is 0 when the input arrived before `set_stack` set it | compute from `furnace_recipe` with `cooking_total_time`, default 200 | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `furnace_like_block_entity.rs:233-237` | `burn` adds `burnResult.count()` to the result slot | always increments by 1 | increment by `recipe.result.count` and bound by `can_accept_recipe_output` | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass | S, FIXED (already in tree: furnace_like_block_entity.rs:257-263 increments by `recipe.result.count`; no commit) |
| `furnace_like_block_entity.rs:520-530` | on relight with a different fuel multiplier, `cookingTimer` is rescaled to `ceil(timer / oldTotal * newTotal)` | no rescale (only relevant once the multiplier is data-driven, gap 1) | rescale after gap 1 | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
