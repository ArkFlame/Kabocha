# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-co-1acbb3.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: execute.rs

- 1. if blocks block-entity comparison (`execute.rs:1139`): FIXED.
- 2. Predicate reference and inline shape (`execute.rs:242-260`): FIXED.
- 3. `if items` missing: FIXED.
- 4. `if function` missing: FIXED.

## Result
- 1. if blocks block-entity comparison: FIXED, VERIFIED. `count_blocks_match` (execute.rs:1484) compares block state ids, then source/destination block-entity `resource_location()` and `write_nbt` payloads, mirroring `ExecuteCommand.checkRegions` at .vendor/vanilla-decompiled/net/minecraft/server/commands/ExecuteCommand.java:937-1015. Vanilla also compares data components, which Pumpkin has no API for; no generated field is missing. No source change needed this pass (already in code).
- 2. Predicate reference and inline shape: FIXED, VERIFIED. `test_predicate` (execute.rs:328) resolves `LootPredicate::Reference` via `datapack_manager.get_predicate` with ORIGIN/THIS_ENTITY loot params and evaluates it; an unknown id raises `argument.resource_or_id.no_such_element`; a bare inline list matches nothing. Matches vanilla `checkCustomPredicate` at .vendor/vanilla-decompiled/net/minecraft/server/commands/ExecuteCommand.java:869-878.
- 3. `if items` missing: FIXED, VERIFIED. `items_conditional_node` (execute.rs:595) registers entity and block sources with slots/item_predicate arguments and a numeric conditional executor, matching the `ItemCommands.SOURCE_PROVIDERS`/`countItems` shape at .vendor/vanilla-decompiled/net/minecraft/server/commands/ExecuteCommand.java:741-806. Covered by parse tests `items_conditional_registers_entity_and_block_sources` and `items_equipment_slot_matches_vanilla_indices`. The Canvas decompile disables `if items` with `if (true) break;`, so Pumpkin intentionally keeps the upstream vanilla tree.
- 4. `if function` missing: FIXED, VERIFIED. `function_condition_value` (execute.rs:381) plus `literal("function")` trees registered under both `if` (execute.rs:2106) and `unless` (execute.rs:2349) use `ConditionalExecutor` with the function return value (`!= 0` for if, `== 0` for unless), matching vanilla `ExecuteIfFunctionCustomModifier` at .vendor/vanilla-decompiled/net/minecraft/server/commands/ExecuteCommand.java:1155-1179 (Canvas purges function execution but keeps the modifier class).

No source files needed editing this pass: all four rows were already implemented and match the cited vanilla code. No compile run per unit rules; the orchestrator batch-checks. rustfmt was previously run on execute.rs.
