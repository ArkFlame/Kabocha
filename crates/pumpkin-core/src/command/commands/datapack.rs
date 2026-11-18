use std::fs;

use pumpkin_data::packet::CURRENT_MC_VERSION;
use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::NamedColor;
use pumpkin_util::version::LAST_PRE_MINOR_DATA_PACK_FORMAT;
use tracing::error;

use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::component::ComponentArgumentType;
use crate::command::argument_types::core::string::StringArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::command::suggestion::provider::{SuggestionProvider, SuggestionProviderResult};
use crate::command::suggestion::suggestions::SuggestionsBuilder;
use crate::data::datapack::{DatapackEnablePosition, DatapackError, DatapackManager};
use crate::server::Server;

const DESCRIPTION: &str = "Controls loaded data packs.";
const PERMISSION: &str = "minecraft:command.datapack";

static ERROR_UNKNOWN_DATAPACK: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_DATAPACK_UNKNOWN,
    translation::java::COMMANDS_DATAPACK_UNKNOWN,
);

static ERROR_ENABLE_FAILED: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_DATAPACK_ENABLE_FAILED,
    translation::java::COMMANDS_DATAPACK_ENABLE_FAILED,
);

static ERROR_DISABLE_FAILED: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_DATAPACK_DISABLE_FAILED,
    translation::java::COMMANDS_DATAPACK_DISABLE_FAILED,
);

static ERROR_CREATE_ALREADY_EXISTS: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_DATAPACK_CREATE_ALREADY_EXISTS,
    translation::java::COMMANDS_DATAPACK_CREATE_ALREADY_EXISTS,
);

static ERROR_CREATE_INVALID_NAME: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_DATAPACK_CREATE_INVALID_NAME,
    translation::java::COMMANDS_DATAPACK_CREATE_INVALID_NAME,
);

static ERROR_CREATE_IO_FAILURE: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_DATAPACK_CREATE_IO_FAILURE,
    translation::java::COMMANDS_DATAPACK_CREATE_IO_FAILURE,
);

/// Maps a typed datapack error onto the vanilla command error type.
const fn error_type_for(error: &DatapackError) -> &'static CommandErrorType<1> {
    match error {
        DatapackError::Unknown(_) | DatapackError::UnknownAnchor(_) => &ERROR_UNKNOWN_DATAPACK,
        DatapackError::AlreadyEnabled(_) | DatapackError::Incompatible(_) => &ERROR_ENABLE_FAILED,
        DatapackError::NotEnabled(_) | DatapackError::Embedded(_) => &ERROR_DISABLE_FAILED,
    }
}

fn get_enabled_packs(server: &Server) -> Vec<String> {
    crate::data::datapack::DatapackManager::get_enabled_packs(server)
}

fn get_available_packs(server: &Server) -> Vec<String> {
    crate::data::datapack::DatapackManager::get_available_packs(server)
}

fn format_pack(name: &str) -> TextComponent {
    TextComponent::text(format!("[{name}]")).color_named(NamedColor::Green)
}

struct AvailablePackSuggestionProvider;

impl SuggestionProvider for AvailablePackSuggestionProvider {
    fn suggest(
        &self,
        context: &CommandContext,
        mut builder: SuggestionsBuilder,
    ) -> SuggestionProviderResult {
        let server = context.server();
        // Vanilla suggests each available pack id once, quoting ids that are
        // not valid unquoted strings (DataPackCommand.UNSELECTED_PACKS).
        for pack in get_available_packs(server) {
            if pack.contains(' ') || pack.contains('/') {
                builder = builder.suggest(format!("\"{pack}\""));
            } else {
                builder = builder.suggest(pack);
            }
        }
        builder.build()
    }
}

struct EnabledPackSuggestionProvider;

impl SuggestionProvider for EnabledPackSuggestionProvider {
    fn suggest(
        &self,
        context: &CommandContext,
        mut builder: SuggestionsBuilder,
    ) -> SuggestionProviderResult {
        let server = context.server();
        // Vanilla suggests each selected pack id once (DataPackCommand.SELECTED_PACKS).
        for pack in get_enabled_packs(server) {
            if pack.contains(' ') || pack.contains('/') {
                builder = builder.suggest(format!("\"{pack}\""));
            } else {
                builder = builder.suggest(pack);
            }
        }
        builder.build()
    }
}

#[derive(Clone, Copy)]
enum ListFilter {
    All,
    Available,
    Enabled,
}

struct DatapackListExecutor(ListFilter);

impl CommandExecutor for DatapackListExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let server = context.server();
        let enabled = get_enabled_packs(server);
        let available = get_available_packs(server);

        match self.0 {
            ListFilter::All => {
                send_enabled_list(context, &enabled);
                send_available_list(context, &available);
                Ok((enabled.len() + available.len()) as i32)
            }
            ListFilter::Enabled => {
                send_enabled_list(context, &enabled);
                Ok(enabled.len() as i32)
            }
            ListFilter::Available => {
                send_available_list(context, &available);
                Ok(available.len() as i32)
            }
        }
    }
}

fn send_enabled_list(context: &CommandContext<'_>, enabled: &[String]) {
    if enabled.is_empty() {
        context.source.send_feedback(
            TextComponent::translate_cross(
                translation::java::COMMANDS_DATAPACK_LIST_ENABLED_NONE,
                translation::java::COMMANDS_DATAPACK_LIST_ENABLED_NONE,
                [],
            ),
            false,
        );
    } else {
        let packs_component =
            TextComponent::join_with_comma(enabled.iter().map(|p| format_pack(p)).collect());
        context.source.send_feedback(
            TextComponent::translate_cross(
                translation::java::COMMANDS_DATAPACK_LIST_ENABLED_SUCCESS,
                translation::java::COMMANDS_DATAPACK_LIST_ENABLED_SUCCESS,
                [
                    TextComponent::text(enabled.len().to_string()),
                    packs_component,
                ],
            ),
            false,
        );
    }
}

fn send_available_list(context: &CommandContext<'_>, available: &[String]) {
    if available.is_empty() {
        context.source.send_feedback(
            TextComponent::translate_cross(
                translation::java::COMMANDS_DATAPACK_LIST_AVAILABLE_NONE,
                translation::java::COMMANDS_DATAPACK_LIST_AVAILABLE_NONE,
                [],
            ),
            false,
        );
    } else {
        let packs_component =
            TextComponent::join_with_comma(available.iter().map(|p| format_pack(p)).collect());
        context.source.send_feedback(
            TextComponent::translate_cross(
                translation::java::COMMANDS_DATAPACK_LIST_AVAILABLE_SUCCESS,
                translation::java::COMMANDS_DATAPACK_LIST_AVAILABLE_SUCCESS,
                [
                    TextComponent::text(available.len().to_string()),
                    packs_component,
                ],
            ),
            false,
        );
    }
}

#[derive(Clone, Copy)]
enum EnablePosition {
    First,
    Last,
}

struct DatapackEnableExecutor(EnablePosition);

impl CommandExecutor for DatapackEnableExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let name_str = StringArgumentType::get(context, "name")?;
        let server = context.server();

        let position = match self.0 {
            EnablePosition::First => DatapackEnablePosition::First,
            EnablePosition::Last => DatapackEnablePosition::Last,
        };
        let resolved = match DatapackManager::enable_pack_checked(server, name_str, position) {
            Ok(resolved) => resolved,
            Err(err) => {
                let error_type = error_type_for(&err);
                return Err(
                    error_type.create_without_context(TextComponent::text(err.into_message()))
                );
            }
        };

        context.source.send_feedback(
            TextComponent::translate_cross(
                translation::java::COMMANDS_DATAPACK_MODIFY_ENABLE,
                translation::java::COMMANDS_DATAPACK_MODIFY_ENABLE,
                [format_pack(&resolved)],
            ),
            true,
        );

        Ok(get_enabled_packs(server).len() as i32)
    }
}

#[derive(Clone, Copy)]
enum BeforeOrAfter {
    Before,
    After,
}

struct DatapackEnableExistingExecutor(BeforeOrAfter);

impl CommandExecutor for DatapackEnableExistingExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let name_str = StringArgumentType::get(context, "name")?;
        let existing_str = StringArgumentType::get(context, "existing")?;
        let server = context.server();

        let position = match self.0 {
            BeforeOrAfter::Before => DatapackEnablePosition::Before(existing_str.to_string()),
            BeforeOrAfter::After => DatapackEnablePosition::After(existing_str.to_string()),
        };
        let resolved = match DatapackManager::enable_pack_checked(server, name_str, position) {
            Ok(resolved) => resolved,
            Err(err) => {
                let error_type = error_type_for(&err);
                return Err(
                    error_type.create_without_context(TextComponent::text(err.into_message()))
                );
            }
        };

        context.source.send_feedback(
            TextComponent::translate_cross(
                translation::java::COMMANDS_DATAPACK_MODIFY_ENABLE,
                translation::java::COMMANDS_DATAPACK_MODIFY_ENABLE,
                [format_pack(&resolved)],
            ),
            true,
        );

        Ok(get_enabled_packs(server).len() as i32)
    }
}

struct DatapackDisableExecutor;

impl CommandExecutor for DatapackDisableExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let name_str = StringArgumentType::get(context, "name")?;
        let server = context.server();

        let resolved = match DatapackManager::disable_pack_checked(server, name_str) {
            Ok(resolved) => resolved,
            Err(err) => {
                let error_type = error_type_for(&err);
                return Err(
                    error_type.create_without_context(TextComponent::text(err.into_message()))
                );
            }
        };

        context.source.send_feedback(
            TextComponent::translate_cross(
                translation::java::COMMANDS_DATAPACK_MODIFY_DISABLE,
                translation::java::COMMANDS_DATAPACK_MODIFY_DISABLE,
                [format_pack(&resolved)],
            ),
            true,
        );

        Ok(get_enabled_packs(server).len() as i32)
    }
}

fn is_valid_pack_name(name: &str) -> bool {
    !name.is_empty()
        && !name.contains('/')
        && !name.contains('\\')
        && !name.contains("..")
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
}

/// Builds the `pack.mcmeta` content for a newly created data pack.
///
/// Vanilla switches from the legacy `pack_format` field to `min_format`/`max_format`
/// once the format passes `LAST_PRE_MINOR_DATA_PACK_FORMAT`.
fn pack_mcmeta_content(description: &serde_json::Value) -> serde_json::Value {
    let format = CURRENT_MC_VERSION.data_pack_format();
    let pack = if format > LAST_PRE_MINOR_DATA_PACK_FORMAT {
        serde_json::json!({
            "min_format": format,
            "max_format": format,
            "description": description
        })
    } else {
        serde_json::json!({
            "pack_format": format,
            "description": description
        })
    };
    serde_json::json!({ "pack": pack })
}

struct DatapackCreateExecutor;

impl CommandExecutor for DatapackCreateExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let name_str = StringArgumentType::get(context, "id")?;
        let description = serde_json::to_value(ComponentArgumentType::get(context, "description")?)
            .unwrap_or_else(|_| serde_json::Value::String(name_str.to_owned()));

        if !is_valid_pack_name(name_str) {
            return Err(ERROR_CREATE_INVALID_NAME
                .create_without_context(TextComponent::text(name_str.to_string())));
        }

        let server = context.server();
        let datapacks_dir = server.basic_config.get_world_path().join("datapacks");
        let pack_dir = datapacks_dir.join(name_str);

        if pack_dir.exists() {
            return Err(ERROR_CREATE_ALREADY_EXISTS
                .create_without_context(TextComponent::text(name_str.to_string())));
        }

        let data_dir = pack_dir.join("data");
        if let Err(err) = fs::create_dir_all(&data_dir) {
            error!("Failed to create datapack directory: {err}");
            return Err(ERROR_CREATE_IO_FAILURE
                .create_without_context(TextComponent::text(name_str.to_string())));
        }

        let mcmeta_content = pack_mcmeta_content(&description);

        let mcmeta_path = pack_dir.join("pack.mcmeta");
        let mcmeta_str = match serde_json::to_string_pretty(&mcmeta_content) {
            Ok(s) => s,
            Err(err) => {
                error!("Failed to serialize pack.mcmeta: {err}");
                return Err(ERROR_CREATE_IO_FAILURE
                    .create_without_context(TextComponent::text(name_str.to_string())));
            }
        };

        if let Err(err) = fs::write(&mcmeta_path, mcmeta_str) {
            error!("Failed to write pack.mcmeta: {err}");
            return Err(ERROR_CREATE_IO_FAILURE
                .create_without_context(TextComponent::text(name_str.to_string())));
        }

        context.source.send_feedback(
            TextComponent::translate_cross(
                translation::java::COMMANDS_DATAPACK_CREATE_SUCCESS,
                translation::java::COMMANDS_DATAPACK_CREATE_SUCCESS,
                [TextComponent::text(name_str.to_string())],
            ),
            true,
        );

        Ok(1)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    let list_builder = literal("list")
        .then(literal("available").executes(DatapackListExecutor(ListFilter::Available)))
        .then(literal("enabled").executes(DatapackListExecutor(ListFilter::Enabled)))
        .executes(DatapackListExecutor(ListFilter::All));

    let enable_builder = literal("enable").then(
        argument("name", StringArgumentType::QuotablePhrase)
            .suggests(AvailablePackSuggestionProvider)
            .then(literal("first").executes(DatapackEnableExecutor(EnablePosition::First)))
            .then(literal("last").executes(DatapackEnableExecutor(EnablePosition::Last)))
            .then(
                literal("before").then(
                    argument("existing", StringArgumentType::QuotablePhrase)
                        .suggests(EnabledPackSuggestionProvider)
                        .executes(DatapackEnableExistingExecutor(BeforeOrAfter::Before)),
                ),
            )
            .then(
                literal("after").then(
                    argument("existing", StringArgumentType::QuotablePhrase)
                        .suggests(EnabledPackSuggestionProvider)
                        .executes(DatapackEnableExistingExecutor(BeforeOrAfter::After)),
                ),
            )
            .executes(DatapackEnableExecutor(EnablePosition::Last)),
    );

    let disable_builder = literal("disable").then(
        argument("name", StringArgumentType::QuotablePhrase)
            .suggests(EnabledPackSuggestionProvider)
            .executes(DatapackDisableExecutor),
    );

    let create_builder = literal("create").then(
        // `datapack create <id> <description>`: both required, as vanilla.
        argument("id", StringArgumentType::SingleWord)
            .then(argument("description", ComponentArgumentType).executes(DatapackCreateExecutor)),
    );

    dispatcher.register(
        command("datapack", DESCRIPTION)
            .requires(PERMISSION)
            .then(list_builder)
            .then(enable_builder)
            .then(disable_builder)
            .then(create_builder),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_pack_name_check() {
        assert!(is_valid_pack_name("my_pack"));
        assert!(is_valid_pack_name("test-1.0"));
        assert!(is_valid_pack_name("pack123"));

        assert!(!is_valid_pack_name(""));
        assert!(!is_valid_pack_name("../hack"));
        assert!(!is_valid_pack_name("folder/pack"));
        assert!(!is_valid_pack_name("pack\\test"));
        assert!(!is_valid_pack_name("invalid char"));
    }

    #[test]
    fn format_pack_brackets() {
        let formatted = format_pack("vanilla");
        assert!(formatted.to_pretty_console().contains("[vanilla]"));
    }

    #[test]
    fn pack_mcmeta_json_structure() {
        let description = serde_json::Value::String("Test description".to_string());
        let mcmeta_str = serde_json::to_string(&pack_mcmeta_content(&description)).unwrap();
        let format = CURRENT_MC_VERSION.data_pack_format();
        assert!(mcmeta_str.contains(&format!("\"min_format\":{format}")));
        assert!(mcmeta_str.contains(&format!("\"max_format\":{format}")));
        assert!(!mcmeta_str.contains("\"pack_format\""));
        assert!(mcmeta_str.contains("\"description\":\"Test description\""));
    }
}
