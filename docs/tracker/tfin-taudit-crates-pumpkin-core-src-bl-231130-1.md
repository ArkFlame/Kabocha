# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-bl-231130.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: furnace_like_block_entity.rs

| `furnace_like_block_entity.rs:867-876` | Bedrock furnace actor stores `StoredXPInt` | hardcoded `0` | pass the accumulated `RecipesUsed` XP | S, VERIFIED (see Result) |
| `furnace_like_block_entity.rs:235-274` | `recipe.assemble` includes result data components | result stack is built from item id only | apply `recipe.result.components` to the output stack | S, VERIFIED (see Result) |
| `furnace_like_block_entity.rs:225` | `ItemStack.isSameItemSameComponents` | only item ids are compared | compare item plus components once result components are supported | S, VERIFIED (see Result) |
Verified every STILL OPEN row from the "Remaining gaps" table against decompiled vanilla

## Result
All three STILL OPEN rows are closed. The fixes were already present in the working tree; this
pass verified them against decompiled vanilla and made no source edits.

- `furnace_like_block_entity.rs:867-876` (Bedrock `StoredXPInt`) -- VERIFIED. `bedrock_block_actor_data`
  (:966-971) passes `stored_experience(&recipes)` into `build_furnace_actor`, which writes
  `StoredXPInt` (:1024); no hardcoded `0` remains. `stored_experience` (:103-109) sums
  `floor(experience * count)` per `RecipesUsed` id, matching the Java XP formula in
  `.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/AbstractFurnaceBlockEntity.java:550-555`
  (`createExperience`, `Mth.floor(amount * value)`). Test: `stored_experience_floors_each_recipes_accrued_xp`.
- `furnace_like_block_entity.rs:235-274` (result data components) -- VERIFIED. `assemble_cooking_result`
  (:86-100) builds `ItemStack::new(recipe.result.count, item)` and patches every
  `recipe.result.components` entry; `can_accept_recipe_output` (:298) and `craft_recipe` (:327) use it.
  Vanilla `SingleItemRecipe.assemble` returns `this.result.create()` from the `ItemStackTemplate`
  (`.vendor/vanilla-decompiled/net/minecraft/world/item/crafting/SingleItemRecipe.java:58-60`;
  template carried by `AbstractCookingRecipe.java:21-33`). Test:
  `assemble_cooking_result_applies_recipe_components`.
- `furnace_like_block_entity.rs:225` (`ItemStack.isSameItemSameComponents`) -- VERIFIED.
  `can_accept_recipe_output` compares the output slot with the assembled stack via
  `are_items_and_components_equal` (:299) and `craft_recipe` does the same (:331), matching vanilla
  `canBurn` (`AbstractFurnaceBlockEntity.java:358-371`) and `ItemStack.isSameItemSameComponents`
  (`.vendor/vanilla-decompiled/net/minecraft/world/item/ItemStack.java:848-850`). The comparison
  excludes count, as vanilla does. Covered by the `pumpkin-data` `are_items_and_components_equal`
  tests plus `assemble_cooking_result_applies_recipe_components`.

No compile or test run (worker rule); compile verification is the orchestrator batch check.
