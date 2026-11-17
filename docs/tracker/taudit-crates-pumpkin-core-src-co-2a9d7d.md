# Audit: crates/pumpkin-core/src/command/commands/function.rs

Vanilla reference: `net.minecraft.commands.Commands`/`FunctionCommand` (26.3 command tree), messages from
`assets/en_us_java.json`. Paper/Spigot keep vanilla behaviour for `/function`.

## Fixed in this branch

| Gap | Vanilla | Fix |
|---|---|---|
| Unknown `#tag` used `arguments.function.unknown` | `FunctionArgument.ERROR_UNKNOWN_TAG` -> `arguments.function.tag.unknown` ("Unknown function tag '%s'") | `function.rs` picks the error type from the `#` prefix (commit 319cdaf8d) |
| Empty function tag sent `commands.function.scheduled.multiple` with an empty list ("Running functions ") | `commands.function.scheduled.no_functions` ("Can't find any functions for name %s") | `scheduled_message()` helper, unit tests (commit 6dce2ec65) |

## Remaining gaps

### 1. `with <block|entity|storage>` accepts a missing `<path>` (medium)
`crates/pumpkin-core/src/command/commands/function.rs:208-234` builds the `with` tree with
`make_target_arg(...)` + optional `path`, and `FunctionArguments::Source { has_path: false }`
(`:109-114`) passes the whole data compound when no path is typed. Vanilla's
`FunctionCommand.registerWith` requires `<path>` for all three sources
(`/function <name> with block <pos> <path>`), so Pumpkin accepts input vanilla rejects at parse time.
Suggested fix: drop the `has_path: false` executors (and the `has_path` field), keeping only
`argument("path", NbtPathArgumentType)`; `TargetKind::access(..).get_data()` then always goes through
`get_single_tag`. Size: S.

### 2. Function return value is not reported (medium)
`function.rs:163-165` returns `Ok(value)` and sends no feedback. Current vanilla prints
`commands.function.result` ("Function %s returned %s") when the function body ends in `return`;
the generated keys `COMMANDS_FUNCTION_RESULT`, `COMMANDS_FUNCTION_SUCCESS_SINGLE_RESULT` and
`COMMANDS_FUNCTION_SUCCESS_MULTIPLE_RESULT` are unused in the tree. Needs the vanilla rule for which
name is printed (called name vs each expanded function id) before implementing. Size: S-M.

### 3. Macro instantiation errors use the wrong keys (medium)
`crates/pumpkin-core/src/data/datapack/mod.rs:1205` and `substitute_macro` return
`FunctionRunError::Instantiation(msg)`, and `function.rs:157-161` always wraps it in
`commands.function.instantiationFailure` ("Failed to instantiate function %s: %s"). Vanilla separates:
- macro called without arguments -> `commands.function.error.missing_arguments` ("Missing arguments to function %s")
- `$` placeholder not supplied -> `commands.function.error.missing_argument` ("Missing argument %2$s to function %1$s")
- substituted line fails to parse -> `commands.function.error.parse` ("While instantiating macro %s: Command '%s' caused error: %s")
Suggested fix: split `FunctionRunError::Instantiation` into `MissingArguments`, `MissingArgument(name)` and
`ParseMacro { function, line, error }`, map each to its own `CommandErrorType`, and pass the argument name
from `substitute_macro`. Size: M.

### 4. Function name resolved at execution, not at parse time (low)
`function.rs:127-161` looks the name up in `execute_function_with`; vanilla `FunctionArgument.functions()`
resolves during parsing, so an unknown function fails the command as a syntax error and never reaches the
executor. Observable difference: parse-vs-execute error phase, `/execute ... run function <bad>` handling and
error nesting. Suggested fix: resolve `name` (and reject unknown plain ids / tags) inside
`FunctionArgumentType::parse` via `DatapackManager`; needs dispatcher access to the server, so keep until a
larger argument-type refactor. Size: M.

### 5. Chain limits are hardcoded, not gamerule driven (low, documented)
`data/datapack/mod.rs:116-122` uses `MAX_FUNCTION_CHAIN_DEPTH = 256` (vanilla has no depth cap beyond
`maxCommandChainLength`) and a fixed 65536 command budget instead of the `maxCommandChainLength` gamerule.
The comment there already explains the stack-overflow reason for the depth cap. Fixing the command budget
requires reading the gamerule in `execute_function_with`. Size: S-M.
