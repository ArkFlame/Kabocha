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
- `recipe give <targets> *` counted only the datapack recipes (`all_recipes.len()`), so with a bare
  recipe manager it printed "Gave 0 recipes" while the add packet actually granted every static
  crafting/cooking entry. It now uses `recipe_book_entry_count`, the same numbering
  `CRecipeBookAdd::write_packet_data` writes (static crafting minus special/decorated pot, then
  static cooking, then dynamic crafting/cooking).
- The multiple-target feedback of both `give` and `take` printed the per-player count. Vanilla sums
  the awarded/removed count over every target (`i += player.awardRecipes(...)`) and passes that total
  to `commands.recipe.{give,take}.success.multiple`. Both messages now print the total.

## Remaining gaps
1. No server-side recipe book. `give` (recipe.rs:59-155) always sends `CRecipeBookAdd` and reports
   success, even when the player already knows every recipe, and it counts `targets * recipes` instead
   of the number of books that changed. Vanilla `ServerRecipeBook.addRecipe`/`addRecipes` return false
   for known recipes and `RecipeCommand` throws `commands.recipe.give.failed` ("No new recipes were
   learned", key already generated as `COMMANDS_RECIPE_GIVE_FAILED`) when nothing changed. Repeated
   `give` can also add duplicate client entries. Needs a per-player known-recipe set (persisted like
   vanilla `RecipeBook` NBT). Size M.

2. `take` (recipe.rs:164-271) has the mirror problem: it always sends the remove packet and reports
   success, never `commands.recipe.take.failed` ("No recipes could be forgotten"), and its return value
   is `targets * taken_count` rather than the number of entries actually removed, so `/execute store
   result` sees wrong counts. Depends on gap 1. Size M.

3. Cannot resolve any vanilla built-in recipe id (M). `/recipe give <id>` and `/recipe take <id>` match
   only `recipe_manager` (datapack recipes); the static `RECIPES_CRAFTING`/`RECIPES_COOKING` tables
   carry no recipe id at all (`crates/pumpkin-data/src/generated/recipes.rs:59-82`). Vanilla resolves
   the id against the whole recipe manager, so `give minecraft:stick` awards the stick recipe. Fix:
   emit `recipe_id` from `tools/pumpkin-codegen/src/recipes.rs` and match static recipes too,
   deriving the display id from the entry index.

4. `CRecipeBookAdd` skips stonecutter recipes (`recipe_book_add.rs:704-711`), but the vanilla recipe
   book has a stonecutter tab fed by the same packet. `/recipe give` of a stonecutter recipe reports
   success while sending no entry, and `take` removes an id that was never sent. Needs a stonecutter
   display writer. Size M.

5. `take *` counts only the displayable entries (crafting/cooking); vanilla counts every known recipe,
   including smithing/stonecutter/special (recipe.rs:199-206). Size XS, depends on gap 1.

6. Display-id layout is duplicated. `dynamic_recipe_offset` (recipe.rs:194) recomputes the static
   prefix that `CRecipeBookAdd::write_packet_data` assigns (pumpkin-protocol/src/java/client/play/
   recipe_book_add.rs:517-533: non-special crafting + all cooking, each only incremented when
   `write_entry` returns true). The new `recipe_book_entry_count` helper centralises the same count
   for `give *`, but `take` still builds its own list. Fix: expose one helper from pumpkin-protocol
   that maps a recipe list to display ids and have both the writer and the command use it. Size S/M.

7. Suggestion provider (recipe.rs:62-79) suggests `get_recipe_id`, which falls back to the result item
   id for recipes without a recipe id (`get_recipe_id`, recipe.rs:27-38) and yields "" for the rest, and
   it never deduplicates. Every construction site currently passes `Some(id)`, so the fallback is dead
   today. Vanilla suggests `RecipeManager` keys only. Size S.

8. Error text uses the raw argument string (recipe.rs:24-25, `recipe.notFound` = "Unknown recipe: %s");
   vanilla passes the parsed `ResourceLocation`, so a namespaced id is echoed with and without
   `minecraft:` differently. Cosmetic. Size XS.
