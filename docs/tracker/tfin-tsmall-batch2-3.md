# Final rows from docs/tracker/tsmall-batch2.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: item.rs

| taudit-crates-pumpkin-core-src-co-20e7ce : 21 (`commands/item.rs:527-532`) | STILL OPEN (partial FIXED) | `uniform` is handled at HEAD (`number_provider`, item.rs). Binomial, score, storage and sum forms are still unhandled. |
| taudit-crates-pumpkin-core-src-co-20e7ce : 22 (`commands/item.rs:507-516`) | STILL OPEN | Verified: vanilla `SetItemCountFunction:43-47` does `setCount(base + value)` with no clamp. Rust `set_count` clamps to `get_max_stack_size()`. Fixing needs an over-max representation; `item_count` is `u8`. |
| taudit-crates-pumpkin-core-src-co-20e7ce : 24 (`commands/item.rs:554-586`) | STILL OPEN | Verified: `set_block_slot` returns `Ok(1)` and the entity/modify executors return a count with no `commands.item.*.modify.success` message. Vanilla `ItemCommands:259` calls `accessor.getModifySuccess`. Fix needs translation-key wiring; not applied without a compile. |

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
