# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-co-7ffaa0.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: recipe.rs

- 1. No server-side recipe book (`recipe.rs:59-128`): STILL OPEN (M).
- 2. `take` remove packet always sent (`recipe.rs:130-237`): STILL OPEN (M).
- 3. Display-id layout duplicated (`recipe.rs:160`): STILL OPEN (S).
- 4. Suggestion provider ids (`recipe.rs:40-57`): STILL OPEN (S).

## Result
FINAL PASS (worker, direct; no cargo/git). All four rows re-verified against decompiled
vanilla .vendor/vanilla-decompiled/net/minecraft/server/commands/RecipeCommand.java and
.vendor/vanilla-decompiled/net/minecraft/stats/ServerRecipeBook.java.

- 1. No server-side recipe book (M): FIXED & VERIFIED. 'new_learnable_recipes' + 'give' in
  recipe.rs:53-61,161-211 add only unknown, non-special learnable ids into Player.known_recipes
  (entity/player.rs:897, persisted under recipeBook.recipes at :9452-9462 write / :9639-9651 read),
  send CRecipeBookAdd only when the added set is non-empty (:177-180), return the added count
  (:210) and throw COMMANDS_RECIPE_GIVE_FAILED when total == 0 (:183-185). Mirrors vanilla
  ServerRecipeBook.addRecipes (ServerRecipeBook.java:61-87: '!known.contains(id) && !isSpecial()',
  send only when recipesToAdd non-empty, return recipesToAdd.size()) and RecipeCommand.giveRecipes
  (:93-104, RESPONSE_GIVE failed at 0).

- 2. take remove packet always sent / wrong count (M): FIXED & VERIFIED. 'take_display_ids'
  (recipe.rs:66-89) removes only known learnable ids that have a display, 'take' sends
  CRecipeBookRemove only when removed is non-empty (:260-263), returns the removed count (:293)
  and throws COMMANDS_RECIPE_TAKE_FAILED when total == 0 (:266-268). Mirrors vanilla
  ServerRecipeBook.removeRecipes (ServerRecipeBook.java:89-106: only 'known.contains(id)', send
  only when recipesToRemove non-empty, return recipesToRemove.size()) and RESPONSE_TAKE (:28-32).
  Counts are per display entry; one display per recipe in current data makes these equal.

- 3. Display-id layout duplicated (S): FIXED & VERIFIED. 'static_recipe_display_count',
  'dynamic_recipe_has_display', 'dynamic_recipe_display_ids' (recipe_book_add.rs:71,87,98) are the
  single source of truth. The writer uses them for the entry count (:567-575) and dynamic skip
  (:664) and increments display_id only for written entries (:613-615/:659); 'write_entry'
  returns Ok(false) for CraftingSpecial/CraftingDecoratedPot (:503/:542), matching the static skip
  set. '/recipe take' uses dynamic_recipe_display_ids (recipe.rs:232). Pure tests
  dynamic_display_ids_skip_non_displayed_recipes (recipe_book_add.rs:1400) and
  take_removes_only_known_recipes_with_displays (recipe.rs:383) cover it.

- 4. Suggestion provider ids (S): FIXED & VERIFIED. 'recipe_suggestion_ids' (recipe.rs:92-106)
  suggests only non-special 'recipe_id()' manager keys and deduplicates; the provider uses it
  (:118-119). Mirrors vanilla ResourceKeyArgument.key(RECIPE, LEARNABLE_RECIPE) with
  LEARNABLE_RECIPE = r -> !r.isSpecial() (RecipeCommand.java:22,43,69). Pure test
  recipe_suggestions_are_unique_and_skip_special (recipe.rs:347).

No STILL OPEN rows. No .rs file edited this pass (all fixes/tests already present in the tree and
verified by reading), so rustfmt was not run. Compile-pitfall re-check against AGENTS.md: no new
.rs diff, so no new risk; all external refs resolve (DynamicRecipe::recipe_id -> Option<&str>,
CRecipeBookRemove::new(&[VarInt]), translation constants exist in generated/translation.rs,
player.known_recipes: Mutex<HashSet<String>>, try_send_client_packet). Compilation and the three
recipe unit tests are left to the orchestrator batch check per worker rules.
