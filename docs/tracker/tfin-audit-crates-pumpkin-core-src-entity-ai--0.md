# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-entity-ai-.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- Rows: STILL OPEN. Note body partly unreadable in this pass; nothing verified.
| 3 | STILL OPEN | `set_progress` returns `()`, so the `if (!moveTo(...)) ticks += 15` penalty has no synchronous result to read. Needs a path-request-result field on `Navigator` (pathfinder files, outside this unit). |

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
