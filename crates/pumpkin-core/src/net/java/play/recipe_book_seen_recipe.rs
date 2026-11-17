#[allow(clippy::wildcard_imports)]
use super::*;
use pumpkin_data::recipes::{CraftingRecipeTypes, RECIPES_COOKING, RECIPES_CRAFTING};

/// Total recipe display ids, resolved in `place_recipe` order: crafting, cooking, then dynamic.
fn recipe_display_count(dynamic_recipes: usize) -> usize {
    let crafting = RECIPES_CRAFTING
        .iter()
        .filter(|r| {
            !matches!(
                r,
                CraftingRecipeTypes::CraftingSpecial(_)
                    | CraftingRecipeTypes::CraftingDecoratedPot { .. }
            )
        })
        .count();
    crafting + RECIPES_COOKING.len() + dynamic_recipes
}

/// Vanilla ignores a seen-recipe packet for an id that resolves to no recipe.
fn recipe_display_exists(display_id: i32, display_count: usize) -> bool {
    usize::try_from(display_id).is_ok_and(|id| id < display_count)
}

impl JavaClient {
    pub fn handle_recipe_book_seen_recipe(
        &self,
        server: &Arc<Server>,
        player: &Arc<Player>,
        packet: &SRecipeBookSeenRecipe,
    ) {
        let dynamic_count = server.recipe_manager.get_dynamic_recipes().len();
        if !recipe_display_exists(
            packet.recipe_display_id.0,
            recipe_display_count(dynamic_count),
        ) {
            return;
        }
        let mut event = crate::plugin::api::events::player::player_recipe_discover::PlayerRecipeDiscoverEvent::new(
            player.clone(),
            format!("display_{}", packet.recipe_display_id.0),
        );
        server.plugin_manager.fire_blocking(server, &mut event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recipe_display_exists_rejects_negative_and_out_of_range_ids() {
        assert!(!recipe_display_exists(-1, 10));
        assert!(recipe_display_exists(0, 10));
        assert!(recipe_display_exists(9, 10));
        assert!(!recipe_display_exists(10, 10));
        assert!(!recipe_display_exists(0, 0));
    }
}
