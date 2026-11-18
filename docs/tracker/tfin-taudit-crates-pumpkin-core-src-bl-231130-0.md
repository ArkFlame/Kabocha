# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-bl-231130.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: furnace_like_block_entity.rs furnace_like_slot.rs

| `furnace_like_block_entity.rs:30-38,511` | `getSpeedMultiplier` reads the fuel item's `CookingFuel.speedMultiplier` (ResolvableFloat, `cooking/speed_default`); `getTotalCookTime` divides by it | multiplier is derived from the block kind; a datapack fuel with an explicit `speed_multiplier` is ignored and the value is not restored on load | store `speed_multiplier` on the three entities, set it from `CookingFuelImpl::get_speed_multiplier(is_fast)` at ignition, use it in `cooking_total_time`, read/write the float key | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `furnace_like_block_entity.rs:511-535` | on relight with a different multiplier, `cookingTimer` is rescaled to `ceil(timer / oldTotal * newTotal)` | no rescale | implement after the data-driven multiplier above | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `furnace_like_block_entity.rs:177-198` | `getRecipesToAwardAndPopExperience` spawns one orb per recipe id | extraction returns one summed `i32`, so one orb | return per-recipe amounts (Vec) or spawn orbs in the block entity | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `pumpkin-inventory/src/furnace_like/furnace_like_slot.rs` take path | `FurnaceResultSlot.checkTakeAchievements` -> `awardUsedRecipesAndPopExperience` -> `player.awardRecipes` + `triggerRecipeCrafted` | used recipes are never unlocked in the recipe book nor counted as crafted | pass `RecipesUsed` ids to the player before clearing | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
