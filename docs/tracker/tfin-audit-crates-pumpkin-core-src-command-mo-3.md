# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-command-mo.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

Final row status: 11 FIXED, 12 FIXED, 13 FIXED, 14 FIXED, 15 FIXED, 16 VERIFIED. No BLOCKED-DATA rows (no missing generated fields or subsystems). No STILL OPEN rows.

## Result
Final pass. Re-read every relevant source file and re-checked each claim against decompiled vanilla at `.vendor/vanilla-decompiled/net/minecraft/`. All six rows end FIXED or VERIFIED; no BLOCKED-DATA. No source edit was needed this pass (all fixes already present in the checkout), so no rustfmt run.

- Row 11 (CustomName in `CommandSender` Display / `into_source`): FIXED. `command_block_name` (`crates/pumpkin-core/src/command/mod.rs:129-136`) returns the entity `custom_name` or `@`; used by `Display` (line 143) and `into_source` (lines 395, 403-404). Matches vanilla `CommandBlockEntity.CloseableCommandBlockSource#createCommandSourceStack` passing `this.getName()` (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/CommandBlockEntity.java:55-65`) and `BaseCommandBlock#getName` returning `customName` or `DEFAULT_NAME` = `@` (`.vendor/vanilla-decompiled/net/minecraft/world/level/BaseCommandBlock.java:24,147-148`). Unit test `command_block_name_matches_vanilla_get_name` (mod.rs:469) covers both branches.

- Row 12 (LastExecution / UpdateLastExecution / CustomName NBT): FIXED. `CommandBlockEntity` fields (`last_execution: AtomicI64`, `update_last_execution: AtomicBool`, `custom_name`; `block/entities/command_block.rs:14-46`), NBT read/write (58-160) and the three round-trip tests (lines 265-360) match vanilla `BaseCommandBlock` save/load (`.vendor/vanilla-decompiled/net/minecraft/world/level/BaseCommandBlock.java:50-82`): CustomName via codec, LastOutput only when TrackOutput, LastExecution written only when UpdateLastExecution and not -1, restored to -1 otherwise.

- Row 13 (`chain_execute` / conditions): FIXED. `conditions_met` (`block/blocks/command.rs:61-83`) returns true for unconditional blocks and checks the block behind facing + `SuccessCount > 0` for conditional ones, matching vanilla `CommandBlockEntity.markConditionMet` (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/CommandBlockEntity.java:146-152`). The once-per-tick guard `last_execution == world_age` in `execute` (lines 163-166) and the chain break (240-244) match vanilla `BaseCommandBlock.performCommand` (`.vendor/vanilla-decompiled/net/minecraft/world/level/BaseCommandBlock.java:93-97`) and `CommandBlock.executeChain` stopping when `performCommand` returns false (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/CommandBlock.java:204-209`). The row's original suggestion to gate unconditional links on previous SuccessCount contradicts vanilla and stays rejected. No pure unit-test seam (`chain_execute` needs a live `World`/`Server`).

- Row 14 (admin broadcast target): FIXED. `send_to_ops` (`command/context/command_source.rs:383-413`) targets op-list members via `operator_config.read().get_entry(&player.gameprofile.id).is_some()` instead of a permission-level threshold, matching vanilla `broadcastToAdmins` gating on op status (`.vendor/vanilla-decompiled/net/minecraft/commands/CommandSourceStack.java:539-548`; Paper `hasPermission("minecraft.admin.command_feedback")`). No pure unit-test seam (needs a live `Server`).

- Row 15 (console feedback via tracing): FIXED. `CommandSender::Console` routes through `tracing::info!` (`command/mod.rs:154`); no `println!`/`print!`/`eprintln!` remains under `command/`. Matches vanilla `MinecraftServer#sendSystemMessage` logging through the server LOGGER (`.vendor/vanilla-decompiled/net/minecraft/commands/CommandSourceStack.java:550-551`).

- Row 16 (`world_or_first`): VERIFIED, no change. Console/dummy sources anchor to the overworld, falling back to the first loaded world (`command/mod.rs:299-303`), matching vanilla `CommandSourceStack` level selection.

Final row status: 11 FIXED, 12 FIXED, 13 FIXED, 14 FIXED, 15 FIXED, 16 VERIFIED. No BLOCKED-DATA rows (no missing generated fields or subsystems). No STILL OPEN rows.
