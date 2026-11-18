use pumpkin_data::recipes::RecipeCategoryTypes;
use pumpkin_protocol::codec::recipe::{
    DynamicRecipe, OwnedBrewingRecipe, OwnedCookingRecipe, OwnedCookingRecipeType,
    OwnedCraftingRecipe, OwnedPotionIngredient, OwnedRecipeIngredient, OwnedRecipeResult,
    OwnedSmithingTransformRecipe, OwnedSmithingTrimRecipe, OwnedSpecialCraftingRecipe,
    OwnedStonecutterRecipe,
};
use serde_json::Value;

pub fn parse_recipe(namespace: &str, name: &str, json_str: &str) -> Option<DynamicRecipe> {
    let value: Value = serde_json::from_str(json_str).ok()?;
    let recipe_id = format!("{namespace}:{name}");
    let raw_type = value.get("type")?.as_str()?;
    let recipe_type = raw_type.strip_prefix("minecraft:").unwrap_or(raw_type);

    match recipe_type {
        "crafting_shaped" => parse_shaped(recipe_id, &value).map(DynamicRecipe::Crafting),
        "crafting_shapeless" => parse_shapeless(recipe_id, &value).map(DynamicRecipe::Crafting),
        "crafting_transmute" => parse_transmute(recipe_id, &value).map(DynamicRecipe::Crafting),
        "crafting_dye" => parse_dye(recipe_id, &value).map(DynamicRecipe::Crafting),
        "crafting_decorated_pot" => {
            parse_decorated_pot(recipe_id, &value).map(DynamicRecipe::Crafting)
        }
        "crafting_imbue" => parse_imbue(recipe_id, &value).map(DynamicRecipe::Crafting),
        "stonecutting" => parse_stonecutting(recipe_id, &value).map(DynamicRecipe::Stonecutting),
        "smithing_transform" => {
            parse_smithing_transform(recipe_id, &value).map(DynamicRecipe::SmithingTransform)
        }
        "smithing_trim" => parse_smithing_trim(recipe_id, &value).map(DynamicRecipe::SmithingTrim),
        // Vanilla registers exactly these crafting_special_* serializers; an unknown type
        // is rejected by RecipeSerializers. CustomRecipe.category() is always MISC and its
        // codec has no category field.
        "crafting_special_bookcloning"
        | "crafting_special_mapextending"
        | "crafting_special_firework_rocket"
        | "crafting_special_firework_star"
        | "crafting_special_firework_star_fade"
        | "crafting_special_bannerduplicate"
        | "crafting_special_shielddecoration"
        | "crafting_special_repairitem" => {
            Some(DynamicRecipe::SpecialCrafting(OwnedSpecialCraftingRecipe {
                recipe_id,
                category: RecipeCategoryTypes::Misc,
            }))
        }
        "smelting" => parse_cooking(recipe_id, &value, 200)
            .map(|c| DynamicRecipe::Cooking(OwnedCookingRecipeType::Smelting(c))),
        "blasting" => parse_cooking(recipe_id, &value, 100)
            .map(|c| DynamicRecipe::Cooking(OwnedCookingRecipeType::Blasting(c))),
        "smoking" => parse_cooking(recipe_id, &value, 100)
            .map(|c| DynamicRecipe::Cooking(OwnedCookingRecipeType::Smoking(c))),
        "campfire_cooking" => parse_cooking(recipe_id, &value, 600)
            .map(|c| DynamicRecipe::Cooking(OwnedCookingRecipeType::CampfireCooking(c))),
        "brewing" => parse_brewing(recipe_id, &value).map(DynamicRecipe::Brewing),
        _ => None,
    }
}

fn normalize_id(id: &str) -> String {
    if id.contains(':') {
        id.to_string()
    } else {
        format!("minecraft:{id}")
    }
}

fn parse_ingredient(value: &Value) -> Option<OwnedRecipeIngredient> {
    match value {
        Value::String(s) => Some(OwnedRecipeIngredient::Simple(normalize_id(s))),
        // Vanilla Ingredient codec rejects an object that sets both `item` and `tag`.
        Value::Object(map) if map.contains_key("item") && map.contains_key("tag") => None,
        Value::Object(map) => map
            .get("tag")
            .and_then(Value::as_str)
            .map(|tag| OwnedRecipeIngredient::Tagged(normalize_id(tag)))
            .or_else(|| {
                map.get("item")
                    .and_then(Value::as_str)
                    .map(|item| OwnedRecipeIngredient::Simple(normalize_id(item)))
            }),
        Value::Array(arr) => {
            // Vanilla HolderSetCodec list entries are Item ids only (`#tag` belongs at the top level).
            // A tag or any other shape fails the whole ingredient instead of silently narrowing it.
            let ids = arr
                .iter()
                .map(|item| match item {
                    Value::String(s) if !s.starts_with('#') => Some(normalize_id(s)),
                    Value::Object(obj) => obj.get("item").and_then(Value::as_str).map(normalize_id),
                    _ => None,
                })
                .collect::<Option<Vec<_>>>()?;
            if ids.is_empty() {
                None
            } else {
                Some(OwnedRecipeIngredient::OneOf(ids))
            }
        }
        _ => None,
    }
}

fn parse_result(value: &Value) -> Option<OwnedRecipeResult> {
    match value {
        Value::String(s) => Some(OwnedRecipeResult {
            item_id: normalize_id(s),
            count: 1,
        }),
        Value::Object(map) => {
            let id = map
                .get("id")
                .or_else(|| map.get("item"))
                .and_then(Value::as_str)?;
            // Vanilla `ItemStackTemplate` uses `intRange(1, 99).optionalFieldOf("count", 1)`:
            // an absent count defaults to 1, but a present non-integer or out-of-range count
            // rejects the recipe rather than being silently defaulted.
            let count = match map.get("count") {
                None => 1u8,
                Some(value) => {
                    let count = value.as_u64()?;
                    if !(1..=99).contains(&count) {
                        return None;
                    }
                    count as u8
                }
            };
            Some(OwnedRecipeResult {
                item_id: normalize_id(id),
                count,
            })
        }
        _ => None,
    }
}

// Vanilla `CraftingBookCategory.CODEC` is a `StringRepresentable` codec: only exact
// serialized names are accepted, and `category` defaults to MISC when absent.
fn parse_crafting_category(value: Option<&Value>) -> Option<RecipeCategoryTypes> {
    let Some(val) = value else {
        return Some(RecipeCategoryTypes::Misc);
    };
    Some(match val.as_str()? {
        "building" => RecipeCategoryTypes::Building,
        "redstone" => RecipeCategoryTypes::Restone,
        "equipment" => RecipeCategoryTypes::Equipment,
        "misc" => RecipeCategoryTypes::Misc,
        _ => return None,
    })
}

// Vanilla `CookingBookCategory.CODEC`: separate strict enum, absent defaults to MISC.
fn parse_cooking_category(value: Option<&Value>) -> Option<RecipeCategoryTypes> {
    let Some(val) = value else {
        return Some(RecipeCategoryTypes::Misc);
    };
    Some(match val.as_str()? {
        "food" => RecipeCategoryTypes::Food,
        "blocks" => RecipeCategoryTypes::Blocks,
        "misc" => RecipeCategoryTypes::Misc,
        _ => return None,
    })
}

/// Vanilla `ShapedRecipePattern.MAX_SIZE`.
const MAX_SIZE: usize = 3;

/// Vanilla `ShapedRecipePattern.shrink`. Returns `None` for patterns vanilla rejects: more than
/// three rows or columns, rows of different widths, or no ingredients.
fn shrink(pattern: &[&str]) -> Option<Vec<String>> {
    if pattern.len() > MAX_SIZE {
        return None;
    }
    let rows: Vec<Vec<char>> = pattern.iter().map(|row| row.chars().collect()).collect();
    let width = rows.first()?.len();
    if width > MAX_SIZE || rows.iter().any(|row| row.len() != width) {
        return None;
    }

    let is_filled = |c: &char| *c != ' ';
    let top = rows.iter().position(|row| row.iter().any(is_filled))?;
    let bottom = rows.iter().rposition(|row| row.iter().any(is_filled))?;
    let left = rows
        .iter()
        .filter_map(|row| row.iter().position(is_filled))
        .min()?;
    let right = rows
        .iter()
        .filter_map(|row| row.iter().rposition(is_filled))
        .max()?;
    Some(
        rows[top..=bottom]
            .iter()
            .map(|row| row[left..=right].iter().collect())
            .collect(),
    )
}

fn parse_shaped(recipe_id: String, value: &Value) -> Option<OwnedCraftingRecipe> {
    let pattern = value
        .get("pattern")?
        .as_array()?
        .iter()
        .map(Value::as_str)
        .collect::<Option<Vec<_>>>()?;
    let pattern = shrink(&pattern)?;

    let key_map = value.get("key")?.as_object()?;
    let mut key = Vec::new();
    for (k, v) in key_map {
        // Vanilla's key symbol codec accepts exactly one character and reserves ' '.
        let mut chars = k.chars();
        let ch = chars.next()?;
        if chars.next().is_some() || ch == ' ' {
            return None;
        }
        let ingredient = parse_ingredient(v)?;
        key.push((ch, ingredient));
    }
    // Vanilla rejects patterns referencing a symbol missing from the key, and keys defining
    // symbols that the pattern never uses.
    for ch in pattern
        .iter()
        .flat_map(|row| row.chars())
        .filter(|c| *c != ' ')
    {
        if !key.iter().any(|(symbol, _)| *symbol == ch) {
            return None;
        }
    }
    if !key
        .iter()
        .all(|(symbol, _)| pattern.iter().any(|row| row.chars().any(|c| c == *symbol)))
    {
        return None;
    }

    let result = parse_result(value.get("result")?)?;
    let category = parse_crafting_category(value.get("category"))?;
    let group = value
        .get("group")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let show_notification = value
        .get("show_notification")
        .and_then(Value::as_bool)
        .unwrap_or(true);

    Some(OwnedCraftingRecipe::Shaped {
        recipe_id: Some(recipe_id),
        category,
        group,
        show_notification,
        key,
        pattern,
        result,
    })
}

fn parse_shapeless(recipe_id: String, value: &Value) -> Option<OwnedCraftingRecipe> {
    let ingredients_arr = value.get("ingredients")?.as_array()?;
    let mut ingredients = Vec::new();
    for ing_val in ingredients_arr {
        ingredients.push(parse_ingredient(ing_val)?);
    }

    let result = parse_result(value.get("result")?)?;
    let category = parse_crafting_category(value.get("category"))?;
    let group = value
        .get("group")
        .and_then(Value::as_str)
        .map(ToString::to_string);

    Some(OwnedCraftingRecipe::Shapeless {
        recipe_id: Some(recipe_id),
        category,
        group,
        ingredients,
        result,
    })
}

fn parse_cooking(
    recipe_id: String,
    value: &Value,
    default_cooking_time: i32,
) -> Option<OwnedCookingRecipe> {
    let ingredient = parse_ingredient(value.get("ingredient")?)?;
    let result = parse_result(value.get("result")?)?;
    let category = parse_cooking_category(value.get("category"))?;
    let group = value
        .get("group")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    // Vanilla `Codec.INT`/`Codec.FLOAT` reject out-of-range numbers. Checked casts avoid
    // silently wrapping an oversized cookingtime or overflowing experience to infinity.
    let cooking_time = match value.get("cookingtime") {
        Some(t) => i32::try_from(t.as_i64()?).ok()?,
        None => default_cooking_time,
    };
    let experience = match value.get("experience") {
        Some(e) => {
            let experience = e.as_f64()? as f32;
            if !experience.is_finite() {
                return None;
            }
            experience
        }
        None => 0.0,
    };

    Some(OwnedCookingRecipe {
        recipe_id,
        category,
        group,
        ingredient,
        cooking_time,
        experience,
        result,
    })
}

// A potion holder set may be a single id, a list of ids, or a tag reference.
fn normalize_potion_id(id: &str) -> String {
    match id.strip_prefix('#') {
        Some(rest) if rest.contains(':') => id.to_string(),
        Some(rest) => format!("#minecraft:{rest}"),
        None => normalize_id(id),
    }
}

// A tag entry in a potion holder set keeps its leading hash so it is not mistaken for an id.
fn normalize_potion_tag(tag: &str) -> String {
    normalize_potion_id(&format!("#{}", tag.strip_prefix('#').unwrap_or(tag)))
}

// Vanilla PotionIngredient is an Ingredient plus an optional PotionsPredicate.
fn parse_potion_ingredient(value: &Value) -> Option<OwnedPotionIngredient> {
    let ingredient = parse_ingredient(value.get("item")?)?;
    let mut potions = Vec::new();
    if let Some(contents) = value.get("potion_contents")
        && let Some(ids) = contents.get("potions")
    {
        match ids {
            Value::String(s) => potions.push(normalize_potion_id(s)),
            Value::Array(arr) => {
                for id in arr {
                    if let Some(s) = id.as_str() {
                        potions.push(normalize_potion_id(s));
                    } else {
                        let tag = id.get("tag").and_then(Value::as_str)?;
                        potions.push(normalize_potion_tag(tag));
                    }
                }
            }
            Value::Object(map) => {
                potions.push(normalize_potion_tag(
                    map.get("tag").and_then(Value::as_str)?,
                ));
            }
            _ => return None,
        }
    }
    Some(OwnedPotionIngredient {
        ingredient,
        potions,
    })
}

fn parse_brewing(recipe_id: String, value: &Value) -> Option<OwnedBrewingRecipe> {
    let input = parse_potion_ingredient(value.get("input")?)?;
    let reagent = parse_potion_ingredient(value.get("reagent")?)?;
    let output_value = value.get("output")?;
    let output = parse_result(output_value)?;
    let output_potion = output_value
        .get("components")
        .and_then(|c| c.get("minecraft:potion_contents"))
        .and_then(|pc| pc.get("potion"))
        .and_then(Value::as_str)
        .map(normalize_potion_id);

    Some(OwnedBrewingRecipe {
        recipe_id,
        input,
        reagent,
        output,
        output_potion,
    })
}

fn parse_transmute(recipe_id: String, value: &Value) -> Option<OwnedCraftingRecipe> {
    let category = parse_crafting_category(value.get("category"))?;
    let group = value
        .get("group")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let input = parse_ingredient(value.get("input")?)?;
    let material = parse_ingredient(value.get("material")?)?;
    // Vanilla's transmute codec requires a result; an invalid one invalidates the recipe.
    // Intentionally skipped: `minecraft:map_cloning` ships `"result": {}`, which vanilla maps
    // through its special-recipe path (copy the input map). Pumpkin has no such variant yet,
    // so it stays unregistered and the existing crafting behaviour is kept.
    let result = parse_result(value.get("result")?)?;
    Some(OwnedCraftingRecipe::Shapeless {
        recipe_id: Some(recipe_id),
        category,
        group,
        ingredients: vec![input, material],
        result,
    })
}

fn parse_dye(recipe_id: String, value: &Value) -> Option<OwnedCraftingRecipe> {
    let category = parse_crafting_category(value.get("category"))?;
    let group = value
        .get("group")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let target = parse_ingredient(value.get("target")?)?;
    let dye = parse_ingredient(value.get("dye")?)?;
    let result = parse_result(value.get("result")?)?;
    Some(OwnedCraftingRecipe::Shapeless {
        recipe_id: Some(recipe_id),
        category,
        group,
        ingredients: vec![target, dye],
        result,
    })
}

fn parse_decorated_pot(recipe_id: String, value: &Value) -> Option<OwnedCraftingRecipe> {
    let category = parse_crafting_category(value.get("category"))?;
    let group = value
        .get("group")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let back = parse_ingredient(value.get("back")?)?;
    let left = parse_ingredient(value.get("left")?)?;
    let right = parse_ingredient(value.get("right")?)?;
    let front = parse_ingredient(value.get("front")?)?;
    let result = parse_result(value.get("result")?)?;
    Some(OwnedCraftingRecipe::Shaped {
        recipe_id: Some(recipe_id),
        category,
        group,
        show_notification: true,
        key: vec![('B', back), ('L', left), ('R', right), ('F', front)],
        pattern: vec![" B ".to_string(), "L R".to_string(), " F ".to_string()],
        result,
    })
}

fn parse_imbue(recipe_id: String, value: &Value) -> Option<OwnedCraftingRecipe> {
    let category = parse_crafting_category(value.get("category"))?;
    let group = value
        .get("group")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let material = parse_ingredient(value.get("material")?)?;
    let source = parse_ingredient(value.get("source")?)?;
    let result = parse_result(value.get("result")?)?;
    Some(OwnedCraftingRecipe::Shaped {
        recipe_id: Some(recipe_id),
        category,
        group,
        show_notification: true,
        key: vec![('#', material), ('X', source)],
        pattern: vec!["###".to_string(), "#X#".to_string(), "###".to_string()],
        result,
    })
}

fn parse_stonecutting(recipe_id: String, value: &Value) -> Option<OwnedStonecutterRecipe> {
    let group = value
        .get("group")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let ingredient = parse_ingredient(value.get("ingredient")?)?;
    let result = parse_result(value.get("result")?)?;
    Some(OwnedStonecutterRecipe {
        recipe_id,
        group,
        ingredient,
        result,
    })
}

fn parse_smithing_transform(
    recipe_id: String,
    value: &Value,
) -> Option<OwnedSmithingTransformRecipe> {
    let template = parse_ingredient(value.get("template")?)?;
    let base = parse_ingredient(value.get("base")?)?;
    let addition = parse_ingredient(value.get("addition")?)?;
    let result = parse_result(value.get("result")?)?;
    Some(OwnedSmithingTransformRecipe {
        recipe_id,
        template,
        base,
        addition,
        result,
    })
}

fn parse_smithing_trim(recipe_id: String, value: &Value) -> Option<OwnedSmithingTrimRecipe> {
    let template = parse_ingredient(value.get("template")?)?;
    let base = parse_ingredient(value.get("base")?)?;
    let addition = parse_ingredient(value.get("addition")?)?;
    let pattern = value.get("pattern")?.as_str()?.to_string();
    Some(OwnedSmithingTrimRecipe {
        recipe_id,
        template,
        base,
        addition,
        pattern: normalize_id(&pattern),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_shaped_recipe() {
        let json = r##"{
            "type": "minecraft:crafting_shaped",
            "group": "sword",
            "pattern": [" X ", " X ", " # "],
            "key": {
                "#": "minecraft:stick",
                "X": "minecraft:diamond"
            },
            "result": {
                "id": "minecraft:diamond_sword",
                "count": 1
            }
        }"##;

        let parsed = parse_recipe("test", "diamond_sword", json);
        assert!(parsed.is_some());
        if let Some(DynamicRecipe::Crafting(OwnedCraftingRecipe::Shaped {
            recipe_id,
            result,
            pattern,
            ..
        })) = parsed
        {
            assert_eq!(recipe_id.as_deref(), Some("test:diamond_sword"));
            assert_eq!(result.item_id, "minecraft:diamond_sword");
            assert_eq!(pattern, ["X", "X", "#"]);
        } else {
            panic!("Expected shaped recipe");
        }
    }

    #[test]
    fn shaped_pattern_is_shrunk_like_vanilla() {
        assert_eq!(
            shrink(&[" # ", " I "]),
            Some(vec!["#".to_string(), "I".to_string()])
        );
        assert_eq!(
            shrink(&["###", "   ", "###"]),
            Some(vec![
                "###".to_string(),
                "   ".to_string(),
                "###".to_string()
            ])
        );
        assert_eq!(shrink(&["   ", "  #"]), Some(vec!["#".to_string()]));
        assert_eq!(shrink(&["##", "#"]), None);
        assert_eq!(shrink(&["   "]), None);
        assert_eq!(shrink(&["  # "]), None);
        assert_eq!(shrink(&["#", "#", "#", "#"]), None);
    }

    #[test]
    fn shaped_pattern_validates_the_key_like_vanilla() {
        let missing_symbol = r##"{
            "type": "minecraft:crafting_shaped",
            "pattern": ["X"],
            "key": {"#": "minecraft:stick"},
            "result": {"id": "minecraft:stick"}
        }"##;
        assert!(parse_recipe("test", "missing_symbol", missing_symbol).is_none());

        let unused_symbol = r##"{
            "type": "minecraft:crafting_shaped",
            "pattern": ["#"],
            "key": {"#": "minecraft:stick", "X": "minecraft:diamond"},
            "result": {"id": "minecraft:stick"}
        }"##;
        assert!(parse_recipe("test", "unused_symbol", unused_symbol).is_none());

        let multi_char_key = r#"{
            "type": "minecraft:crafting_shaped",
            "pattern": ["X"],
            "key": {"XY": "minecraft:stick"},
            "result": {"id": "minecraft:stick"}
        }"#;
        assert!(parse_recipe("test", "multi_char_key", multi_char_key).is_none());

        let space_key = r#"{
            "type": "minecraft:crafting_shaped",
            "pattern": ["X"],
            "key": {" ": "minecraft:stick"},
            "result": {"id": "minecraft:stick"}
        }"#;
        assert!(parse_recipe("test", "space_key", space_key).is_none());
    }

    #[test]
    fn transmute_recipe_requires_a_result() {
        let json = r#"{
            "type": "minecraft:crafting_transmute",
            "input": "minecraft:filled_map",
            "material": "minecraft:map",
            "result": {}
        }"#;
        assert!(parse_recipe("test", "map_cloning", json).is_none());
    }

    #[test]
    fn parse_shapeless_recipe() {
        let json = r#"{
            "type": "minecraft:crafting_shapeless",
            "ingredients": ["minecraft:oak_log"],
            "result": {
                "id": "minecraft:oak_planks",
                "count": 4
            }
        }"#;

        let parsed = parse_recipe("minecraft", "oak_planks_from_oak_log", json);
        assert!(parsed.is_some());
        if let Some(DynamicRecipe::Crafting(OwnedCraftingRecipe::Shapeless {
            recipe_id,
            result,
            ingredients,
            ..
        })) = parsed
        {
            assert_eq!(
                recipe_id.as_deref(),
                Some("minecraft:oak_planks_from_oak_log")
            );
            assert_eq!(result.item_id, "minecraft:oak_planks");
            assert_eq!(result.count, 4);
            assert_eq!(ingredients.len(), 1);
        } else {
            panic!("Expected shapeless recipe");
        }
    }

    #[test]
    fn parse_smelting_recipe() {
        let json = r#"{
            "type": "minecraft:smelting",
            "ingredient": "minecraft:raw_iron",
            "result": {
                "id": "minecraft:iron_ingot"
            },
            "experience": 0.7,
            "cookingtime": 200
        }"#;

        let parsed = parse_recipe("minecraft", "iron_ingot_from_smelting_raw_iron", json);
        assert!(parsed.is_some());
        if let Some(DynamicRecipe::Cooking(OwnedCookingRecipeType::Smelting(cooking))) = parsed {
            assert_eq!(
                cooking.recipe_id,
                "minecraft:iron_ingot_from_smelting_raw_iron"
            );
            assert_eq!(cooking.result.item_id, "minecraft:iron_ingot");
            assert_eq!(cooking.cooking_time, 200);
        } else {
            panic!("Expected smelting recipe");
        }
    }

    #[test]
    fn parse_brewing_recipe() {
        let json = r#"{
            "type": "minecraft:brewing",
            "input": {
                "item": "minecraft:potion",
                "potion_contents": {
                    "potions": "minecraft:water"
                }
            },
            "output": {
                "components": {
                    "minecraft:potion_contents": {
                        "potion": "minecraft:awkward"
                    }
                },
                "id": "minecraft:potion"
            },
            "reagent": {
                "item": "minecraft:nether_wart"
            }
        }"#;

        let parsed = parse_recipe("minecraft", "brewing/potion_water_nether_wart", json);
        assert!(matches!(&parsed, Some(DynamicRecipe::Brewing(_))));
        if let Some(DynamicRecipe::Brewing(brewing)) = parsed {
            assert_eq!(
                brewing.recipe_id,
                "minecraft:brewing/potion_water_nether_wart"
            );
            assert!(matches!(
                brewing.input.ingredient,
                OwnedRecipeIngredient::Simple(ref id) if id.as_str() == "minecraft:potion"
            ));
            assert_eq!(brewing.input.potions, vec!["minecraft:water".to_string()]);
            assert!(matches!(
                brewing.reagent.ingredient,
                OwnedRecipeIngredient::Simple(ref id) if id.as_str() == "minecraft:nether_wart"
            ));
            assert!(brewing.reagent.potions.is_empty());
            assert_eq!(brewing.output.item_id, "minecraft:potion");
            assert_eq!(brewing.output_potion.as_deref(), Some("minecraft:awkward"));
        }
    }

    #[test]
    fn brewing_potion_holder_set_is_kept_as_a_list() {
        let json = r#"{
            "type": "minecraft:brewing",
            "input": {
                "item": "minecraft:potion",
                "potion_contents": { "potions": ["minecraft:water", "minecraft:awkward"] }
            },
            "reagent": { "item": "minecraft:nether_wart" },
            "output": {
                "id": "minecraft:potion",
                "components": { "minecraft:potion_contents": { "potion": "minecraft:strength" } }
            }
        }"#;
        let parsed = parse_recipe("minecraft", "brewing/multi", json);
        assert!(matches!(&parsed, Some(DynamicRecipe::Brewing(_))));
        if let Some(DynamicRecipe::Brewing(brewing)) = parsed {
            assert_eq!(
                brewing.input.potions,
                vec![
                    "minecraft:water".to_string(),
                    "minecraft:awkward".to_string()
                ]
            );
            assert_eq!(brewing.output_potion.as_deref(), Some("minecraft:strength"));
        }
    }

    #[test]
    fn result_count_must_be_within_one_to_ninety_nine() {
        let ok = parse_result(&serde_json::json!({"id": "minecraft:stick", "count": 99}));
        assert_eq!(ok.map(|r| r.count), Some(99));
        assert!(parse_result(&serde_json::json!({"id": "minecraft:stick", "count": 0})).is_none());
        assert!(
            parse_result(&serde_json::json!({"id": "minecraft:stick", "count": 100})).is_none()
        );
    }

    #[test]
    fn result_count_must_be_an_integer() {
        // Vanilla `intRange` rejects a present count that is not a non-negative integer;
        // the old `as_u64().unwrap_or(1)` silently turned these into a count of 1.
        assert!(parse_result(&serde_json::json!({"id": "minecraft:stick", "count": -1})).is_none());
        assert!(
            parse_result(&serde_json::json!({"id": "minecraft:stick", "count": 2.5})).is_none()
        );
        assert!(
            parse_result(&serde_json::json!({"id": "minecraft:stick", "count": "2"})).is_none()
        );
        assert_eq!(
            parse_result(&serde_json::json!({"id": "minecraft:stick"})).map(|r| r.count),
            Some(1)
        );
    }

    #[test]
    fn cooking_time_and_experience_use_checked_conversion() {
        let recipe = |cookingtime: &str, experience: &str| {
            format!(
                r#"{{
                    "type": "minecraft:smelting",
                    "ingredient": "minecraft:raw_iron",
                    "result": {{"id": "minecraft:iron_ingot"}},
                    "cookingtime": {cookingtime},
                    "experience": {experience}
                }}"#
            )
        };

        // In-range values still parse.
        let ok = recipe("200", "0.7");
        assert!(parse_recipe("test", "ok", &ok).is_some());

        // A cookingtime outside i32 range is rejected instead of wrapping.
        let too_big = recipe("2147483648", "0.0");
        assert!(parse_recipe("test", "too_big", &too_big).is_none());
        let too_small = recipe("-2147483649", "0.0");
        assert!(parse_recipe("test", "too_small", &too_small).is_none());

        // A finite f64 that overflows to an infinite f32 is rejected.
        let overflow = recipe("200", "1e100");
        assert!(parse_recipe("test", "overflow", &overflow).is_none());
    }

    #[test]
    fn category_codecs_are_strict_and_separate() {
        use serde_json::json;

        assert!(matches!(
            parse_crafting_category(None),
            Some(RecipeCategoryTypes::Misc)
        ));
        assert!(matches!(
            parse_crafting_category(Some(&json!("building"))),
            Some(RecipeCategoryTypes::Building)
        ));
        assert!(matches!(
            parse_crafting_category(Some(&json!("redstone"))),
            Some(RecipeCategoryTypes::Restone)
        ));
        assert!(matches!(
            parse_crafting_category(Some(&json!("equipment"))),
            Some(RecipeCategoryTypes::Equipment)
        ));
        assert!(matches!(
            parse_crafting_category(Some(&json!("misc"))),
            Some(RecipeCategoryTypes::Misc)
        ));
        // The old "restone" typo, wrong case, cross-enum values and non-strings are rejected.
        assert!(parse_crafting_category(Some(&json!("restone"))).is_none());
        assert!(parse_crafting_category(Some(&json!("Building"))).is_none());
        assert!(parse_crafting_category(Some(&json!("food"))).is_none());
        assert!(parse_crafting_category(Some(&json!(123))).is_none());

        assert!(matches!(
            parse_cooking_category(Some(&json!("food"))),
            Some(RecipeCategoryTypes::Food)
        ));
        assert!(matches!(
            parse_cooking_category(Some(&json!("blocks"))),
            Some(RecipeCategoryTypes::Blocks)
        ));
        assert!(matches!(
            parse_cooking_category(Some(&json!("misc"))),
            Some(RecipeCategoryTypes::Misc)
        ));
        assert!(parse_cooking_category(Some(&json!("equipment"))).is_none());
        assert!(parse_cooking_category(Some(&json!("Building"))).is_none());
    }

    #[test]
    fn invalid_category_rejects_the_recipe() {
        let smelting = r#"{
            "type": "minecraft:smelting",
            "category": "equipment",
            "ingredient": "minecraft:raw_iron",
            "result": {"id": "minecraft:iron_ingot"}
        }"#;
        assert!(parse_recipe("test", "bad_smelting_category", smelting).is_none());

        let shaped = r#"{
            "type": "minecraft:crafting_shaped",
            "category": "restone",
            "pattern": ["X"],
            "key": {"X": "minecraft:stick"},
            "result": {"id": "minecraft:stick"}
        }"#;
        assert!(parse_recipe("test", "bad_shaped_category", shaped).is_none());
    }

    #[test]
    fn crafting_special_rejects_unregistered_suffixes() {
        let unknown = r#"{"type": "minecraft:crafting_special_bogus"}"#;
        assert!(parse_recipe("test", "bogus", unknown).is_none());

        let known = r#"{"type": "minecraft:crafting_special_bookcloning"}"#;
        assert!(matches!(
            parse_recipe("test", "book_cloning", known),
            Some(DynamicRecipe::SpecialCrafting(_))
        ));
    }
}

#[cfg(test)]
mod ingredient_codec_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn ingredient_object_with_item_and_tag_is_rejected() {
        assert!(
            parse_ingredient(&json!({"item": "minecraft:stick", "tag": "minecraft:planks"}))
                .is_none()
        );
        assert!(parse_ingredient(&json!({"item": "minecraft:stick"})).is_some());
    }

    #[test]
    fn ingredient_list_with_tag_entry_is_rejected() {
        // Vanilla HolderSetCodec list elements are Item ids only, so a mixed tag list fails as a whole.
        assert!(
            parse_ingredient(&json!(["minecraft:stick", {"tag": "minecraft:planks"}])).is_none()
        );
        assert!(parse_ingredient(&json!(["minecraft:stick", "#minecraft:planks"])).is_none());
        assert!(parse_ingredient(&json!(["minecraft:stick", "minecraft:bowl"])).is_some());
    }
}
