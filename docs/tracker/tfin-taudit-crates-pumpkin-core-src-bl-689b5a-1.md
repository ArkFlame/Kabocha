# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-bl-689b5a.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: furnace_like_block_entity.rs

| `furnace_like_block_entity.rs:510-521,599-610` | `getSpeedMultiplier(level, fuel)` reads the fuel item's `CookingFuel.speedMultiplier` (`ResolvableFloat`, `cooking/speed_default` conditional on `FAST_FURNACE`), then `getTotalCookTime = ceil(recipeTime / speedMultiplier)` | the 2x multiplier is hardcoded per recipe kind; a datapack fuel with an explicit `speed_multiplier` is ignored, and the saved `speed_multiplier` float is written but never restored | resolve and store `speed_multiplier` per fuel ignition (`CookingFuelImpl::get_speed_multiplier`), use it for `cooking_total_time`, read the float in `from_nbt` | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `furnace_like_block_entity.rs:535-560` | on (re)ignition with a different multiplier, `cookingTimer` is rescaled to `ceil(timer / oldTotal * newTotal)` | no rescale (only observable once the multiplier is data-driven) | rescale after the data-driven multiplier lands | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `furnace_like_block_entity.rs:252-275` | `canBurn`/`burn` use `recipe.assemble(input)`, which includes the recipe result's data components | the output stack is built from `result.id`/`result.count` only; `recipe.result.components` is dropped | build the result stack with the recipe result components | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `furnace_like_block_entity.rs:521-533` | Paper `FurnaceBurnEvent#setConsumeFuel(false)` keeps the furnace lit without consuming the fuel item | the event has no `consume_fuel` field | add the field to the event (additive plugin API) and honour it | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
