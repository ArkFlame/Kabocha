# Final rows from docs/tracker/tblocks-creaking-heart-0a1e6a.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- Row 2 (isProtector UUID match): STILL OPEN (S, blocked on the missing creaking spawn/bind path in tick, L).
- Row 1, 3, 4, 6: STILL OPEN (L/M, need spawn loop and creaking AI teardown; piston row needs push-reaction data).
- Row 5 hurt particle: STILL OPEN (S, left as-is by design).
| 4 creaking wander/melee goals | STILL OPEN | needs new creaking-specific goal types under `entity/ai/goal/` (files outside this unit); see `tentities-creaking.md`. |

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
