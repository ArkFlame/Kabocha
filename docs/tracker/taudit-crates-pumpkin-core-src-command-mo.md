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
