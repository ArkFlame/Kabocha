use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;

use crate::command::argument_builder::{
    ArgumentBuilder, LiteralArgumentBuilder, argument, command, literal,
};
use crate::command::argument_types::function::FunctionArgumentType;
use crate::command::argument_types::nbt::NbtCompoundArgumentType;
use crate::command::argument_types::nbt_path::{NbtPath, NbtPathArgumentType};
use crate::command::commands::data::{TargetKind, get_single_tag, make_target_arg};
use crate::command::context::command_context::CommandContext;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::{CommandErrorType, LiteralCommandErrorType};
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::command::suggestion::provider::{SuggestionProvider, SuggestionProviderResult};
use crate::command::suggestion::suggestions::SuggestionsBuilder;
use crate::data::datapack::{FunctionRunError, is_nested_function_call};

const DESCRIPTION: &str = "Runs commands found in the corresponding function files.";
const PERMISSION: &str = "minecraft:command.function";

static ERROR_UNKNOWN_FUNCTION: CommandErrorType<1> = CommandErrorType::new(
    translation::java::ARGUMENTS_FUNCTION_UNKNOWN,
    translation::java::ARGUMENTS_FUNCTION_UNKNOWN,
);

static ERROR_UNKNOWN_FUNCTION_TAG: CommandErrorType<1> = CommandErrorType::new(
    translation::java::ARGUMENTS_FUNCTION_TAG_UNKNOWN,
    translation::java::ARGUMENTS_FUNCTION_TAG_UNKNOWN,
);

static ERROR_CHAIN_LIMIT_EXCEEDED: LiteralCommandErrorType =
    LiteralCommandErrorType::new("Maximum number of nested function calls reached");

static ERROR_COMMAND_CHAIN_LIMIT_EXCEEDED: LiteralCommandErrorType =
    LiteralCommandErrorType::new("Maximum number of commands in a function chain reached");

struct FunctionSuggestionProvider;

impl SuggestionProvider for FunctionSuggestionProvider {
    fn suggest(
        &self,
        context: &CommandContext,
        mut builder: SuggestionsBuilder,
    ) -> SuggestionProviderResult {
        let server = context.server();
        let function_names = server.datapack_manager.get_function_names();
        for name in function_names {
            builder = builder.suggest(name);
        }
        builder.build()
    }
}

static ERROR_ARGUMENT_NOT_COMPOUND: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_FUNCTION_ERROR_ARGUMENT_NOT_COMPOUND,
    translation::java::COMMANDS_FUNCTION_ERROR_ARGUMENT_NOT_COMPOUND,
);

static ERROR_INSTANTIATION_FAILURE: CommandErrorType<2> = CommandErrorType::new(
    translation::java::COMMANDS_FUNCTION_INSTANTIATIONFAILURE,
    translation::java::COMMANDS_FUNCTION_INSTANTIATIONFAILURE,
);

/// Mirrors vanilla `TagType.getName()`.
const fn tag_type_name(tag: &NbtTag) -> &'static str {
    match tag {
        NbtTag::End => "END",
        NbtTag::Byte(_) => "BYTE",
        NbtTag::Short(_) => "SHORT",
        NbtTag::Int(_) => "INT",
        NbtTag::Long(_) => "LONG",
        NbtTag::Float(_) => "FLOAT",
        NbtTag::Double(_) => "DOUBLE",
        NbtTag::ByteArray(_) => "BYTE[]",
        NbtTag::String(_) => "STRING",
        NbtTag::List(_) => "LIST",
        NbtTag::Compound(_) => "COMPOUND",
        NbtTag::IntArray(_) => "INT[]",
        NbtTag::LongArray(_) => "LONG[]",
    }
}

#[derive(Clone, Copy)]
enum FunctionArguments {
    None,
    Inline,
    Source { kind: TargetKind, has_path: bool },
}

struct FunctionExecutor(FunctionArguments);

impl FunctionExecutor {
    fn arguments(
        &self,
        context: &CommandContext,
    ) -> Result<Option<NbtCompound>, CommandSyntaxError> {
        match self.0 {
            FunctionArguments::None => Ok(None),
            FunctionArguments::Inline => Ok(Some(
                NbtCompoundArgumentType::get(context, "arguments")?.clone(),
            )),
            FunctionArguments::Source { kind, has_path } => {
                let accessor = kind.access(context, "source")?;
                if !has_path {
                    return Ok(Some(accessor.get_data()?));
                }
                let path = context.get_argument::<NbtPath>("path")?;
                match get_single_tag(path, accessor.as_ref())? {
                    NbtTag::Compound(compound) => Ok(Some(compound)),
                    other => Err(ERROR_ARGUMENT_NOT_COMPOUND
                        .create_without_context(TextComponent::text(tag_type_name(&other)))),
                }
            }
        }
    }
}

impl CommandExecutor for FunctionExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let name_str = FunctionArgumentType::get(context, "name")?;
        let server = context.server();
        let arguments = self.arguments(context)?;

        let result = match server.datapack_manager.execute_function_with(
            server,
            &context.source,
            name_str,
            arguments.as_ref(),
        ) {
            Ok(result) => result,
            Err(FunctionRunError::ChainLimitExceeded) => {
                return Err(ERROR_CHAIN_LIMIT_EXCEEDED.create_without_context());
            }
            Err(FunctionRunError::CommandChainLimitExceeded) => {
                return Err(ERROR_COMMAND_CHAIN_LIMIT_EXCEEDED.create_without_context());
            }
            Err(FunctionRunError::Unknown(_)) => {
                // Vanilla resolves the name while parsing: `FunctionArgument` reports a bad
                // `#tag` with the tag-specific message and a plain id with the function one.
                let error_type = if name_str.starts_with('#') {
                    &ERROR_UNKNOWN_FUNCTION_TAG
                } else {
                    &ERROR_UNKNOWN_FUNCTION
                };
                return Err(
                    error_type.create_without_context(TextComponent::text(name_str.to_string())),
                );
            }
            Err(FunctionRunError::Instantiation(message)) => {
                return Err(ERROR_INSTANTIATION_FAILURE.create_without_context(
                    TextComponent::text(name_str.to_string()),
                    TextComponent::text(message),
                ));
            }
        };
        if let Some(value) = result.returned {
            return Ok(value);
        }
        let executed_count = result.executed;
        pumpkin_command::context::command_context::skip_result_callbacks();

        // Only the outermost call reports a result, or unwinding prints one line per level.
        if !is_nested_function_call() {
            // Like vanilla 26.3, report which functions ran; the client dropped the
            // `commands.function.success.*` keys.
            let functions = server
                .datapack_manager
                .function_tag_members(name_str)
                .unwrap_or_else(|| vec![name_str.to_string()]);
            context
                .source
                .send_feedback(scheduled_message(&functions, name_str), true);
        }

        Ok(executed_count as i32)
    }
}

/// Vanilla's `commands.function.scheduled.*` feedback for the functions a name expanded to.
fn scheduled_message(functions: &[String], name: &str) -> TextComponent {
    match functions {
        // An existing function tag may still expand to nothing.
        [] => TextComponent::translate_cross(
            translation::java::COMMANDS_FUNCTION_SCHEDULED_NO_FUNCTIONS,
            translation::java::COMMANDS_FUNCTION_SCHEDULED_NO_FUNCTIONS,
            [TextComponent::text(name.to_string())],
        ),
        [single] => TextComponent::translate_cross(
            translation::java::COMMANDS_FUNCTION_SCHEDULED_SINGLE,
            translation::java::COMMANDS_FUNCTION_SCHEDULED_SINGLE,
            [TextComponent::text(single.clone())],
        ),
        _ => TextComponent::translate_cross(
            translation::java::COMMANDS_FUNCTION_SCHEDULED_MULTIPLE,
            translation::java::COMMANDS_FUNCTION_SCHEDULED_MULTIPLE,
            [TextComponent::text(functions.join(", "))],
        ),
    }
}

fn with_sources() -> LiteralArgumentBuilder {
    let mut with = literal("with");
    for (kind, name) in [
        (TargetKind::Block, "block"),
        (TargetKind::Entity, "entity"),
        (TargetKind::Storage, "storage"),
    ] {
        with = with.then(
            literal(name).then(
                make_target_arg(kind, "source_target")
                    .executes(FunctionExecutor(FunctionArguments::Source {
                        kind,
                        has_path: false,
                    }))
                    .then(
                        argument("path", NbtPathArgumentType).executes(FunctionExecutor(
                            FunctionArguments::Source {
                                kind,
                                has_path: true,
                            },
                        )),
                    ),
            ),
        );
    }
    with
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher.register(
        command("function", DESCRIPTION).requires(PERMISSION).then(
            argument("name", FunctionArgumentType)
                .suggests(FunctionSuggestionProvider)
                .executes(FunctionExecutor(FunctionArguments::None))
                .then(
                    argument("arguments", NbtCompoundArgumentType)
                        .executes(FunctionExecutor(FunctionArguments::Inline)),
                )
                .then(with_sources()),
        ),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn serialized(component: &TextComponent) -> String {
        serde_json::to_string(component).expect("text components serialize")
    }

    #[test]
    fn scheduled_message_reports_an_empty_tag_as_no_functions() {
        let json = serialized(&scheduled_message(&[], "#test:empty"));
        assert!(json.contains(translation::java::COMMANDS_FUNCTION_SCHEDULED_NO_FUNCTIONS));
        assert!(json.contains("#test:empty"));
    }

    #[test]
    fn scheduled_message_picks_single_and_multiple_variants() {
        let single = serialized(&scheduled_message(&["test:a".to_string()], "test:a"));
        assert!(single.contains(translation::java::COMMANDS_FUNCTION_SCHEDULED_SINGLE));
        let multiple = serialized(&scheduled_message(
            &["test:a".to_string(), "test:b".to_string()],
            "#test:pair",
        ));
        assert!(multiple.contains(translation::java::COMMANDS_FUNCTION_SCHEDULED_MULTIPLE));
        assert!(multiple.contains("test:a, test:b"));
    }
}
