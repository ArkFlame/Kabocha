use crate::command::CommandSource;
use crate::command::argument_types::FromStringReader;
use crate::command::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::command::context::command_context::CommandContext;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::string_reader::StringReader;
use crate::command::suggestion::suggestions::{Suggestions, SuggestionsBuilder};
use crate::data::datapack::dynamic_registry_manager::DynamicRegistryManager;
use pumpkin_data::{Advancement, translation};
use pumpkin_util::identifier::Identifier;
use pumpkin_util::resource::ResourceKey;
use pumpkin_util::text::TextComponent;
use std::string::ToString;

pub static ADVANCEMENT_REGISTRY: &Identifier = &Identifier::vanilla_static("advancement");
pub static DIMENSION_REGISTRY: &Identifier = &Identifier::vanilla_static("dimension");
pub static BIOME_REGISTRY: &Identifier = &Identifier::vanilla_static("worldgen/biome");

pub const ERROR_INVALID_ADVANCEMENT: CommandErrorType<1> = CommandErrorType::new(
    translation::java::ADVANCEMENT_ADVANCEMENTNOTFOUND,
    translation::java::ADVANCEMENT_ADVANCEMENTNOTFOUND,
);

pub const ERROR_INVALID_BIOME: CommandErrorType<2> = CommandErrorType::new(
    translation::java::ARGUMENT_RESOURCE_NOT_FOUND,
    translation::java::ARGUMENT_RESOURCE_NOT_FOUND,
);

/// Represents an argument type used to get a resource key from an identifier.
///
/// If you want an [`Advancement`] use [`ADVANCEMENT_REGISTRY`] and resolve it with
/// [`ResourceKeyArgument::get_advancement`] after parsing.
///
/// If you just want the [`ResourceKey`], use [`ResourceKeyArgument::get_registry_key`].
///
/// Recipes are not handled by this argument type.
pub struct ResourceKeyArgument(pub &'static Identifier);

impl ArgumentType<CommandSource> for ResourceKeyArgument {
    type Item = ResourceKey;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        let identifier = Identifier::from_reader(reader)?;
        Ok(ResourceKey::new(self.0.clone(), identifier))
    }

    fn list_suggestions(
        &self,
        context: &CommandContext,
        suggestions_builder: SuggestionsBuilder,
    ) -> Suggestions {
        if self.0 == ADVANCEMENT_REGISTRY {
            let advancements = context.server().advancement_manager.get_advancements();
            suggestions_builder
                .filter_and_suggest_iter(advancements.iter().map(ToString::to_string))
                .build()
        } else if self.0 == BIOME_REGISTRY {
            let biomes = pumpkin_data::biome::Biome::ALL
                .iter()
                .map(|biome| format!("minecraft:{}", biome.registry_id));
            suggestions_builder.filter_and_suggest_iter(biomes).build()
        } else if self.0 == DIMENSION_REGISTRY {
            let worlds = context.server().worlds.load();
            let dimensions = worlds
                .iter()
                .map(|world| world.dimension.minecraft_name.to_string());
            suggestions_builder
                .filter_and_suggest_iter(dimensions)
                .build()
        } else {
            Suggestions::empty()
        }
    }

    fn client_side_parser(&self) -> JavaClientArgumentType {
        // Vanilla has its own parser for dimensions and takes biomes as plain
        // `ResourceArgument`s; the rest are resource keys.
        if self.0 == DIMENSION_REGISTRY {
            JavaClientArgumentType::Dimension
        } else if self.0 == BIOME_REGISTRY {
            JavaClientArgumentType::Resource {
                identifier: self.0.clone(),
            }
        } else {
            JavaClientArgumentType::ResourceKey {
                identifier: self.0.clone(),
            }
        }
    }
}

/// Resolves a biome identifier to the u8 id used by chunk biome storage.
///
/// Vanilla biomes come from generated data; datapack biomes come from the dynamic
/// registry, whose ids share the same u8 space as the chunk biome palette.
fn resolve_biome_id(manager: &DynamicRegistryManager, identifier: &Identifier) -> Option<u8> {
    identifier
        .is_vanilla_then()
        .and_then(pumpkin_data::biome::Biome::from_name)
        .map(|biome| biome.id)
        .or_else(|| {
            manager
                .get_raw_id(&BIOME_REGISTRY.to_string(), &identifier.to_string())
                .and_then(|raw| u8::try_from(raw).ok())
        })
}

impl ResourceKeyArgument {
    /// Returns a [`CommandContext`]'s parsed resource key argument as an [`Advancement`].
    ///
    /// # Arguments
    /// * `context` - The [`CommandContext`] that has the parsed [`ResourceKey`] with the provided argument name.
    /// * `name` - The name of the argument that was parsed.
    ///
    /// # Returns
    /// The `Advancement` containing the advancement get from the resource key argument, wrapped in an `Ok`,
    /// or an `Err` with the appropriate [`CommandSyntaxError`] if it could not be resolved or
    /// that the key correspond to an unknown advancement.
    pub fn get_advancement(
        context: &CommandContext,
        name: &str,
    ) -> Result<&'static Advancement, CommandSyntaxError> {
        let resource_key: &ResourceKey = Self::get_registry_key(
            context,
            name,
            ADVANCEMENT_REGISTRY,
            &ERROR_INVALID_ADVANCEMENT,
        )?;
        resource_key
            .identifier
            .is_vanilla_then()
            .and_then(Advancement::from_name)
            .ok_or_else(|| {
                ERROR_INVALID_ADVANCEMENT.create_without_context(TextComponent::text(
                    resource_key.identifier.to_string(),
                ))
            })
    }

    /// Returns a [`CommandContext`]'s parsed resource key argument as a [`Biome`](pumpkin_data::biome::Biome).
    pub fn get_biome(
        context: &CommandContext,
        name: &str,
    ) -> Result<&'static pumpkin_data::biome::Biome, CommandSyntaxError> {
        let argument = context.get_argument::<ResourceKey>(name)?;
        let invalid = || {
            ERROR_INVALID_BIOME.create_without_context_args_slice(&[
                TextComponent::text(argument.identifier.to_string()),
                TextComponent::text(BIOME_REGISTRY.to_string()),
            ])
        };
        let resource_key = argument.cast(BIOME_REGISTRY).ok_or_else(invalid)?;
        resource_key
            .identifier
            .is_vanilla_then()
            .and_then(pumpkin_data::biome::Biome::from_name)
            .ok_or_else(invalid)
    }

    /// Resolves a biome resource key to the id used by chunk biome storage,
    /// including biomes added by datapacks.
    pub fn get_biome_id(context: &CommandContext, name: &str) -> Result<u8, CommandSyntaxError> {
        let argument = context.get_argument::<ResourceKey>(name)?;
        let invalid = || {
            ERROR_INVALID_BIOME.create_without_context_args_slice(&[
                TextComponent::text(argument.identifier.to_string()),
                TextComponent::text(BIOME_REGISTRY.to_string()),
            ])
        };
        let resource_key = argument.cast(BIOME_REGISTRY).ok_or_else(invalid)?;
        resolve_biome_id(
            &context.server().datapack_manager.dynamic_registry_manager,
            &resource_key.identifier,
        )
        .ok_or_else(invalid)
    }

    /// Returns a [`CommandContext`]'s parsed resource key argument in the form of a [`ResourceKey`].
    ///
    /// # Arguments
    /// * `context` - The [`CommandContext`] that has the parsed [`ResourceKey`] with the provided argument name.
    /// * `name` - The name of the argument that was parsed.
    ///
    /// # Returns
    /// The `ResourceKey` containing the key get from the argument, wrapped in an `Ok`,
    /// or an `Err` with the appropriate [`CommandSyntaxError`] if it could not be resolved
    pub fn get_registry_key<'a>(
        context: &'a CommandContext,
        name: &str,
        registry: &Identifier,
        error: &'static CommandErrorType<1>,
    ) -> Result<&'a ResourceKey, CommandSyntaxError> {
        let argument = context.get_argument::<ResourceKey>(name)?;
        argument.cast(registry).ok_or_else(|| {
            error.create_without_context(TextComponent::text(argument.identifier.to_string()))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::context::command_context::{CommandContextBuilder, ParsedArgument};
    use crate::command::context::command_source::CommandSource;
    use crate::command::context::string_range::StringRange;
    use crate::command::node::dispatcher::CommandDispatcher;
    use crate::command::node::tree::ROOT_NODE_ID;
    use std::sync::Arc;

    // The get_* helpers and biome suggestions never touch the server: get_argument
    // reads only the parsed arguments and is_vanilla_then resolves static data, so
    // a dummy source suffices for these unit tests.
    fn dummy_context<R>(f: impl FnOnce(&CommandContext<'_>) -> R) -> R {
        let dispatcher = CommandDispatcher::new();
        let builder = CommandContextBuilder::new(
            &dispatcher,
            Arc::new(CommandSource::dummy()),
            ROOT_NODE_ID,
            0,
        );
        f(&builder.build(""))
    }

    fn context_with<R>(
        name: &str,
        key: ResourceKey,
        f: impl FnOnce(&CommandContext<'_>) -> R,
    ) -> R {
        let dispatcher = CommandDispatcher::new();
        let mut builder = CommandContextBuilder::new(
            &dispatcher,
            Arc::new(CommandSource::dummy()),
            ROOT_NODE_ID,
            0,
        );
        builder.with_argument(
            name.to_string(),
            Arc::new(ParsedArgument::new(StringRange::at(0), Box::new(key))),
        );
        f(&builder.build(""))
    }

    #[test]
    fn parse_keeps_registry_and_identifier_namespace() {
        let argument = ResourceKeyArgument(BIOME_REGISTRY);
        let mut reader = StringReader::new("minecraft:forest");
        assert_eq!(
            argument.parse(&mut reader),
            Ok(ResourceKey::new(
                (*BIOME_REGISTRY).clone(),
                Identifier::from_static("minecraft", "forest")
            ))
        );
    }

    #[test]
    fn parse_defaults_bare_path_to_minecraft_namespace() {
        let argument = ResourceKeyArgument(DIMENSION_REGISTRY);
        let mut reader = StringReader::new("overworld");
        assert_eq!(
            argument.parse(&mut reader),
            Ok(ResourceKey::new(
                (*DIMENSION_REGISTRY).clone(),
                Identifier::vanilla_static("overworld")
            ))
        );
    }

    #[test]
    fn client_parser_matches_registry() {
        assert!(matches!(
            ResourceKeyArgument(DIMENSION_REGISTRY).client_side_parser(),
            JavaClientArgumentType::Dimension
        ));
        assert!(matches!(
            ResourceKeyArgument(BIOME_REGISTRY).client_side_parser(),
            JavaClientArgumentType::Resource { .. }
        ));
        assert!(matches!(
            ResourceKeyArgument(ADVANCEMENT_REGISTRY).client_side_parser(),
            JavaClientArgumentType::ResourceKey { .. }
        ));
    }

    #[test]
    fn biome_suggestions_are_filtered_by_prefix() {
        dummy_context(|context| {
            let suggestions = ResourceKeyArgument(BIOME_REGISTRY)
                .list_suggestions(context, SuggestionsBuilder::new("minecraft:bad", 0));
            assert!(!suggestions.is_empty());
            assert!(
                suggestions
                    .suggestions
                    .iter()
                    .any(|s| s.text.cached_text() == "minecraft:badlands")
            );
        });
    }

    #[test]
    fn get_registry_key_accepts_matching_and_rejects_other_registry() {
        let key = ResourceKey::new(
            (*ADVANCEMENT_REGISTRY).clone(),
            Identifier::vanilla_static("adventure/root"),
        );
        context_with("key", key, |context| {
            assert!(matches!(
                ResourceKeyArgument::get_registry_key(
                    context,
                    "key",
                    ADVANCEMENT_REGISTRY,
                    &ERROR_INVALID_ADVANCEMENT,
                ),
                Ok(k) if k.identifier == Identifier::vanilla_static("adventure/root")
            ));
            assert!(
                ResourceKeyArgument::get_registry_key(
                    context,
                    "key",
                    BIOME_REGISTRY,
                    &ERROR_INVALID_ADVANCEMENT,
                )
                .is_err()
            );
        });
    }

    #[test]
    fn get_biome_resolves_vanilla_biome() {
        let key = ResourceKey::new(
            (*BIOME_REGISTRY).clone(),
            Identifier::vanilla_static("badlands"),
        );
        context_with("biome", key, |context| {
            assert!(matches!(
                ResourceKeyArgument::get_biome(context, "biome"),
                Ok(b) if b == &pumpkin_data::biome::Biome::BADLANDS
            ));
        });
    }

    #[test]
    fn get_advancement_resolves_vanilla_advancement() {
        let key = ResourceKey::new(
            (*ADVANCEMENT_REGISTRY).clone(),
            Identifier::vanilla_static("adventure/root"),
        );
        context_with("advancement", key, |context| {
            assert!(matches!(
                ResourceKeyArgument::get_advancement(context, "advancement"),
                Ok(a) if a == pumpkin_data::Advancement::ADVENTURE_ROOT
            ));
        });
    }

    #[test]
    fn resolve_biome_id_accepts_vanilla_and_datapack() {
        use pumpkin_data::packet::CURRENT_MC_VERSION;
        use pumpkin_data::registry::RegistryEntryData;
        use std::collections::HashMap;

        let manager = DynamicRegistryManager::new(CURRENT_MC_VERSION);
        assert_eq!(
            resolve_biome_id(&manager, &Identifier::vanilla_static("badlands")),
            Some(pumpkin_data::biome::Biome::BADLANDS.id)
        );
        assert_eq!(
            resolve_biome_id(&manager, &Identifier::from_static("custom", "unknown")),
            None
        );

        let base_count = manager
            .get_registry_entries(&BIOME_REGISTRY.to_string())
            .len();
        let mut custom_biomes = HashMap::new();
        custom_biomes.insert(
            "custom:crystal".to_string(),
            RegistryEntryData {
                entry_id: "custom:crystal".to_string(),
                data: None,
            },
        );
        let mut custom = HashMap::new();
        custom.insert(BIOME_REGISTRY.to_string(), custom_biomes);
        manager.apply_custom_registries(&custom);

        assert_eq!(
            resolve_biome_id(&manager, &Identifier::from_static("custom", "crystal")),
            u8::try_from(base_count).ok()
        );
    }
}
