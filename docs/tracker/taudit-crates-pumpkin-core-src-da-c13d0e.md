# datapack recipe_loader.rs (DEEP AUDIT)

Target: crates/pumpkin-core/src/data/datapack/recipe_loader.rs at JavaMinecraftVersion::V_26_3.
Reference: decompiled tree /run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft (.vendor absent),
mainly world/item/crafting/ShapedRecipePattern.java, TransmuteRecipe.java, BrewingRecipe.java,
PotionIngredient.java, Ingredient.java and RecipeSerializers.java.
Note: this loader only feeds custom datapack recipes; vanilla recipes come from generated
pumpkin-data (RECIPES_CRAFTING / RECIPES_SMITHING_TRIM / ...).

Markers: the file has no TODO/FIXME. The two "vanilla" mentions are doc comments on
MAX_SIZE/shrink and on the shrink test, both accurate.

Fixed in this pass:

- parse_shaped: key symbols are now validated like vanilla ShapedRecipePattern.Data
  SYMBOL_CODEC and unpack: exactly one character, space reserved, every pattern symbol must
  exist in the key, every key symbol must be used by the pattern. Before, a pattern
  referencing an undefined symbol was sent to clients with a missing key entry, and malformed
  keys were silently accepted or truncated to their first character.
  Test: shaped_pattern_validates_the_key_like_vanilla.
- parse_transmute: the hard-coded minecraft:filled_map fallback for an unparsable result is
  gone; vanilla TransmuteRecipe.MAP_CODEC uses a required TransmuteResult CODEC fieldOf
  "result", so an invalid result invalidates the recipe.
  Test: transmute_recipe_requires_a_result.

Checked and already matching vanilla (no change):

- shrink: raw rows/columns are capped at 3, rows must be equally wide, blank border rows and
  columns are trimmed, an all-blank pattern is rejected. Vanilla enforces the 3x3 cap on the
  raw pattern via ShapedRecipePattern.Data PATTERN_CODEC (the MAX_SIZE constant there is dead
  in this version), which is what this file does.
- Cooking defaults: 200/100/100/600 ticks and experience 0.0 match the per-serializer defaults;
  show_notification defaults to true only for shaped recipes.
- smithing_trim pattern is a real vanilla field (SmithingTrimRecipeStruct.pattern in
  tools/pumpkin-codegen/src/recipes.rs generates it from assets/).
- Tag ingredients: is_tagged_with accepts both #-prefixed and namespaced tag names, so the
  missing # prefix in parse_ingredient is harmless.

Remaining gaps:

| Location | Vanilla | Gap | Fix | Size |
|---|---|---|---|---|
| parse_ingredient 69-83 (Value::Array) | Ingredient.CODEC is a non-empty HolderSet: item ids, #tag strings, or a list mixing both | Array entries only accept plain strings and item objects; a tag object is dropped, so a mixed alternative list silently narrows to its item entries | Give OwnedRecipeIngredient::OneOf a tag representation (or #-prefixed entries handled in match_item), then update crates/pumpkin-core/src/net/bedrock/recipe.rs:179 and the plugin API Ingredient encoders | M |
| parse_result 99-103 | ItemStackTemplate count is validated 1..=99; out of range is a recipe error | Count is silently clamped with min(99) and 0 is accepted | Return None when count is 0 or > 99 | S |
| parse_cooking 255-262 | cookingtime/experience are parsed as int/float with codec range checks | t as i32 and e as f32 truncate or wrap a large JSON number | Use i32::try_from(t).ok() and reject non-finite experience | S |
| parse_category 113-125 | CraftingBookCategory (building/equipment/redstone/misc) and CookingBookCategory (food/blocks/misc) are separate strict enums; stonecutting has no category | One lenient matcher lowercases, maps the typo "restone", accepts any value as Misc, and lets a smelting recipe claim equipment | Split crafting/cooking category parsing, drop the typo branch, reject unknown values | S |
| parse_ingredient 57-68 | Ingredient codec errors when an object has both item and tag | Prefers tag and silently ignores item | Reject when both keys are present | S |
| parse_recipe 29-35 | Only the registered crafting_special_* serializers exist and each has required fields | Any crafting_special_* suffix becomes SpecialCrafting and type-specific fields are discarded | Match the known special types and keep their fields for display | M |
| parse_brewing 274-300 | BrewingRecipe = PotionIngredient input (item + optional potion_contents.potions HolderSet), same for reagent, plus an ItemStackTemplate output | potions is only read as a string so a HolderSet list yields None; reagent must be an item object so a tag or list reagent drops the recipe; OwnedBrewingRecipe models a single potion/item | Extend OwnedBrewingRecipe to a potion list and an OwnedRecipeIngredient reagent, then update crates/pumpkin-inventory brewing matching | M |
| parse_transmute 304-321 | TransmuteRecipe also has material_count (1..8), add_material_count_to_result and a component-carrying result | Parsed as a plain shapeless recipe [input, material]; material count bounds, result components and the keep-the-input-components semantics are lost | Model transmute separately (server and client display) instead of synthesising a shapeless recipe | M |
| parse_dye 323-339, parse_decorated_pot 341-361, parse_imbue 363-381 | Distinct recipe types whose results inherit components from the input or target slot | Flattened into synthetic Shapeless/Shaped recipes with a fixed key layout; recipe-book display and result components differ from vanilla | Keep the recipe type and derive the result in the craft handler | M |
