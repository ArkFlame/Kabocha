# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-co-7ffaa0.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

Rows 1-5: FIXED and VERIFIED against vanilla (see Result).

## Result

Final pass. Re-read all three source files in the main checkout and verified each row against
decompiled vanilla at .vendor/vanilla-decompiled/net/minecraft/server/commands/RecipeCommand.java
and .vendor/vanilla-decompiled/net/minecraft/stats/ServerRecipeBook.java. All five rows were already
resolved in the tree on arrival; no .rs edit or rustfmt was required this pass.

- 1. No server-side recipe book (recipe.rs:59-128): VERIFIED FIXED. `Player.known_recipes`
  (entity/player.rs:897, persisted under `recipeBook.recipes` NBT at :9452-9462 write and
  :9639-9643 read) backs give. `new_learnable_recipes`/`learnable_recipe_id` add only unknown,
  non-special ids, `CRecipeBookAdd` is sent only when `added` is non-empty, the return value is the
  added count, and `COMMANDS_RECIPE_GIVE_FAILED` is thrown at total == 0
  (recipe.rs:163-185). Matches vanilla `ServerRecipeBook.addRecipes` (:61-87: `!known &&
  !isSpecial`, send only when `recipesToAdd` non-empty, return `recipesToAdd.size()`).

- 2. take remove packet always sent / wrong count (recipe.rs:130-237): VERIFIED FIXED.
  `take_display_ids` (recipe.rs:66-89) removes only known, learnable ids that have a display;
  `CRecipeBookRemove` is sent only when `ids_to_remove` is non-empty (:260-263); the return value
  is the actually-removed count and `COMMANDS_RECIPE_TAKE_FAILED` is thrown at total == 0
  (:266-268). Matches vanilla `ServerRecipeBook.removeRecipes` (:89-106: `known.contains`,
  send only when `recipesToRemove` non-empty, return `recipesToRemove.size()`).

- 3. Display-id layout duplicated (recipe.rs:160): VERIFIED FIXED. `static_recipe_display_count`
  (recipe_book_add.rs:71), `dynamic_recipe_has_display` (:87) and `dynamic_recipe_display_ids`
  (:98) are the single source for the writer's entry count/skip/increment (:570, :664) and for
  `/recipe take` (recipe.rs:232). `write_entry` returns `Ok(false)` for special/decorated-pot
  (:503), aligned with `dynamic_recipe_has_display`. Pure test
  `dynamic_display_ids_skip_non_displayed_recipes` (recipe_book_add.rs:1373).

- 4. Suggestion provider ids (recipe.rs:40-57): VERIFIED FIXED. `recipe_suggestion_ids`
  (recipe.rs:92-106) suggests only `recipe_id()` manager keys, skips `SpecialCrafting`, dedupes via
  `HashSet`; the provider uses it (:118). Matches vanilla `ResourceKeyArgument.key(RECIPE,
  LEARNABLE_RECIPE)` with `LEARNABLE_RECIPE = r -> !r.isSpecial()`
  (RecipeCommand.java:22,43,69). Pure test `recipe_suggestions_are_unique_and_skip_special`
  (recipe.rs:347).

- 5. Raw argument in error text (recipe.rs:24-25): VERIFIED FIXED, no change needed. The parsed
  `Identifier` (`context.get_argument::<Identifier>("recipe")?.to_string()`, always namespace:path)
  is passed to `recipe.notFound` (:136, :224), matching vanilla's parsed ResourceLocation echo via
  `ResourceKeyArgument.getRecipe` (RecipeCommand.java:48,74).

Parity fix already present from the fifth pass and re-verified: both executors resolve the argument
through `recipe_matches` (recipe.rs:40-42, used at :145 and :237), which requires key equality only,
matching vanilla `RecipeManager` key resolution; the keyless-recipe fallback to the result item id
is gone. Pure test `recipe_matching_uses_manager_keys_only` (recipe.rs:444).

Compile-pitfall re-check against AGENTS.md: no new .rs diff this pass, so no new risk. Existing code
uses `unwrap_or_else(PoisonError::into_inner)` instead of unwrap, `#[must_use]`/`const fn` where
applicable, full paths in tests, `tracing`-free, and `pub use recipe_book_add::*` re-export so
recipe.rs imports resolve. No unwrap/expect/panic, no println, no em dashes.

No rows remain open. No .rs file edited, so rustfmt was not run; compilation and the three recipe
unit tests are deferred to the orchestrator batch check per worker rules.
