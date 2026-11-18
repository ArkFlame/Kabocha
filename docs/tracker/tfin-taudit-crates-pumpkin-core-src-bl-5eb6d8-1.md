# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-bl-5eb6d8.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: command.rs command_block.rs mod.rs vine.rs

- 6. Bedrock editor round-trip: STILL OPEN. Reason: needs a live Bedrock client runtime check (`scripts/boot.sh`), outside a CODER unit.
2026-11-18 recheck of rows previously marked STILL OPEN. All in-scope code fixes verified against decompiled vanilla; no new .rs change was needed.
- 5. Block entity `CustomName` / `LastExecution` (`entities/command_block.rs` + `blocks/command.rs`): FIXED in this unit's files (verified against vanilla `BaseCommandBlock#performCommand` / `#save` / `#load`). `CommandBlockEntity` already carries `custom_name`, `last_execution` and `update_last_execution` with NBT round-trip tests, and `CommandBlock::execute` early-returns when `world.get_world_age() == last_execution` then records the tick under the `update_last_execution` guard, exactly matching vanilla. Remaining sliver STILL OPEN: the command-source name is still hardcoded `@` (`command/mod.rs:388` TODO in `into_source`) and the GUI title lives in the editor. Reason: both edits sit in `command/mod.rs` and the editor, outside this unit's editable files (`command.rs`, `command_block.rs`, `vine.rs`).
- 6. Bedrock editor round-trip: STILL OPEN. Reason: needs a live Bedrock client runtime check (`scripts/boot.sh`), outside a CODER unit and with no vanilla-source or pure-unit-test seam.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
