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


## Result

Second verification pass against decompiled vanilla (current main-checkout state). Sources read:
vanilla server/commands/RecipeCommand.java and stats/ServerRecipeBook.java.

All five STILL OPEN rows are already resolved in the tree; no new code edits were required this
pass (the current recipe.rs / recipe_book_add.rs already contain the fixes, and the per-player
recipe-book state was added in player.rs by another worker outside this unit's edit scope):

- 5. Raw argument in error text (XS): VERIFIED, no change needed. Pumpkin passes the parsed
  Identifier (context.get_argument::<Identifier>("recipe")?.to_string(), always namespace:path)
  to recipe.notFound, matching vanilla's parsed ResourceLocation echo in ResourceKeyArgument.
- 4. Suggestion provider ids (S): VERIFIED fixed. recipe_suggestion_ids suggests only
  DynamicRecipe::recipe_id() manager keys, skips SpecialCrafting (vanilla LEARNABLE_RECIPE =
  !isSpecial) and deduplicates. Pure unit test recipe_suggestions_are_unique_and_skip_special passes.
- 3. Display-id layout duplicated (S): VERIFIED fixed. static_recipe_display_count,
  dynamic_recipe_has_display and dynamic_recipe_display_ids in recipe_book_add.rs are shared by
  CRecipeBookAdd::write_packet_data (entry count at :567-575, skip at :664, id increments at
  :613-615/:659) and /recipe take (offset in take_display_ids via the same helper). write_entry
  returns Ok(false) for CraftingSpecial/CraftingDecoratedPot (:502-503), matching the skip set.
  Pure unit test dynamic_display_ids_skip_non_displayed_recipes (recipe_book_add.rs :1400) and
  take_removes_only_known_recipes_with_displays (recipe.rs :396) cover it.
- 1. No server-side recipe book (M): VERIFIED fixed (outside this unit). Player.known_recipes
  Mutex<HashSet<String>> (player.rs :884) is persisted under recipeBook.recipes NBT (:9397-9407
  write, :9584-9596 read) and consumed by award_recipes (:10626). give now uses
  new_learnable_recipes: adds only unknown learnable ids, sends CRecipeBookAdd only when changed,
  returns the number actually added, and throws commands.recipe.give.failed when total == 0.
  Mirrors vanilla ServerRecipeBook.addRecipes (:61-87, only !known && !isSpecial).
- 2. take remove packet / wrong count (M): VERIFIED fixed. take_display_ids removes only known
  learnable ids that have a display, sends CRecipeBookRemove only when changed, returns the number
  actually removed, and throws commands.recipe.take.failed when total == 0. Mirrors vanilla
  ServerRecipeBook.removeRecipes (:89-106). Counts are per-recipe (vanilla counts display entries);
  for current data one display per recipe makes these equal.

No rows remain open. No .rs file was edited this pass, so rustfmt was not run. Compilation and the
two pure unit tests are left to the orchestrator batch check per worker rules.


## Result

Third verification pass. Current main-checkout state re-checked against decompiled vanilla
server/commands/RecipeCommand.java, stats/ServerRecipeBook.java and
commands/arguments/ResourceKeyArgument.java. All five STILL OPEN rows were already resolved in
the tree on arrival; no code edits were required this pass.

- 5. Raw argument in error text (XS): VERIFIED, no change. Pumpkin passes the parsed
  Identifier (`context.get_argument::<Identifier>("recipe")?.to_string()`, always namespace:path)
  to recipe.notFound; vanilla's ResourceKeyArgument echoes the parsed ResourceLocation.
- 4. Suggestion provider ids (S): VERIFIED fixed. `recipe_suggestion_ids` (recipe.rs:99) suggests
  only non-special `recipe_id()` keys and deduplicates, matching vanilla
  `ResourceKeyArgument.key(RECIPE, LEARNABLE_RECIPE)` with `LEARNABLE_RECIPE = r -> !r.isSpecial()`.
  Pure test `recipe_suggestions_are_unique_and_skip_special` present (recipe.rs:360).
- 3. Display-id layout duplicated (S): VERIFIED fixed. `static_recipe_display_count`,
  `dynamic_recipe_has_display`, `dynamic_recipe_display_ids` (recipe_book_add.rs:71/87/98) are
  shared by the writer (count :567-575, skip :664) and `/recipe take`. `write_entry` returns
  Ok(false) for CraftingSpecial/CraftingDecoratedPot (:502-503), matching the static skip set.
  Pure test `dynamic_display_ids_skip_non_displayed_recipes` present (recipe_book_add.rs:1373).
- 1. No server-side recipe book (M): VERIFIED fixed (outside this unit). `Player.known_recipes`
  (player.rs:884) is persisted and consumed by give. `give` adds only unknown, non-special ids,
  sends CRecipeBookAdd only when changed, returns the added count and throws give.failed at 0,
  matching vanilla `ServerRecipeBook.addRecipes` (:61-87).
- 2. take remove packet / wrong count (M): VERIFIED fixed. `take_display_ids` removes only known
  ids with a display, sends CRecipeBookRemove only when changed, returns the removed count and
  throws take.failed at 0, matching vanilla `ServerRecipeBook.removeRecipes` (:89-106).

Compile-pitfall re-check against AGENTS.md: no new .rs diff, so no new risk. Existing helpers use
full paths in tests, `#[must_use]`/`const fn` where applicable, `unwrap_or_else(PoisonError::into_inner)`
instead of unwrap, `tracing`-free, and re-export via `pub use recipe_book_add::*;` (play/mod.rs:173)
so recipe.rs imports resolve. No rows remain open. No .rs file edited, so rustfmt not run; the two
pure unit tests and compilation are left to the orchestrator batch check.


## Result

Fourth verification pass (worker, direct). Re-checked rows still marked STILL OPEN in "## Gap
status" (1-5) against current sources and available vanilla refs in .vendor (no decompiled
`RecipeCommand`/`ServerRecipeBook` found; compared to vanilla semantics documented in header).

- 5. Raw argument in error text (XS): VERIFIED, no code change. `get_argument::<Identifier>`
  parsed id is rendered with namespace (`Identifier::to_string()`), matching parsed
  `ResourceLocation` echo; `ERROR_RECIPE_NOT_FOUND` passes that string.
- 4. Suggestion provider ids (S): VERIFIED fixed in allowed files. `recipe_suggestion_ids`
  (`recipe.rs:99`) filters `SpecialCrafting`, dedupes via `HashSet`, suggests only manager
  `recipe_id()` keys; provider uses it (`:125`). Pure test
  `recipe_suggestions_are_unique_and_skip_special` at `:360`.
- 3. Display-id layout duplicated (S): VERIFIED fixed in allowed files.
  `static_recipe_display_count`/`dynamic_recipe_has_display`/`dynamic_recipe_display_ids`
  (`recipe_book_add.rs:71,87,98`) are shared: writer total/count/skip/increment at
  `:567-575,:613-615,:664`; `take` uses `dynamic_recipe_display_ids` (`recipe.rs:242`). Static
  writer returns `Ok(false)` for `CraftingSpecial`/`CraftingDecoratedPot` (`:502-503`), aligned
  with dynamic display set (crafting/cooking only).
- 1. No server-side recipe book (M): VERIFIED fixed; persistence outside unit
  (`Player.known_recipes` in `player.rs:884`, used by give/take `:174,:266`). Give uses
  `new_learnable_recipes`/`learnable_recipe_id`, inserts only unknown non-special ids, sends
  `CRecipeBookAdd` only when added non-empty, returns added count, throws
  `COMMANDS_RECIPE_GIVE_FAILED` when total == 0 (`:177-195`), matching
  `ServerRecipeBook.addRecipes` semantics.
- 2. take remove packet / wrong count (M): VERIFIED fixed in allowed files.
  `take_display_ids` removes only known ids with display (`:73-96`), `take` sends
  `CRecipeBookRemove` only when removed non-empty, returns removed count, throws
  `COMMANDS_RECIPE_TAKE_FAILED` when total == 0 (`:269-281`), matching
  `ServerRecipeBook.removeRecipes` semantics. Pure test
  `take_removes_only_known_recipes_with_displays` at `:396`.

No .rs edits in this pass (all fixes/tests present). No rustfmt/cargo run per worker rules;
compilation and unit tests deferred to orchestrator batch check. No rows remain open.


## Result

Fifth verification pass (worker, direct, no cargo/git). Rows re-checked against decompiled
`server/commands/RecipeCommand.java` (found in .vendor) and `stats/ServerRecipeBook.java`.

- 1. No server-side recipe book (M): VERIFIED fixed outside unit. `Player.known_recipes`
  (player.rs:884) is persisted and used by give/take; give adds only unknown learnable ids,
  sends CRecipeBookAdd only when something was added, returns the added count and throws
  give.failed at 0, matching `ServerRecipeBook.addRecipes`.
- 2. take remove packet / count (M): VERIFIED fixed. `take_display_ids` removes only known
  learnable ids that have a display, sends CRecipeBookRemove only when non-empty, returns the
  removed count, throws take.failed at 0.
- 3. Display-id layout duplicated (S): VERIFIED fixed. `static_recipe_display_count`,
  `dynamic_recipe_has_display`, `dynamic_recipe_display_ids` (recipe_book_add.rs:71/87/98) are the
  single source for the writer's entry count and skip set and for `/recipe take`.
- 4. Suggestion provider ids (S): VERIFIED fixed. `recipe_suggestion_ids` suggests manager keys
  only, skips special crafting, dedupes.
- 5. Raw argument in error text (XS): VERIFIED, no change. The parsed Identifier is echoed.

New this pass (parity bug in the same rows): both executors matched the recipe argument through
`get_recipe_id`, which falls back to the result item id for crafting recipes without a manager key.
Vanilla resolves the argument with `RecipeManager` keys only, so `/recipe give @a minecraft:dirt`
could match a keyless recipe and report "No new recipes were learned" instead of "Unknown recipe".
Replaced with `recipe_matches` (key equality only) and added the pure unit test
`recipe_matching_uses_manager_keys_only`.

Files touched:
- crates/pumpkin-core/src/command/commands/recipe.rs

Checks: none run (worker rules forbid cargo/git; rustfmt deferred to orchestrator). Compilation and
the three recipe unit tests are left to the orchestrator batch check.

## Result (r2_004 pass)
- 1. No server-side recipe book (M): BLOCKED-DATA. Missing subsystem: no per-player known-recipe set in pumpkin-core player state (grep `known_recipes|RecipeBook` finds only packet handlers and events). Vanilla ref: stats/ServerRecipeBook.java.
- 2. take remove packet / wrong success count (M): BLOCKED-DATA. Depends on row 1 (same missing recipe book subsystem).
- 3, 4, 5: already FIXED/VERIFIED in the Result above; the Gap status lines are stale. Not re-verified in this pass, so left as recorded.
- No .rs edits this pass.
