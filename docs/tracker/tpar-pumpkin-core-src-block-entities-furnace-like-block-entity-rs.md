# Parity rows for crates/pumpkin-core/src/block/entities/furnace_like_block_entity.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-bl-231130** [partial/OWN-OPEN] own audit: 8 open, 19 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-bl-231130.md) docs/tracker/taudit-crates-pumpkin-core-src-bl-231130.md

## Result
All 8 audit rows are closed in the current tree; see the final Result section below. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)


## Result (third pass / final close)

Verified all 8 "Remaining gaps" rows against decompiled Canvas/Paper vanilla
/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft/world/level/block/entity/AbstractFurnaceBlockEntity.java
(the brief's .vendor/vanilla-decompiled tree is absent in this checkout; this is the path the audit doc uses throughout).
No source edits were needed this pass: every row is already resolved in the working tree and matches vanilla.
No STILL OPEN rows remain.

- data-driven fuel speed_multiplier (M) -- VERIFIED. Vanilla lines 72,209,227,274-281,420-431 persist a
  speedMultiplier float under the speed_multiplier key, read it from the fuel CookingFuel.speedMultiplier at
  ignition, and divide recipe time by it (getTotalCookTime). Pumpkin persists speed_multiplier_bits on the
  furnace/smoker/blast-furnace entities, reads/writes the float key, derives cooking_total_time from it, and sets it
  from CookingFuelImpl::get_speed_multiplier(is_fast).
- rescale timer on relight (S) -- VERIFIED. Vanilla lines 278-282 rescale cookingTimer = ceil(ratio * newTotal)
  while timer is below oldTotal; rescale_cooking_after_relight implements that and runs at ignition.
- one XP orb per recipe (S) -- VERIFIED. Vanilla getRecipesToAwardAndPopExperience/createExperience (lines
  531-577) spawn one orb per RecipesUsed id; experience_per_recipe/extract_recipes return one RecipeExperience per
  id, block/entities/mod.rs spawns one orb per amount on replacement, and award_experience_orbs does so on the
  take path.
- unlock recipes in recipe book (M) -- VERIFIED. Vanilla FurnaceResultSlot.checkTakeAchievements ->
  awardUsedRecipesAndPopExperience -> Player.awardRecipes + ServerRecipeBook.addRecipes; furnace_like_slot.rs forwards
  extracted recipe ids to player.award_recipes, implemented in entity/player.rs (recipe book, RecipeUnlocked/
  RecipeCrafted, CRecipeBookAdd).
- Bedrock StoredXPInt (S) -- VERIFIED. stored_experience() feeds accumulated RecipesUsed XP into
  bedrock_block_actor_data instead of a hardcoded 0.
- result data components (S) -- VERIFIED. Vanilla AbstractCookingRecipe.assemble applies result components;
  assemble_cooking_result patches them into the output stack in both can-burn and craft paths.
- isSameItemSameComponents (S) -- VERIFIED. are_items_and_components_equal is used in can-burn and craft paths
  on the component-carrying assembled stack.
- getUpdateTag (S) -- VERIFIED. Vanilla AbstractFurnaceBlockEntity does not override getUpdateTag, so the base
  empty compound is correct; container packets carry contents.

Tests: pure unit coverage already exists for each seam (cooking_time_divides_by_the_persisted_multiplier,
relight_rescales_progress_by_the_multiplier_change, furnace_speed_multiplier_round_trips_through_nbt,
experience_per_recipe_returns_one_amount_for_each_used_recipe, assemble_cooking_result_applies_recipe_components,
stored_experience_floors_each_recipes_accrued_xp). No new tests needed. No compile/check run (worker rule).
