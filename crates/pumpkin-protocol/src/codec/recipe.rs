use pumpkin_data::recipes::RecipeCategoryTypes;

use pumpkin_data::item::Item;
use pumpkin_data::tag::Taggable;

#[derive(Clone, Debug)]
pub enum OwnedRecipeIngredient {
    Simple(String),
    Tagged(String),
    OneOf(Vec<String>),
}

impl OwnedRecipeIngredient {
    #[must_use]
    pub fn match_item(&self, item: &Item) -> bool {
        match self {
            // Unqualified ids are sent to clients as minecraft ids, so they have to match the same way.
            Self::Simple(id) => id.strip_prefix("minecraft:").unwrap_or(id) == item.registry_key,
            Self::Tagged(tag) => item.is_tagged_with(tag).unwrap_or(false),
            Self::OneOf(ids) => ids
                .iter()
                .any(|id| id.strip_prefix("minecraft:").unwrap_or(id) == item.registry_key),
        }
    }
}

/// A vanilla brewing recipe ingredient: an item ingredient plus an optional potion predicate.
#[derive(Clone, Debug)]
pub struct OwnedPotionIngredient {
    pub ingredient: OwnedRecipeIngredient,
    /// Potion ids from the potion predicate holder set; empty means unrestricted.
    pub potions: Vec<String>,
}

impl OwnedPotionIngredient {
    #[must_use]
    pub fn match_item(&self, item: &Item) -> bool {
        self.ingredient.match_item(item)
    }

    /// Returns whether the given potion id satisfies the stored potion predicate.
    #[must_use]
    pub fn matches_potion(&self, potion: Option<&str>) -> bool {
        self.potions.is_empty()
            || potion.is_some_and(|potion| {
                let key = potion.strip_prefix("minecraft:").unwrap_or(potion);
                self.potions
                    .iter()
                    .any(|id| id.strip_prefix("minecraft:").unwrap_or(id) == key)
            })
    }
}

#[derive(Clone, Debug)]
pub struct OwnedRecipeResult {
    pub item_id: String,
    pub count: u8,
    // TODO: Add components/enchantments if needed for the display result
}

#[derive(Clone, Debug)]
pub enum OwnedCraftingRecipe {
    Shaped {
        recipe_id: Option<String>,
        category: RecipeCategoryTypes,
        group: Option<String>,
        show_notification: bool,
        key: Vec<(char, OwnedRecipeIngredient)>,
        pattern: Vec<String>,
        result: OwnedRecipeResult,
    },
    Shapeless {
        recipe_id: Option<String>,
        category: RecipeCategoryTypes,
        group: Option<String>,
        ingredients: Vec<OwnedRecipeIngredient>,
        result: OwnedRecipeResult,
    },
}

#[derive(Clone, Debug)]
pub struct OwnedCookingRecipe {
    pub recipe_id: String,
    pub category: RecipeCategoryTypes,
    pub group: Option<String>,
    pub ingredient: OwnedRecipeIngredient,
    pub cooking_time: i32,
    pub experience: f32,
    pub result: OwnedRecipeResult,
}

#[derive(Clone, Debug)]
pub enum OwnedCookingRecipeType {
    Blasting(OwnedCookingRecipe),
    Smelting(OwnedCookingRecipe),
    Smoking(OwnedCookingRecipe),
    CampfireCooking(OwnedCookingRecipe),
}

/// A vanilla brewing recipe: potion input, potion reagent and a templated output.
#[derive(Clone, Debug)]
pub struct OwnedBrewingRecipe {
    pub recipe_id: String,
    pub input: OwnedPotionIngredient,
    pub reagent: OwnedPotionIngredient,
    pub output: OwnedRecipeResult,
    /// The potion id on the output stack, from its potion contents component.
    pub output_potion: Option<String>,
}

#[derive(Clone, Debug)]
pub struct OwnedStonecutterRecipe {
    pub recipe_id: String,
    pub group: Option<String>,
    pub ingredient: OwnedRecipeIngredient,
    pub result: OwnedRecipeResult,
}

#[derive(Clone, Debug)]
pub struct OwnedSmithingTransformRecipe {
    pub recipe_id: String,
    pub template: OwnedRecipeIngredient,
    pub base: OwnedRecipeIngredient,
    pub addition: OwnedRecipeIngredient,
    pub result: OwnedRecipeResult,
}

#[derive(Clone, Debug)]
pub struct OwnedSmithingTrimRecipe {
    pub recipe_id: String,
    pub template: OwnedRecipeIngredient,
    pub base: OwnedRecipeIngredient,
    pub addition: OwnedRecipeIngredient,
    pub pattern: String,
}

#[derive(Clone, Debug)]
pub struct OwnedSpecialCraftingRecipe {
    pub recipe_id: String,
    pub category: RecipeCategoryTypes,
}

#[derive(Clone, Debug)]
pub enum DynamicRecipe {
    Crafting(OwnedCraftingRecipe),
    Cooking(OwnedCookingRecipeType),
    Brewing(OwnedBrewingRecipe),
    Stonecutting(OwnedStonecutterRecipe),
    SmithingTransform(OwnedSmithingTransformRecipe),
    SmithingTrim(OwnedSmithingTrimRecipe),
    SpecialCrafting(OwnedSpecialCraftingRecipe),
}

impl DynamicRecipe {
    #[must_use]
    pub fn recipe_id(&self) -> Option<&str> {
        match self {
            Self::Crafting(
                OwnedCraftingRecipe::Shaped { recipe_id, .. }
                | OwnedCraftingRecipe::Shapeless { recipe_id, .. },
            ) => recipe_id.as_deref(),
            Self::Cooking(
                OwnedCookingRecipeType::Blasting(r)
                | OwnedCookingRecipeType::Smelting(r)
                | OwnedCookingRecipeType::Smoking(r)
                | OwnedCookingRecipeType::CampfireCooking(r),
            ) => Some(&r.recipe_id),
            Self::Brewing(r) => Some(&r.recipe_id),
            Self::Stonecutting(r) => Some(&r.recipe_id),
            Self::SmithingTransform(r) => Some(&r.recipe_id),
            Self::SmithingTrim(r) => Some(&r.recipe_id),
            Self::SpecialCrafting(r) => Some(&r.recipe_id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ingredient_ids_match_with_or_without_the_namespace() {
        for id in ["stone", "minecraft:stone"] {
            assert!(OwnedRecipeIngredient::Simple(id.to_string()).match_item(&Item::STONE));
            assert!(OwnedRecipeIngredient::OneOf(vec![id.to_string()]).match_item(&Item::STONE));
        }
        assert!(!OwnedRecipeIngredient::Simple("stone".to_string()).match_item(&Item::DIRT));
    }

    #[test]
    fn potion_ingredient_matches_potion_holder_set() {
        let ingredient = OwnedPotionIngredient {
            ingredient: OwnedRecipeIngredient::Simple("minecraft:potion".to_string()),
            potions: vec![
                "minecraft:water".to_string(),
                "minecraft:awkward".to_string(),
            ],
        };
        assert!(ingredient.matches_potion(Some("water")));
        assert!(ingredient.matches_potion(Some("minecraft:awkward")));
        assert!(!ingredient.matches_potion(Some("minecraft:strength")));
        assert!(!ingredient.matches_potion(None));

        let unrestricted = OwnedPotionIngredient {
            ingredient: OwnedRecipeIngredient::Simple("minecraft:potion".to_string()),
            potions: Vec::new(),
        };
        assert!(unrestricted.matches_potion(None));
    }
}
