# Parity rows for crates/pumpkin-core/src/block/blocks/command.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-bl-5eb6d8** [partial/OWN-OPEN] own audit: 8 open, 6 fixed, 4 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-bl-5eb6d8.md) docs/tracker/taudit-crates-pumpkin-core-src-bl-5eb6d8.md
- **own/taudit-crates-pumpkin-core-src-command-mo** [partial/OWN-OPEN] own audit: 7 open, 11 fixed, 2 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-command-mo.md) docs/tracker/taudit-crates-pumpkin-core-src-command-mo.md

## Result
FINAL terminal status. No STILL OPEN rows remain across either tracked audit.

Row group 1, own/taudit-crates-pumpkin-core-src-bl-5eb6d8 (block/blocks/command.rs):
- 1 on_place nearest-looking facing: FIXED / VERIFIED - placement_facing() uses get_entity_facing_order()[0].opposite() (command.rs:31,288-289), matching .vendor/vanilla-decompiled/net/minecraft/world/level/block/CommandBlock.java:185-186. Pure test placement_facing_mirrors_vanilla_nearest_looking_direction present.
- 2 can_place_at + normal_use GameMaster gate: FIXED / VERIFIED - can_use_game_master_blocks() and Consume (command.rs:293-305,405-413), matching CommandBlock.java:31,135.
- 3 chain cap: FIXED / VERIFIED - MAX_CHAIN_LENGTH = 65536 with debug! (command.rs:25,264-268), matching the default maxCommandChainLength.
- 4 last_output TrackOutput gate: FIXED / VERIFIED - send_message early-returns when track_output is false (command/mod.rs:156-163), matching .vendor/vanilla-decompiled/net/minecraft/world/level/BaseCommandBlock.java.
- 5 CustomName / LastExecution / UpdateLastExecution: FIXED / VERIFIED - entity NBT plus the once-per-tick guard (command.rs:161-203) plus command_block_name (command/mod.rs:129-136), matching BaseCommandBlock.java:24,60,78,95,135,147-148. Round-trip tests present.
- 6 Bedrock editor round-trip: BLOCKED-DATA. Missing subsystem: a live Bedrock client runtime harness (scripts/boot.sh) to observe the editor opening and the SBlockActorData round-trip. No generated pumpkin-data field is missing. The static path is fully wired and unit-tested.

Row group 2, own/taudit-crates-pumpkin-core-src-command-mo (command/mod.rs):
- 11 CustomName in Display / into_source: FIXED / VERIFIED - command_block_name (command/mod.rs:129-136) used by Display and into_source, matching BaseCommandBlock.java:147-148. Pure test command_block_name_matches_vanilla_get_name present.
- 12 LastExecution / UpdateLastExecution / CustomName NBT: FIXED / VERIFIED - entity fields and NBT read/write with round-trip tests, matching BaseCommandBlock.java:50-82.
- 13 chain_execute / conditions: FIXED / VERIFIED - conditions_met and the once-per-tick stop (command.rs:61-83,240-244), matching .vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/CommandBlockEntity.java:146-159 and BaseCommandBlock.java:95-97. The original gate-unconditional-on-SuccessCount suggestion contradicts vanilla and stays rejected.
- 14 admin broadcast target: FIXED / VERIFIED - send_to_ops checks op-list membership (command_source.rs:383-413), matching .vendor/vanilla-decompiled/net/minecraft/commands/CommandSourceStack.java:539-548.
- 15 console feedback: FIXED / VERIFIED - CommandSender::Console uses tracing::info! (command/mod.rs:154); no println!/print!/eprintln! remains under command/.
- 16 world_or_first: VERIFIED, no change - console/dummy sources anchor to the overworld.

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

## Result

FINAL PASS (2026-11-18). Every row in both tracked audits is terminal (FIXED / VERIFIED or BLOCKED-DATA); no STILL OPEN row remains. Re-read command.rs, command_source.rs, command/mod.rs and the cited decompiled vanilla files; every fix claimed in earlier passes is present and type-consistent. No production source edit was needed this pass (all fixes landed earlier), so no rustfmt or build run was required.

Verified vanilla anchors under .vendor/vanilla-decompiled/net/minecraft/:
- world/level/block/CommandBlock.java:185-186: getStateForPlacement -> getNearestLookingDirection().getOpposite() (row 1); CommandBlock implements GameMasterBlock (row 2).
- world/level/BaseCommandBlock.java:24 DEFAULT_NAME = Component.literal("@"); :60,78 LastExecution save/load; :95 getGameTime() == lastExecution; :135 lastExecution update; :147-148 getName custom name or @ (rows 4,5,11,12).
- world/level/block/entity/CommandBlockEntity.java:146-159: markConditionMet (row 13).
- commands/CommandSourceStack.java:539-548: broadcastToAdmins gating on op status (row 14).

All cited vanilla files exist. Only row 6 is BLOCKED-DATA, and that is a live Bedrock client runtime harness, not a missing generated pumpkin-data field; it is a client-only GUI observation that does not block landing. Tracker doc finalized; no .rs file changed this pass.
