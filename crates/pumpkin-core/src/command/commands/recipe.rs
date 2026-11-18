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
    CRecipeBookAdd, CRecipeBookRemove, dynamic_recipe_display_ids,
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

static ERROR_GIVE_FAILED: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_RECIPE_GIVE_FAILED,
    translation::java::COMMANDS_RECIPE_GIVE_FAILED,
);
static ERROR_TAKE_FAILED: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_RECIPE_TAKE_FAILED,
    translation::java::COMMANDS_RECIPE_TAKE_FAILED,
);

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

/// A recipe manager key when it is learnable (vanilla LEARNABLE_RECIPE = !isSpecial).
fn learnable_recipe_id(recipe: &DynamicRecipe) -> Option<&str> {
    if matches!(recipe, DynamicRecipe::SpecialCrafting(_)) {
        return None;
    }
    recipe.recipe_id()
}

/// Learnable recipes the player does not know yet, in recipe-list order.
fn new_learnable_recipes<'a>(
    known: &HashSet<String>,
    recipes: &'a [DynamicRecipe],
) -> Vec<&'a DynamicRecipe> {
    recipes
        .iter()
        .filter(|recipe| learnable_recipe_id(recipe).is_some_and(|id| !known.contains(id)))
        .collect()
}

/// Display ids a `take` request removes, mirroring vanilla `ServerRecipeBook.removeRecipes`:
/// only recipes the player actually knows and that have a recipe-book display, matched by id
/// (or all when `*` was given). Each matched id is removed from `known`.
fn take_display_ids(
    known: &mut HashSet<String>,
    recipes: &[DynamicRecipe],
    display_ids: &[Option<i32>],
    is_all: bool,
    matches: &dyn Fn(&DynamicRecipe) -> bool,
) -> Vec<VarInt> {
    let mut ids_to_remove = Vec::new();
    for (recipe, display_id) in recipes.iter().zip(display_ids.iter()) {
        let Some(display_id) = *display_id else {
            continue;
        };
        if !is_all && !matches(recipe) {
            continue;
        }
        let Some(id) = learnable_recipe_id(recipe) else {
            continue;
        };
        if known.remove(id) {
            ids_to_remove.push(VarInt(display_id));
        }
    }
    ids_to_remove
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
            && seen.insert(id)
        {
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
        let matches = |recipe: &DynamicRecipe| {
            let id = get_recipe_id(recipe);
            id == recipe_str || id.strip_prefix("minecraft:").unwrap_or(&id) == recipe_str
        };
        let matching_recipes = if is_all {
            all_recipes
        } else {
            all_recipes
                .iter()
                .filter(|recipe| matches(recipe))
                .cloned()
                .collect::<Vec<_>>()
        };

        if !is_all && matching_recipes.is_empty() {
            return Err(ERROR_RECIPE_NOT_FOUND
                .create_without_context(TextComponent::text(recipe_str.clone())));
        }

        let mut total = 0i32;
        for player in &targets {
            let mut known = player
                .known_recipes
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let new_recipes = new_learnable_recipes(&known, &matching_recipes);
            let mut added = Vec::with_capacity(new_recipes.len());
            for recipe in new_recipes {
                if let Some(id) = learnable_recipe_id(recipe) {
                    known.insert(id.to_string());
                }
                added.push(recipe.clone());
            }
            drop(known);
            total += added.len() as i32;
            if !added.is_empty() {
                let packet = CRecipeBookAdd::new(false, &added);
                player.try_send_client_packet(&packet);
            }
        }

        if total == 0 {
            return Err(ERROR_GIVE_FAILED.create_without_context());
        }

        let total_str = total.to_string();
        if targets.len() == 1 {
            let msg = TextComponent::translate_cross(
                translation::java::COMMANDS_RECIPE_GIVE_SUCCESS_SINGLE,
                translation::java::COMMANDS_RECIPE_GIVE_SUCCESS_SINGLE,
                [
                    TextComponent::text(total_str),
                    targets[0].get_display_name(),
                ],
            );
            context.source.send_feedback(msg, true);
        } else {
            let msg = TextComponent::translate_cross(
                translation::java::COMMANDS_RECIPE_GIVE_SUCCESS_MULTIPLE,
                translation::java::COMMANDS_RECIPE_GIVE_SUCCESS_MULTIPLE,
                [
                    TextComponent::text(total_str),
                    TextComponent::text(targets.len().to_string()),
                ],
            );
            context.source.send_feedback(msg, true);
        }

        Ok(total)
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
            all_recipes.iter().filter(|recipe| matches(recipe)).count()
        };

        if !is_all && matched_count == 0 {
            return Err(ERROR_RECIPE_NOT_FOUND
                .create_without_context(TextComponent::text(recipe_str.clone())));
        }

        let mut total = 0i32;
        for player in &targets {
            let mut known = player
                .known_recipes
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let ids_to_remove =
                take_display_ids(&mut *known, &all_recipes, &display_ids, is_all, &matches);
            drop(known);
            total += ids_to_remove.len() as i32;
            if !ids_to_remove.is_empty() {
                let packet = CRecipeBookRemove::new(&ids_to_remove);
                player.try_send_client_packet(&packet);
            }
        }

        if total == 0 {
            return Err(ERROR_TAKE_FAILED.create_without_context());
        }

        let total_str = total.to_string();
        if targets.len() == 1 {
            let msg = TextComponent::translate_cross(
                translation::java::COMMANDS_RECIPE_TAKE_SUCCESS_SINGLE,
                translation::java::COMMANDS_RECIPE_TAKE_SUCCESS_SINGLE,
                [
                    TextComponent::text(total_str),
                    targets[0].get_display_name(),
                ],
            );
            context.source.send_feedback(msg, true);
        } else {
            let msg = TextComponent::translate_cross(
                translation::java::COMMANDS_RECIPE_TAKE_SUCCESS_MULTIPLE,
                translation::java::COMMANDS_RECIPE_TAKE_SUCCESS_MULTIPLE,
                [
                    TextComponent::text(total_str),
                    TextComponent::text(targets.len().to_string()),
                ],
            );
            context.source.send_feedback(msg, true);
        }

        Ok(total)
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

        let known = HashSet::from(["test:a".to_string()]);
        assert!(new_learnable_recipes(&known, &recipes).is_empty());
        let new = new_learnable_recipes(&HashSet::new(), &recipes);
        assert_eq!(new.len(), 2);
        assert!(
            new.iter()
                .all(|recipe| learnable_recipe_id(recipe) == Some("test:a"))
        );
    }

    #[test]
    fn take_removes_only_known_recipes_with_displays() {
        use pumpkin_protocol::codec::recipe::{
            OwnedCraftingRecipe, OwnedRecipeIngredient, OwnedRecipeResult, OwnedStonecutterRecipe,
        };

        let crafting = |id: &str| {
            DynamicRecipe::Crafting(OwnedCraftingRecipe::Shapeless {
                recipe_id: Some(id.to_string()),
                category: pumpkin_data::recipes::RecipeCategoryTypes::Misc,
                group: None,
                ingredients: vec![OwnedRecipeIngredient::Simple("minecraft:stone".to_string())],
                result: OwnedRecipeResult {
                    item_id: "minecraft:dirt".to_string(),
                    count: 1,
                },
            })
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

        // Known displayed recipe, non-displayed recipe, unknown displayed recipe, duplicate known id.
        let recipes = [
            crafting("test:a"),
            stonecutting("test:b"),
            crafting("test:c"),
            crafting("test:a"),
        ];
        let display_ids = dynamic_recipe_display_ids(&recipes);
        let offset = display_ids.iter().flatten().next().map_or(0, |id| id.0);

        let mut known = HashSet::from(["test:a".to_string()]);
        let matches = |recipe: &DynamicRecipe| recipe.recipe_id() == Some("test:a");
        let removed = take_display_ids(&mut known, &recipes, &display_ids, false, &matches);
        let removed_ids: Vec<i32> = removed.iter().map(|id| id.0).collect();

        assert_eq!(removed_ids, vec![offset]);
        assert!(known.is_empty());

        // The wildcard removes every known displayed recipe regardless of the match filter.
        let mut known_all = HashSet::from(["test:a".to_string(), "test:c".to_string()]);
        let removed_all = take_display_ids(&mut known_all, &recipes, &display_ids, true, &matches);
        assert_eq!(removed_all.len(), 2);
        assert!(known_all.is_empty());

        // Nothing known means nothing removed and no packet would be sent.
        let mut known_none = HashSet::new();
        assert!(
            take_display_ids(&mut known_none, &recipes, &display_ids, true, &matches).is_empty()
        );
    }
}
