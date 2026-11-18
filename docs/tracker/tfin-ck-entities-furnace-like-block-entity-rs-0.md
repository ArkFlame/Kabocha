# Final rows from docs/tracker/tpar-pumpkin-core-src-block-entities-furnace-like-block-entity-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files:

## Result (final close)

All 8 rows from the linked parity doc are closed in the current tree. The brief's
.vendor/vanilla-decompiled/net/minecraft/ tree is absent in this checkout; verification
used the decompiled Canvas/Paper reference the audit doc cites throughout:
/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft/world/level/block/entity/AbstractFurnaceBlockEntity.java

- data-driven fuel speed_multiplier (M) -- VERIFIED. Vanilla lines 66/72/209/227/421/424-431 persist
  a speed_multiplier float (default 1.0), read it from the fuel CookingFuel.speedMultiplier at
  ignition, and divide recipe time by it in getTotalCookTimeVanilla. Pumpkin persists
  speed_multiplier_bits on furnace.rs, smoker.rs and blasting_furnace.rs, reads/writes the
  speed_multiplier float key, derives cooking_total_time from it, and sets it from
  CookingFuelImpl::get_speed_multiplier(is_fast).
- rescale timer on relight (S) -- VERIFIED. Vanilla lines 274-282 rescale cookingTimer to the
  ceil of the ratio times the new total while the timer is below the old total;
  rescale_cooking_after_relight matches that guard and runs at ignition.
- one XP orb per recipe (S) -- VERIFIED. Vanilla getRecipesToAwardAndPopExperience /
  createExperience (lines 531-577) spawn one orb per RecipesUsed id. experience_per_recipe
  returns one RecipeExperience per id; block/entities/mod.rs spawns one orb per entry on
  replacement and award_experience_orbs (player.rs) does so on the take path.
- unlock recipes in recipe book (M) -- VERIFIED. Vanilla
  FurnaceResultSlot.checkTakeAchievements -> awardUsedRecipesAndPopExperience (line 520)
  -> Player.awardRecipes + ServerRecipeBook.addRecipes. furnace_like_slot.rs::on_take_item
  forwards recipe ids to player.award_recipes (screen_handler.rs trait + player.rs impl).
- Bedrock StoredXPInt (S) -- VERIFIED. build_furnace_actor is fed stored_experience(), which
  sums accrued RecipesUsed XP instead of a hardcoded 0.
- result data components (S) -- VERIFIED. assemble_cooking_result applies recipe.result.components
  to the output stack in both the can-burn and craft paths.
- isSameItemSameComponents (S) -- VERIFIED. are_items_and_components_equal is used in the
  can-burn and craft paths on the component-carrying assembled stack.
- getUpdateTag (S) -- VERIFIED. Vanilla AbstractFurnaceBlockEntity has no getUpdateTag override;
  the base empty compound is correct, and container packets carry contents.

Tests: pure unit coverage exists for each seam (cooking_time_divides_by_the_persisted_multiplier,
relight_rescales_progress_by_the_multiplier_change, furnace_speed_multiplier_round_trips_through_nbt,
experience_per_recipe_returns_one_amount_for_each_used_recipe,
assemble_cooking_result_applies_recipe_components, stored_experience_floors_each_recipes_accrued_xp).
No new tests or source edits this pass. No compile/check run (worker rule).
No PRIOR-OPEN rows remain.
