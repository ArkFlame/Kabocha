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

## Gap status
- Fixed in this pass section: FIXED per note.
- 1. No server-side recipe book (`recipe.rs:59-128`): STILL OPEN (M).
- 2. `take` remove packet always sent (`recipe.rs:130-237`): STILL OPEN (M).
- 3. Display-id layout duplicated (`recipe.rs:160`): STILL OPEN (S).
- 4. Suggestion provider ids (`recipe.rs:40-57`): STILL OPEN (S).
- 5. Raw argument in error text (`recipe.rs:24-25`): STILL OPEN (S).


## Result

Verification against decompiled vanilla: server/commands/RecipeCommand.java,
commands/arguments/ResourceKeyArgument.java, stats/ServerRecipeBook.java.

- 5. Raw argument in error text (XS): VERIFIED, no code change needed. Vanilla
  ResourceKeyArgument passes the parsed ResourceLocation to recipe.notFound; Pumpkin already
  passes context.get_argument::<Identifier>().to_string(), which always renders namespace:path
  (default minecraft:), so the echoed value matches. Only the cosmetic fallback literal differs.
- 4. Suggestion provider ids (S): FIXED. Added pure helper recipe_suggestion_ids in recipe.rs:
  suggests only DynamicRecipe::recipe_id() manager keys, skips SpecialCrafting (vanilla
  LEARNABLE_RECIPE filter from ResourceKeyArgument.key) and deduplicates. Provider now uses it.
  Added pure unit test recipe_suggestions_are_unique_and_skip_special.
- 3. Display-id layout duplicated (S): FIXED. Added static_recipe_display_count,
  dynamic_recipe_has_display and dynamic_recipe_display_ids in recipe_book_add.rs. The packet
  writer (entry count and skip decision) and /recipe take (offset and per-recipe ids) now share
  these helpers, so the skip set and prefix offset have one source of truth. Added pure unit
  test dynamic_display_ids_skip_non_displayed_recipes.
- 1. No server-side recipe book (M): STILL OPEN. Needs a per-player known-recipe set persisted
  like vanilla RecipeBook NBT (cross-crate Player state, outside these two files).
- 2. take remove packet always sent / wrong success count (M): STILL OPEN. Depends on gap 1;
  without server-side book state there is no way to know what each player actually knows.

Files touched:
- crates/pumpkin-core/src/command/commands/recipe.rs
- crates/pumpkin-protocol/src/java/client/play/recipe_book_add.rs

Checks run: rustfmt --edition 2024 on both touched .rs files (rc 0, no errors). No cargo run per
worker rules; compilation and tests left to orchestrator batch check.
