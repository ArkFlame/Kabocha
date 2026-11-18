# audit / crates/pumpkin-core/src/command/mod.rs

Reviewed: `crates/pumpkin-core/src/command/mod.rs` (CommandSender, feedback routing, command-block source), plus call sites `command/context/command_source.rs`, `block/blocks/command.rs`, `block/entities/command_block.rs`.

Fixed: command-block source yaw mapped `facing=up/down` to 270, while vanilla `Direction#toYRot` returns 0 for `UP`/`DOWN`; the same wrong mapping was duplicated inline in `into_source` and `rotation()`. Both now use one `command_block_y_rot`. Command-block feedback was gated on the `sendCommandFeedback` gamerule instead of vanilla `TrackOutput`, so a command block with TrackOutput on stopped recording `LastOutput` whenever that gamerule was off (and recorded it with TrackOutput off). `send_message` now honours `CommandBlockEntity::track_output`, `should_receive_feedback` no longer applies the player-only gamerule to command blocks. Commit: fix(command): command block yaw and TrackOutput feedback.

## Remaining gaps

| file:line | vanilla | suggested fix | size |
|---|---|---|---|
| mod.rs:381 (TODO) | `CommandBlockEntity#getName` uses the block's custom name when set, otherwise `@` | add `CustomName` to `CommandBlockEntity` (see `block/entities/command_block.rs:11` todo), persist in NBT, and use it in `into_source`/`Display` | M |
| block/entities/command_block.rs:11 | `LastExecution` / `UpdateLastExecution` and `CustomName` fields missing; `LastOutput` also carries the `TrackOutput` semantics only when recorded | add `last_execution: AtomicI64`, write both NBT tags, update on execution | M |
| block/blocks/command.rs:165 (`chain_execute`) | vanilla chain command blocks only run when the previous block's result was non-zero (`SuccessCount`), and auto blocks reset `SuccessCount` each tick | use `success_count` from the previous link as the condition when unconditional, keep `condition_met` for conditional links | S-M |
| mod.rs:325 `should_broadcast_console_to_ops` | `CommandSourceStack#broadcastToAdmins` targets players where `PlayerList#isOp` (server op list), not a permission-level threshold | broadcast to `player.is_op()` if the server tracks op status separately from `op_permission_level` | S (verify) |
| mod.rs:322 console feedback | dedicated server logs console command feedback through the log file as well as stdout | route `CommandSender::Console` output through `tracing`/log sink instead of bare `println!` | S |
| mod.rs:276 `world_or_first` | console/dummy sources use the overworld | already matches vanilla; kept as note only | none |

## h6 gap status

- Row 15 console output via tracing: STILL OPEN (console send_message is the only terminal path; changing it changes console format, needs a decision).


## Result

- Row 15 console output via tracing: STILL OPEN. Reason: the only console terminal path is the `println!` in `crates/pumpkin-core/src/command/mod.rs:147`, a file outside this unit's editable files (basenames command.rs / command_block.rs / command_source.rs), and routing it through tracing changes the console format (needs a decision). Verified against decompiled vanilla `net/minecraft/commands/CommandSourceStack.java:524-552` (console/admin feedback goes through `source.sendSystemMessage` / `server.sendSystemMessage`, a Pumpkin-side sink choice, not a parity defect in an in-scope file). No source file edited; no unit test added (no behaviour change); no rustfmt run needed.


## Result

- Row 11 (mod.rs:381 CustomName in `into_source`/`Display`): STILL OPEN. `custom_name` is stored, NBT-persisted and restored in `block/entities/command_block.rs`, but `CommandSender::fmt` and `into_source` at `command/mod.rs:136,389` still hard-code `"@"` (the `// TODO: when command blocks get custom names...` at line 388). `mod.rs` is outside this unit's editable basenames.
- Row 12 (block/entities/command_block.rs:11 LastExecution/UpdateLastExecution/CustomName): FIXED (already in the checkout). `last_execution: AtomicI64`, `update_last_execution: AtomicBool`, `custom_name`, the NBT read/write at lines 58-159 and the execute-time update at `block/blocks/command.rs:197-203` match vanilla `BaseCommandBlock.save/load/performCommand`; round-trip unit tests exist. No new edit needed.
- Row 13 (block/blocks/command.rs `chain_execute`): FIXED. Verified against decompiled vanilla `net/minecraft/world/level/block/CommandBlock.java:189-224`: unconditional links always run because `CommandBlockEntity.markConditionMet()` (line 146-159) returns true for non-conditional blocks; only conditional links gate on the previous block's `SuccessCount`, which `conditions_met` already implements. The row's suggested "gate unconditional links on previous SuccessCount" contradicts vanilla and was NOT applied. The real gap is `if (!baseCommandBlock.performCommand(level)) break` (vanilla `BaseCommandBlock.java:93-97` returns false when the block already ran this tick): `chain_execute` now breaks when `last_execution == world age` before executing. File edited: `crates/pumpkin-core/src/block/blocks/command.rs`; `rustfmt --edition 2024` run on it. No unit test added: `chain_execute` needs a live `World`/`Server`, so no pure seam exists.
- Row 14 (mod.rs:325 admin broadcast target): STILL OPEN. `send_to_ops` at `command/context/command_source.rs:383-406` uses `permission_lvl >= op_permission_level`; vanilla uses `PlayerList#isOp` (decompiled Paper uses `hasPermission("minecraft.admin.command_feedback")`). Pumpkin has no `player.is_op()` API and models ops as permission levels, so switching the target set to `server.data.operator_config.ops` membership would change permission-plugin semantics; needs a decision. No edit made.
- Row 15 (mod.rs:322 console feedback via tracing): VERIFIED FIXED (already in the checkout). `CommandSender::send_message` at `command/mod.rs:148` now uses `tracing::info!("{}", text.to_pretty_console())`; no `println!` remains in that path. `mod.rs` is outside this unit's editable basenames and needed no edit here.
- Row 16 (mod.rs:276 `world_or_first`): no change; console/dummy sources anchoring to the overworld already matches vanilla.


## Result

- Row 14 (mod.rs:325 admin broadcast target): FIXED (supersedes the earlier STILL OPEN note). Verified against decompiled vanilla `net/minecraft/commands/CommandSourceStack.java:539-548`: `broadcastToAdmins` sends to players where the source differs and the player is an op (`PlayerList#isOp`, exposed here by the Paper patch as `hasPermission("minecraft.admin.command_feedback")`); it does not use a permission-level threshold. `send_to_ops` in `command/context/command_source.rs:383` now checks `server.data.operator_config.read().get_entry(&player.gameprofile.id).is_some()` (the same op-list membership pattern used in `item/items/spawn_egg.rs:82` and `server/mod.rs:1404`) instead of `player.permission_lvl >= op_permission_level`. File edited: `crates/pumpkin-core/src/command/context/command_source.rs`. No unit test added: `send_to_ops` needs a live `Server`, so no pure seam exists. Trade-off: non-op players with an elevated permission level granted by plugins no longer receive admin feedback, which is the parity-correct behaviour but may affect permission-plugin setups.
- Final row status: 11 STILL OPEN (needs `command/mod.rs`, outside editable basenames); 12 FIXED (pre-existing); 13 FIXED (pre-existing edit); 14 FIXED (this pass); 15 VERIFIED FIXED (pre-existing); 16 no change. No remaining in-scope actionable rows.
