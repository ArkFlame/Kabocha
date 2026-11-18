# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-co-7ffaa0.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: recipe.rs

- 5. Raw argument in error text (`recipe.rs:24-25`): STILL OPEN (S).
- 1. No server-side recipe book (M): STILL OPEN. Needs a per-player known-recipe set persisted
- 2. take remove packet always sent / wrong success count (M): STILL OPEN. Depends on gap 1;
All five STILL OPEN rows are already resolved in the tree; no new code edits were required this

## Result

Final pass. All five rows VERIFIED against decompiled vanilla; no .rs edits were needed this pass
because every fix is already present in the tree.

- 5. Raw argument in error text (S): VERIFIED, no change. `recipe.rs:136/224` pass the parsed
  `Identifier.to_string()` (always namespace:path) to `ERROR_RECIPE_NOT_FOUND`, matching vanilla
  `ResourceKeyArgument` which echoes the parsed `ResourceLocation`
  (`.vendor/vanilla-decompiled/net/minecraft/commands/arguments/ResourceKeyArgument.java`).
- 4. Suggestion provider ids (S): VERIFIED fixed. `recipe_suggestion_ids` (`recipe.rs:92`) suggests
  only non-special `recipe_id()` keys and dedupes, matching vanilla `RecipeCommand.LEARNABLE_RECIPE`
  (`r -> !r.isSpecial()`) with `ResourceKeyArgument.key(RECIPE, LEARNABLE_RECIPE)` in
  `.vendor/vanilla-decompiled/net/minecraft/server/commands/RecipeCommand.java:22`. Pure test
  `recipe_suggestions_are_unique_and_skip_special` present (`recipe.rs:346`).
- 3. Display-id layout duplicated (S): VERIFIED fixed. `static_recipe_display_count`,
  `dynamic_recipe_has_display`, `dynamic_recipe_display_ids`
  (`recipe_book_add.rs:71/87/98`, re-exported via `play/mod.rs:173 pub use recipe_book_add::*`)
  are the single source for the writer's entry count/skip set and `/recipe take`. Writer uses them
  at `:570-572` and `:664`; `recipe.rs:232` uses the helper for `take` display ids. Pure test
  `dynamic_display_ids_skip_non_displayed_recipes` present in recipe_book_add.rs.
- 1. No server-side recipe book (M): VERIFIED fixed (persistence is in player.rs, outside this
  unit's basename but already applied). `Player.known_recipes: Mutex<HashSet<String>>`
  (`player.rs:897`, persisted at `:9455`/`:9643`). `give` uses `new_learnable_recipes` /
  `learnable_recipe_id` (`recipe.rs:53/45`), inserts only unknown non-special ids, sends
  `CRecipeBookAdd` only when `added` is non-empty (`:177-180`), returns the count actually added and
  throws `COMMANDS_RECIPE_GIVE_FAILED` when total == 0 (`:183-185`). Mirrors vanilla
  `ServerRecipeBook.addRecipes` (`.vendor/vanilla-decompiled/net/minecraft/stats/ServerRecipeBook.java:61-87`,
  `if (!known.contains(id) && !recipe.value().isSpecial())`, packet only when non-empty, return
  recipesToAdd.size()).
- 2. take remove packet always sent / wrong success count (M): VERIFIED fixed. `take_display_ids`
  (`recipe.rs:66`) removes only known ids that have a display, `take` sends `CRecipeBookRemove` only
  when `ids_to_remove` is non-empty (`:260-263`), returns the count actually removed and throws
  `COMMANDS_RECIPE_TAKE_FAILED` when total == 0 (`:266-268`). Mirrors vanilla
  `ServerRecipeBook.removeRecipes` (`ServerRecipeBook.java:89-106`, remove only when `known`, packet
  only when non-empty, return recipesToRemove.size()). Pure test
  `take_removes_only_known_recipes_with_displays` present (`recipe.rs:382`).

Additional parity fix (from prior pass, still present): both executors match the recipe argument via
`recipe_matches` (`recipe.rs:40`, key equality only) rather than `get_recipe_id`, so a keyless
crafting recipe cannot be matched by its result item id. Vanilla `RecipeCommand` resolves against
recipe-manager keys only. Pure test `recipe_matching_uses_manager_keys_only` present (`recipe.rs:443`).

Compile-pitfall re-check: no new .rs diff this pass, so no new risk. Existing code uses
`unwrap_or_else(PoisonError::into_inner)` (no unwrap/expect/panic), no println, helper placement in
free fns (not inside trait impls), full re-export path for `dynamic_recipe_display_ids`, and no
multi-line-edit hazards. No rows remain open. No rustfmt/cargo run per worker rules; compilation and
the three recipe unit tests are left to the orchestrator batch check.
