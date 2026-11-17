use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::argument_types::registry_id::RegistryIdArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::command::suggestion::provider::{SuggestionProvider, SuggestionProviderResult};
use crate::command::suggestion::suggestions::SuggestionsBuilder;
use crate::entity::EntityBase;
use pumpkin_data::translation;
use pumpkin_protocol::codec::recipe::DynamicRecipe;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::{
    CRecipeBookAdd, CRecipeBookRemove, dynamic_recipe_display_ids, static_recipe_display_count,
};
use pumpkin_util::PermissionLvl;
use pumpkin_util::identifier::Identifier;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;
use std::collections::HashSet;

const DESCRIPTION: &str = "Gives or takes player recipes.";
const PERMISSION: &str = "minecraft:command.recipe";

static ERROR_RECIPE_NOT_FOUND: CommandErrorType<1> =
    CommandErrorType::new(translation::java::RECIPE_NOTFOUND, "Unknown recipe: %s");

fn get_recipe_id(recipe: &DynamicRecipe) -> String {
    recipe.recipe_id().map_or_else(
        || match recipe {
            DynamicRecipe::Crafting(
                pumpkin_protocol::codec::recipe::OwnedCraftingRecipe::Shaped { result, .. }
                | pumpkin_protocol::codec::recipe::OwnedCraftingRecipe::Shapeless { result, .. },
            ) => result.item_id.clone(),
            _ => String::new(),
        },
        str::to_string,
    )
}

/// Vanilla's recipe argument suggests recipe-manager keys only, filtered to learnable recipes.
fn recipe_suggestion_ids(recipes: &[DynamicRecipe]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut ids = Vec::new();
    for recipe in recipes {
        if matches!(recipe, DynamicRecipe::SpecialCrafting(_)) {
            continue;
        }
        if let Some(id) = recipe.recipe_id()
            && seen.insert(id) {
                ids.push(id.to_string());
            }
    }
    ids
}

struct RecipeSuggestionProvider;

impl SuggestionProvider for RecipeSuggestionProvider {
    fn suggest(
        &self,
        context: &CommandContext,
        mut builder: SuggestionsBuilder,
    ) -> SuggestionProviderResult {
        if let Some(server) = &context.source.server {
            let recipes = server.recipe_manager.get_dynamic_recipes_internal();
            for id in recipe_suggestion_ids(&recipes) {
                builder = builder.suggest(id);
            }
        }
        builder.build()
    }
}

struct RecipeGiveExecutor {
    all: bool,
}

impl CommandExecutor for RecipeGiveExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let targets = EntityArgumentType::get_players(context, "targets")?;
        let recipe_str = if self.all {
            "*".to_string()
        } else {
            context.get_argument::<Identifier>("recipe")?.to_string()
        };

        let server = context.source.server.as_ref().ok_or_else(|| {
            ERROR_RECIPE_NOT_FOUND.create_without_context(TextComponent::text(recipe_str.clone()))
        })?;

        let all_recipes = server.recipe_manager.get_dynamic_recipes_internal();

        let is_all = recipe_str == "*";
        let matching_recipes = if is_all {
            all_recipes
        } else {
            all_recipes
                .iter()
                .filter(|r| {
                    let id = get_recipe_id(r);
                    id == recipe_str || id.strip_prefix("minecraft:").unwrap_or(&id) == recipe_str
                })
                .cloned()
                .collect::<Vec<_>>()
        };

        if !is_all && matching_recipes.is_empty() {
            return Err(ERROR_RECIPE_NOT_FOUND
                .create_without_context(TextComponent::text(recipe_str.clone())));
        }

        let recipe_count = matching_recipes.len();
        let packet = CRecipeBookAdd::new(false, &matching_recipes);
        for player in &targets {
            player.try_send_client_packet(&packet);
        }

        let recipe_count_str = recipe_count.to_string();
        if targets.len() == 1 {
            let msg = TextComponent::translate_cross(
                translation::java::COMMANDS_RECIPE_GIVE_SUCCESS_SINGLE,
                translation::java::COMMANDS_RECIPE_GIVE_SUCCESS_SINGLE,
                [
                    TextComponent::text(recipe_count_str),
                    targets[0].get_display_name(),
                ],
            );
            context.source.send_feedback(msg, true);
        } else {
            let msg = TextComponent::translate_cross(
                translation::java::COMMANDS_RECIPE_GIVE_SUCCESS_MULTIPLE,
                translation::java::COMMANDS_RECIPE_GIVE_SUCCESS_MULTIPLE,
                [
                    TextComponent::text(recipe_count_str),
                    TextComponent::text(targets.len().to_string()),
                ],
            );
            context.source.send_feedback(msg, true);
        }

        Ok((targets.len() * recipe_count) as i32)
    }
}

struct RecipeTakeExecutor {
    all: bool,
}

impl CommandExecutor for RecipeTakeExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let targets = EntityArgumentType::get_players(context, "targets")?;
        let recipe_str = if self.all {
            "*".to_string()
        } else {
            context.get_argument::<Identifier>("recipe")?.to_string()
        };

        let server = context.source.server.as_ref().ok_or_else(|| {
            ERROR_RECIPE_NOT_FOUND.create_without_context(TextComponent::text(recipe_str.clone()))
        })?;

        let all_recipes = server.recipe_manager.get_dynamic_recipes_internal();
        let display_ids = dynamic_recipe_display_ids(&all_recipes);

        let is_all = recipe_str == "*";

        // Vanilla resolves the id against the recipe manager, so a recipe that exists but has no
        // recipe book display (smithing, stonecutting, special crafting) is still valid here.
        let matches = |recipe: &DynamicRecipe| {
            let id = get_recipe_id(recipe);
            id == recipe_str || id.strip_prefix("minecraft:").unwrap_or(&id) == recipe_str
        };

        let matched_count = if is_all {
            0
        } else {
            all_recipes.iter().filter(|r| matches(r)).count()
        };

        if !is_all && matched_count == 0 {
            return Err(ERROR_RECIPE_NOT_FOUND
                .create_without_context(TextComponent::text(recipe_str.clone())));
        }

        let recipe_ids_to_remove: Vec<VarInt> = if is_all {
            let total = static_recipe_display_count()
                + display_ids.iter().filter(|id| id.is_some()).count();
            (0..total).map(|id| VarInt(id as i32)).collect()
        } else {
            display_ids
                .iter()
                .zip(all_recipes.iter())
                .filter_map(|(display_id, recipe)| {
                    (*display_id).and_then(|id| matches(recipe).then_some(VarInt(id)))
                })
                .collect()
        };

        let taken_count = if is_all {
            recipe_ids_to_remove.len()
        } else {
            matched_count
        };

        if !recipe_ids_to_remove.is_empty() {
            let packet = CRecipeBookRemove::new(&recipe_ids_to_remove);
            for player in &targets {
                player.try_send_client_packet(&packet);
            }
        }

        let taken_count_str = taken_count.to_string();
        if targets.len() == 1 {
            let msg = TextComponent::translate_cross(
                translation::java::COMMANDS_RECIPE_TAKE_SUCCESS_SINGLE,
                translation::java::COMMANDS_RECIPE_TAKE_SUCCESS_SINGLE,
                [
                    TextComponent::text(taken_count_str),
                    targets[0].get_display_name(),
                ],
            );
            context.source.send_feedback(msg, true);
        } else {
            let msg = TextComponent::translate_cross(
                translation::java::COMMANDS_RECIPE_TAKE_SUCCESS_MULTIPLE,
                translation::java::COMMANDS_RECIPE_TAKE_SUCCESS_MULTIPLE,
                [
                    TextComponent::text(taken_count_str),
                    TextComponent::text(targets.len().to_string()),
                ],
            );
            context.source.send_feedback(msg, true);
        }

        Ok((targets.len() * taken_count) as i32)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    let builder = command("recipe", DESCRIPTION)
        .requires(PERMISSION)
        .then(
            literal("give").then(
                argument("targets", EntityArgumentType::Players)
                    .then(literal("*").executes(RecipeGiveExecutor { all: true }))
                    .then(
                        argument(
                            "recipe",
                            RegistryIdArgumentType::resource_key(Identifier::vanilla_static(
                                "recipe",
                            )),
                        )
                        .suggests(RecipeSuggestionProvider)
                        .executes(RecipeGiveExecutor { all: false }),
                    ),
            ),
        )
        .then(
            literal("take").then(
                argument("targets", EntityArgumentType::Players)
                    .then(literal("*").executes(RecipeTakeExecutor { all: true }))
                    .then(
                        argument(
                            "recipe",
                            RegistryIdArgumentType::resource_key(Identifier::vanilla_static(
                                "recipe",
                            )),
                        )
                        .suggests(RecipeSuggestionProvider)
                        .executes(RecipeTakeExecutor { all: false }),
                    ),
            ),
        );

    dispatcher.register(builder);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recipe_suggestions_are_unique_and_skip_special() {
        use pumpkin_protocol::codec::recipe::{
            OwnedRecipeIngredient, OwnedRecipeResult, OwnedSpecialCraftingRecipe,
            OwnedStonecutterRecipe,
        };

        let stonecutting = |id: &str| {
            DynamicRecipe::Stonecutting(OwnedStonecutterRecipe {
                recipe_id: id.to_string(),
                group: None,
                ingredient: OwnedRecipeIngredient::Simple("minecraft:stone".to_string()),
                result: OwnedRecipeResult {
                    item_id: "minecraft:dirt".to_string(),
                    count: 1,
                },
            })
        };
        let special = DynamicRecipe::SpecialCrafting(OwnedSpecialCraftingRecipe {
            recipe_id: "test:special".to_string(),
            category: pumpkin_data::recipes::RecipeCategoryTypes::Misc,
        });

        let recipes = [stonecutting("test:a"), stonecutting("test:a"), special];
        assert_eq!(recipe_suggestion_ids(&recipes), vec!["test:a".to_string()]);
    }
}
