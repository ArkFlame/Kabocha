# Parity rows for crates/pumpkin-core/src/command/commands/recipe.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-co-7ffaa0** [partial/OWN-OPEN] own audit: 10 open, 3 fixed, 21 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-co-7ffaa0.md) docs/tracker/taudit-crates-pumpkin-core-src-co-7ffaa0.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)


## Result

Sixth (final) pass, worker direct. No cargo/git. All five STILL OPEN rows were already resolved
in the main-checkout tree on arrival; verified each against decompiled vanilla
.vendor/vanilla-decompiled/net/minecraft/server/commands/RecipeCommand.java,
.vendor/vanilla-decompiled/net/minecraft/stats/ServerRecipeBook.java and the shared helper
crates/pumpkin-protocol/src/java/client/play/recipe_book_add.rs. No new .rs edits required.

- 5. Raw argument in error text (XS): VERIFIED. recipe.rs passes the parsed
  context.get_argument::<Identifier>("recipe")?.to_string() (always namespace:path) to
  ERROR_RECIPE_NOT_FOUND (recipe.notFound), matching vanilla ResourceKeyArgument.getRecipe,
  which echoes the parsed ResourceLocation.
- 4. Suggestion provider ids (S): VERIFIED fixed. recipe_suggestion_ids (recipe.rs:92) suggests
  only non-special recipe_id() manager keys and dedupes via HashSet, matching vanilla
  ResourceKeyArgument.key(RECIPE, LEARNABLE_RECIPE) with LEARNABLE_RECIPE = r -> !r.isSpecial()
  (RecipeCommand.java:22,43,69). Pure test recipe_suggestions_are_unique_and_skip_special present.
- 3. Display-id layout duplicated (S): VERIFIED fixed. static_recipe_display_count (:71),
  dynamic_recipe_has_display (:87) and dynamic_recipe_display_ids (:98) in recipe_book_add.rs
  are the single source shared by the writer (entry count :567-575, skip :664) and /recipe take
  (dynamic_recipe_display_ids at recipe.rs:232). Pure test
  dynamic_display_ids_skip_non_displayed_recipes present.
- 1. No server-side recipe book (M): VERIFIED fixed (outside unit). Player.known_recipes
  (player.rs:897, persisted and consumed by give/take) backs give. new_learnable_recipes adds
  only unknown non-special ids, sends CRecipeBookAdd only when added non-empty, returns the
  added count and throws COMMANDS_RECIPE_GIVE_FAILED at 0, matching vanilla
  ServerRecipeBook.addRecipes and RecipeCommand.giveRecipes (:93-104).
- 2. take remove packet / wrong count (M): VERIFIED fixed. take_display_ids (recipe.rs:66)
  removes only known learnable ids that have a display, sends CRecipeBookRemove only when
  removed non-empty, returns the removed count and throws COMMANDS_RECIPE_TAKE_FAILED at 0,
  matching vanilla ServerRecipeBook.removeRecipes and RecipeCommand.takeRecipes (:106-117).
  Pure test take_removes_only_known_recipes_with_displays present.
- Fifth-pass parity fix (row 1/2 id resolution): VERIFIED. recipe_matches (recipe.rs:40) matches
  manager keys only (recipe_id() == Some(str)), so a keyless recipe is not matched by its result
  item id, matching vanilla registry-key resolution. Pure test
  recipe_matching_uses_manager_keys_only present.

All rows now VERIFIED/FIXED; none remain STILL OPEN and none are BLOCKED-DATA. No .rs file edited
this pass, so rustfmt was not needed; compilation and the three pure unit tests are left to the
orchestrator batch check.
