# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-block-blocks-command-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: command.rs command_source.rs mod.rs

FINAL terminal status. No STILL OPEN rows remain across either tracked audit.
Final: 0 STILL OPEN rows across both tracked audits. One BLOCKED-DATA row (6, Bedrock runtime harness - no generated field missing). No source file edited this pass.
FINAL PASS (2026-11-18). Every row in both tracked audits is terminal (FIXED / VERIFIED or BLOCKED-DATA); no STILL OPEN row remains. Re-read command.rs, command_source.rs, command/mod.rs and the cited decompiled vanilla files; every fix claimed in earlier passes is present and type-consistent. No production source edit was needed this pass (all fixes landed earlier), so no rustfmt or build run was required.

## Result

FINAL PASS (2026-11-18). Every row in both tracked audits ends terminal (FIXED / VERIFIED or BLOCKED-DATA); no STILL OPEN row remains. Re-read `command.rs`, `command_source.rs`, `command/mod.rs` and the cited decompiled vanilla files. No production `.rs` edit was needed this pass (all fixes landed earlier), so no rustfmt or build was run; the orchestrator compiles.

Row group 1, `own/taudit-crates-pumpkin-core-src-bl-5eb6d8` (`block/blocks/command.rs`):
- 1 `on_place` nearest-looking facing: FIXED / VERIFIED. `placement_facing()` uses `get_entity_facing_order()[0].opposite()`, matching `.vendor/vanilla-decompiled/net/minecraft/world/level/block/CommandBlock.java:185-187` (`getStateForPlacement` -> `getNearestLookingDirection().getOpposite()`). Pure test `test_placement_facing_mirrors_vanilla_nearest_looking_direction` present.
- 2 `can_place_at` game-master gate: FIXED / VERIFIED. `can_use_game_master_blocks()` guard, matching `CommandBlock implements GameMasterBlock` and `useWithoutItem` (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/CommandBlock.java:31,135`).
- 3 chain cap: FIXED / VERIFIED. `MAX_CHAIN_LENGTH = 65536` matches vanilla's default `max_command_sequence_length` (`CommandBlock.java:192`); diagnostic log level is cosmetic.
- 4 `last_output` TrackOutput gate: FIXED / VERIFIED. Output recorded only when `track_output`, matching `.vendor/vanilla-decompiled/net/minecraft/world/level/BaseCommandBlock.java:93-137,216`.
- 5 CustomName / LastExecution / UpdateLastExecution: FIXED / VERIFIED. Entity NBT plus once-per-tick guard and `command_block_name`, matching `BaseCommandBlock.java:24,52-80,95,135,147-148`. Round-trip tests present.
- 6 Bedrock editor round-trip: BLOCKED-DATA. Missing subsystem: a live Bedrock client runtime harness (`scripts/boot.sh`) to observe the editor opening and the `SBlockActorData` round-trip. No generated `pumpkin-data` field is missing. Static path is fully wired and unit-tested; client-only GUI observation does not block landing.

Row group 2, `own/taudit-crates-pumpkin-core-src-command-mo` (`command/mod.rs` / `command_source.rs`):
- 11 CustomName in Display / into_source: FIXED / VERIFIED. `command_block_name` returns custom name or `@`, matching `BaseCommandBlock#getName` (`BaseCommandBlock.java:147-148`) and `CloseableCommandBlockSource#createCommandSourceStack` (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/CommandBlockEntity.java:55-65`). Pure test present.
- 12 LastExecution / UpdateLastExecution / CustomName NBT: FIXED / VERIFIED. Entity fields + NBT read/write with round-trip tests, matching `BaseCommandBlock.java:52-80`.
- 13 `chain_execute` / conditions: FIXED / VERIFIED. `conditions_met` checks previous `SuccessCount > 0` for conditional links; once-per-tick stop matches `CommandBlockEntity#markConditionMet` (`CommandBlockEntity.java:146-159`) and `BaseCommandBlock#performCommand` (`BaseCommandBlock.java:95-97`). The "gate unconditional links on previous SuccessCount" suggestion was correctly rejected as contradicting vanilla.
- 14 admin broadcast target: FIXED / VERIFIED. `send_to_ops` (`command_source.rs:383-413`) builds gray italic `chat.type.admin`, gates on `send_command_feedback`, skips the output player and targets only op-list members via `operator_config.get_entry(...).is_some()`, matching `.vendor/vanilla-decompiled/net/minecraft/commands/CommandSourceStack.java:539-548` (`broadcastToAdmins`: `chat.type.admin` with GRAY+ITALIC, `SEND_COMMAND_FEEDBACK`, skip source, op-only permission).
- 15 console feedback: FIXED / VERIFIED. `CommandSender::Console` routes through `tracing::info!`; no `println!`/`print!`/`eprintln!` under `command/`.
- 16 `world_or_first`: VERIFIED, no change. Console/dummy sources anchor to the first (overworld) world, matching vanilla.

Final: 0 STILL OPEN rows across both tracked audits; one BLOCKED-DATA row (6, Bedrock runtime harness, no generated field missing). No source file edited this pass.
