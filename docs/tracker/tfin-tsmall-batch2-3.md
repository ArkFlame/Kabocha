# Final rows from docs/tracker/tsmall-batch2.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: item.rs

| taudit-crates-pumpkin-core-src-co-20e7ce : 21 (`commands/item.rs:527-532`) | VERIFIED | `number_provider_with` (`item.rs:1273-1312`) handles constant, uniform, binomial and sum. `ProviderContext::resolve_score` (:1188) and `resolve_storage` (:1209) cover score and storage. Vanilla `world/level/storage/loot/providers/number/ints/ContextIntProviderTypes.java`, `ScoreboardValue.java:18-54`, `StorageValue.java:11-24`. |
| taudit-crates-pumpkin-core-src-co-20e7ce : 22 (`commands/item.rs:507-516`) | FIXED (already at HEAD) | `set_count` (`item.rs:863-864`) does not clamp to the max stack size, matching vanilla `SetItemCountFunction.java:43-47`. `LimitCount` clamps to `get_max_stack_size()` (`item.rs:935-936`), matching `ItemCommands` limitSize. |
| taudit-crates-pumpkin-core-src-co-20e7ce : 24 (`commands/item.rs:554-586`) | VERIFIED | `BlockModifyExecutor` (`item.rs:1361-1372`) sends `COMMANDS_ITEM_BLOCK_MODIFY_SUCCESS`. `EntityModifyExecutor` (`item.rs:1402-1418`) sends the `.SINGLE` or `.MULTIPLE` message. Vanilla `server/commands/item/ItemCommands.java:259` calls `getModifySuccess`. |

## Result
All three rows closed: 21 VERIFIED, 22 FIXED at HEAD, 24 VERIFIED. No row remains open. No `.rs` file was edited this pass.
