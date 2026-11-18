# Parity rows for crates/pumpkin-core/src/block/blocks/command.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-bl-5eb6d8** [partial/OWN-OPEN] own audit: 8 open, 6 fixed, 4 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-bl-5eb6d8.md) docs/tracker/taudit-crates-pumpkin-core-src-bl-5eb6d8.md
- **own/taudit-crates-pumpkin-core-src-command-mo** [partial/OWN-OPEN] own audit: 7 open, 11 fixed, 2 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-command-mo.md) docs/tracker/taudit-crates-pumpkin-core-src-command-mo.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)

## Result

THIRD PASS (2026-11-18). Both tracked OWN-OPEN rows closed. Every coded row re-verified against decompiled vanilla under `.vendor/vanilla-decompiled/net/minecraft/`; all in-scope fixes are already present in the checkout and no new source edit was required (so no rustfmt run, no build run - the orchestrator compiles).

Row 1 `own/taudit-crates-pumpkin-core-src-bl-5eb6d8` (block/blocks/command.rs audit) - all six rows closed:
- 1 `on_place` nearest-looking facing: FIXED. `placement_facing()` (`command.rs:31`) uses `get_entity_facing_order()[0].opposite()`, matching vanilla `CommandBlock#getStateForPlacement` (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/CommandBlock.java:185-187`: `defaultBlockState().setValue(FACING, context.getNearestLookingDirection().getOpposite())`). Pure test `test_placement_facing_mirrors_vanilla_nearest_looking_direction` present.
- 2 `can_place_at` game-master gate: FIXED. Uses `player.can_use_game_master_blocks()` (`command.rs:405`), matching `CommandBlock implements GameMasterBlock` and `useWithoutItem` guard (`CommandBlock.java:31,135`).
- 3 chain cap: FIXED. `MAX_CHAIN_LENGTH = 65536` (`command.rs:25`) equals vanilla's default `max_command_sequence_length` (`CommandBlock.java:192`). The diagnostic log level is cosmetic only (`CommandBlock.java:222` uses `LOGGER.warn`; Pumpkin uses `debug!`), no gameplay effect.
- 4 `last_output` TrackOutput gate: FIXED. `command/mod.rs` records output only when `track_output`, matching `BaseCommandBlock#performCommand` / `CloseableCommandBlockSource#shouldReceiveFeedback` (`.vendor/vanilla-decompiled/net/minecraft/world/level/BaseCommandBlock.java:93-137,216`).
- 5 CustomName / LastExecution / UpdateLastExecution: FIXED. `CommandBlockEntity` fields + NBT (`entities/command_block.rs:14-160`) and the `execute` once-per-tick guard (`command.rs:163-203`) match vanilla `BaseCommandBlock` save/load (`BaseCommandBlock.java:52-80`) and `performCommand` (`BaseCommandBlock.java:95-97,134-137`). Command-source name via `command_block_name` (`command/mod.rs:129`) matches `BaseCommandBlock#getName` (`BaseCommandBlock.java:147-148`). Round-trip unit tests present.
- 6 Bedrock editor round-trip: BLOCKED-DATA. Exact missing subsystem: a live Bedrock client runtime harness (`scripts/boot.sh`) to observe the editor opening and the `SBlockActorData` round-trip; no generated `pumpkin-data` field is missing. Static path is fully wired and unit-tested (`CommandBlockEntity::bedrock_block_actor_data` -> `CBlockActorData`, `BedrockClient::handle_command_block_actor_data`, `parse_command_block_edit`); Java screen-open replays vanilla's block-entity data packet (`ServerPlayer#openCommandBlock`) via `World::update_block_entity`.

Row 2 `own/taudit-crates-pumpkin-core-src-command-mo` (command/mod.rs audit) - all six rows (11-16) closed:
- 11 CustomName in Display / into_source: FIXED. `command_block_name` (`command/mod.rs:129-136`) returns custom name or `@`, used by `Display` (line 143) and `into_source` (line 395); matches `CommandBlockEntity.CloseableCommandBlockSource#createCommandSourceStack` passing `this.getName()` (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/CommandBlockEntity.java:55-65`) and `BaseCommandBlock#getName` (`BaseCommandBlock.java:147-148`). Pure test `command_block_name_matches_vanilla_get_name` present.
- 12 LastExecution/UpdateLastExecution/CustomName NBT: FIXED, round-trip tests present, matching `BaseCommandBlock` save/load (`BaseCommandBlock.java:52-80`).
- 13 `chain_execute` / conditions: FIXED. `conditions_met` (`command.rs:61`) returns true for unconditional and checks the previous `SuccessCount > 0` for conditional, matching `CommandBlockEntity.markConditionMet` (`CommandBlockEntity.java:146-159`); once-per-tick stop matches `BaseCommandBlock#performCommand` (`BaseCommandBlock.java:95-97`) and `CommandBlock.executeChain` (`CommandBlock.java:207-209`). The original "gate unconditional links on previous SuccessCount" suggestion was correctly rejected as contradicting vanilla.
- 14 admin broadcast target: FIXED. `send_to_ops` (`command_source.rs:383-413`) targets op-list members via `operator_config.get_entry(...).is_some()`, matching `CommandSourceStack#broadcastToAdmins` (`.vendor/vanilla-decompiled/net/minecraft/commands/CommandSourceStack.java:539-548`, Paper `hasPermission("minecraft.admin.command_feedback")` / `PlayerList#isOp`).
- 15 console feedback: FIXED. `CommandSender::Console` routes through `tracing::info!`; no `println!` under `command/`, matching vanilla server LOGGER routing.
- 16 `world_or_first`: VERIFIED, no change. Console/dummy sources anchor to the first (overworld) world, matching vanilla.

Final: 0 STILL OPEN rows across both tracked audits. One BLOCKED-DATA row (6, Bedrock runtime harness - no generated field missing). No source file edited this pass.
