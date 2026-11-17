# Tracker: audit / crates/pumpkin-core/src/block/entities/furnace.rs

Deep audit of the furnace block entity. `furnace.rs` itself only holds the
struct plus the cooking macros, so the behaviour lives in the macros in
`crates/pumpkin-core/src/block/entities/furnace_like_block_entity.rs`, which is
shared with blast furnace and smoker. Vanilla reference:
`AbstractFurnaceBlockEntity` (1.21.x codec/NBT + `serverTick`),
`FurnaceResultSlot`, `FurnaceFuelSlot`, `AbstractFurnaceMenu`.

## Markers

The two "vanilla" hits in `furnace.rs` (lines 34-39) are doc comments on
`recipes_used`/`experience`, not TODOs; both were inaccurate and were corrected
in this pass (recipe ID, not result item ID; vanilla has no `experience` field).

## Audit result

`serverTick` order, fuel consumption, cooking-progress reset/decrement, lit
block-state update, `canBurn`/`canPlaceItem`/`canTakeItemThroughFace` slot rules,
property-delegate indices and the NBT keys (`cooking_time_spent`,
`cooking_total_time`, `lit_time_remaining`, `lit_total_time`, `RecipesUsed`)
match vanilla. Remaining gaps below.

## Remaining gaps

| Gap | Rust | Vanilla | Suggested fix | Size |
|---|---|---|---|---|
| XP is accumulated as one float across all recipes and floored once at extraction, so rounding differs from vanilla | `furnace_like_block_entity.rs:121-145` | `AbstractFurnaceBlockEntity.getRecipesToAwardAndPopExperience` -> `createExperience`: per recipe `i = Mth.floor(count * experience)`, then `if (Mth.frac(...) != 0 && Math.random() < frac) ++i`, orbs awarded per recipe | Iterate `recipes_used`, floor + probabilistic round per recipe (needs RNG in extraction; drop the `experience` field and its `from_nbt` reconstruction). Note: commits `6b488148c`/`13f31276a` deliberately introduced the fraction carry, so confirm the design call first | M |
| Taking output never unlocks the used recipes in the player's recipe book nor fires `triggerRecipeCrafted` | `pumpkin-inventory/src/furnace_like/furnace_like_slot.rs:140-159`, `furnace_like_block_entity.rs:131-145` | `FurnaceResultSlot.checkTakeAchievements` -> `AbstractFurnaceBlockEntity.awardUsedRecipesAndPopExperience` -> `player.awardRecipes(list)` + `triggerRecipeCrafted` | Add a recipe-unlock method to `InventoryPlayer` and call it with the recipes from `RecipesUsed` before clearing | M |
| Fuel with a crafting remainder and a stack size > 1 spawns the remainder as an item entity and keeps the rest of the stack | `furnace_like_block_entity.rs:481-505` | `serverTick`: if `itemstack.hasCraftingRemainingItem()` -> `items.set(1, itemstack.getCraftingRemainingItem())` (whole stack replaced, leftovers discarded); no item entity | Match vanilla, or document the deliberate divergence if a Paper patch is preferred | M |
| Smelting/fuel consumption silently no-ops when `items.try_write()` fails, while `cooking_time_spent` was already reset and `is_dirty` set: lock contention can consume no fuel yet light the furnace, or lose a completed craft | `furnace_like_block_entity.rs:184-186`, `480-507`, `530-551` | single-threaded tick never skips; fuel is consumed whenever the furnace lights | Retry with a blocking `write()` (no await in `tick`) or roll the state changes back when the lock is unavailable | S |
| Fractional un-awarded experience is dropped on save/load: `from_nbt` rebuilds it only from `recipes_used` | `furnace_like_block_entity.rs:635-653` | vanilla has no stored fraction; XP is derived from `RecipesUsed` at take time, so nothing is lost | Resolved by the per-recipe XP fix above (no float state to persist) | S |
| `can_accept_recipe_output` ignores the recipe result count (`itemstack1.getCount() + itemstack.getCount() <= maxStackSize` in newer vanilla) | `furnace_like_block_entity.rs:147-178` | 1.21.2+ `canBurn` adds the result count | Only matters if a datapack cooking recipe returns more than 1 item; add the count check | S |
