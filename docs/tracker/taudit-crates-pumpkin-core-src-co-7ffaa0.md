# audit/crates/pumpkin-core/src/command/commands/recipe.rs

Source: crates/pumpkin-core/src/command/commands/recipe.rs (no TODO/FIXME markers; the two grep hits were
`Identifier::vanilla_static("recipe")` registry-key literals).
No decompiled RecipeCommand in .vendor; comparisons are against vanilla `RecipeCommand` and
`ServerRecipeBook` (1.21.x), plus the en_us keys already shipped in assets/translations/en_us.json.

## Fixed in this pass
- `take` reported "Unknown recipe" for a recipe that exists in the manager but has no recipe book
  display (smithing, stonecutting, special crafting, decorated pot, brewing). The id lookup only ran
  over `DynamicRecipe::Crafting | Cooking`, while `give` accepted the same id. Vanilla `RecipeCommand`
  resolves the id with `RecipeManager.byKey` for both subcommands and only errors when it is absent.
  `RecipeTakeExecutor` now resolves against every dynamic recipe and skips the remove packet when there
  is no display id to send.

## Remaining gaps
1. No server-side recipe book. `give` (recipe.rs:59-128) always sends `CRecipeBookAdd` and reports
   success, even when the player already knows every recipe, and it counts `targets * recipes` instead
   of the number of books that changed. Vanilla `ServerRecipeBook.addRecipe`/`addRecipes` return false
   for known recipes and `RecipeCommand` throws `commands.recipe.give.failed` ("No new recipes were
   learned", key already generated as `COMMANDS_RECIPE_GIVE_FAILED`) when nothing changed. Needs a
   per-player known-recipe set (persisted like vanilla `RecipeBook` NBT). Size M.
2. `take` (recipe.rs:130-237) has the mirror problem: it always sends the remove packet and reports
   success, never `commands.recipe.take.failed` ("No recipes could be forgotten"), and its return value
   is `targets * taken_count` rather than the number of entries actually removed, so `/execute store
   result` sees wrong counts. Depends on gap 1. Size M.
3. Display-id layout is duplicated. `dynamic_recipe_offset` (recipe.rs:160) recomputes the static prefix
   that `CRecipeBookAdd::write_packet_data` assigns (pumpkin-protocol/src/java/client/play/
   recipe_book_add.rs:517-533: non-special crafting + all cooking, each only incremented when
   `write_entry` returns true). Today the two agree, but any change to the writer's skip set silently
   makes `/recipe take` remove unrelated recipes. Fix: expose one helper from pumpkin-protocol that maps
   a recipe list to display ids (and have the writer use it). Size S/M.
4. Suggestion provider (recipe.rs:40-57) suggests `get_recipe_id`, which falls back to the result item id
   for recipes without a recipe id (`get_recipe_id`, recipe.rs:27-38) and yields "" for the rest, and it
   never deduplicates. Vanilla suggests `RecipeManager` keys only. Size S.
5. Error text uses the raw argument string (recipe.rs:24-25, `recipe.notFound` = "Unknown recipe: %s");
   vanilla passes the parsed `ResourceLocation`, so a namespaced id is echoed with and without
   `minecraft:` differently. Cosmetic. Size XS.
