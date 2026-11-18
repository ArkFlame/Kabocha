# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-bl-5eb6d8.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: command.rs command_block.rs

- 1. `on_place` horizontal facing only (`command.rs`): STILL OPEN (S).
- 5. Block entity dependency (`entities/command_block.rs:10`): STILL OPEN (owned by block entity).
- 6. Bedrock editor round-trip: STILL OPEN (needs Bedrock client check).
- 5. Block entity `LastExecution`/`CustomName` (`entities/command_block.rs`): STILL OPEN. Reason: size-M cross-cutting change, needs `world.get_game_time()` threaded through `CommandBlock::execute` plus new atomic fields and NBT/GUI wiring across the block entity, command source and editor; exceeds a single-pass S fix. Confirmed against `BaseCommandBlock#performCommand` (`if (level.getGameTime() == this.lastExecution) return false`, `UpdateLastExecution`).

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
