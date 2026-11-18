# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-co-20e7ce.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: item.rs mod.rs

| 5 | `commands/item.rs:527-532` | Number providers support constant, uniform, binomial, score, storage and sum forms | Only a plain number or `{value:...}` is read. `{type:"uniform",min,max}` yields `None` -> defaults to `1.0`, silently producing the wrong count/damage. | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 6 | `commands/item.rs:507-516` | `SetCountFunction` calls `stack.setCount(i)` for a non-add provider, which does not clamp to the item's max stack size (only `grow` clamps) | Both branches clamp to `get_max_stack_size()`, so `set_count` with a large value differs from vanilla. | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 7 | `commands/item.rs:95`, `455-480` | `ItemCommand.setBlockItem`/`modifyBlockItem` call `container.setChanged()` so the block entity is saved | No dirty flag is set; `BlockEntity` still lacks `mark_dirty` (`block/entities/mod.rs:76` TODO), so `item replace`/`item modify` changes to container block entities can be lost on save. | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 8 | `commands/item.rs:554-586` | 26.3 sends `commands.item.block.modify.success` and `commands.item.entity.modify.success.single/multiple` | Neither `item modify` executor sends feedback. | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
