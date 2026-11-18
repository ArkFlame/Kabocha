# Final rows from docs/tracker/tentities-panda.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: sneeze.rs

| 7 | `panda/sneeze.rs:84-109` | none (AI island note): `after_sneeze` reads and jumps pandas up to 10 blocks away from the mob tick, and `PandaSitGoal`/`PandaBreedGoal` still return `None` from `tick_reach` | audit that reach before ever setting `has_audited_tick` on pandas (the file already documents why it is false) | S | STILL OPEN (h6 budget ended, not attempted) |

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
