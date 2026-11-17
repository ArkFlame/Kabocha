# map_cloning crafting recipe (h2al sweep)

Vanilla: `assets/datapack/data/minecraft/recipe/map_cloning.json` is `minecraft:crafting_transmute` with `input` `#minecraft:clonable_maps`, `material` `minecraft:map`, `material_count` 1 to 8, `add_material_count_to_result` true, and `"result": {}`. Vanilla maps the empty result through its special-recipe path, which copies the input map and adds material count.

| Gap | Code | Vanilla | Status |
|---|---|---|---|
| map_cloning is never registered | `data/datapack/recipe_loader.rs` `parse_transmute` (around :318-321) returns `None` for `"result": {}` | `MapCloningRecipe` copies the input map stack (id, scale, decorations) | OPEN, size M |

Why not done in h2al: a registered recipe needs a copy-the-input result, which `OwnedCraftingRecipe` and the crafting result path cannot express today. Adding a variant touches the crafting matcher, the net recipe book (`net/java/play/place_recipe.rs`) and the bedrock recipe path, which cannot be compiled in this pass. Plan: add a `MapCloning` owned variant that parses `add_material_count_to_result` and `material_count`, then copy the input `filled_map` stack in the result step. Add a pure test for the material-count clamp (1 to 8) and a gametest for the copied map id.
