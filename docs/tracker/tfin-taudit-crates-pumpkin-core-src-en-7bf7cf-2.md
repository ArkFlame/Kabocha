# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-7bf7cf.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: goals.rs sneeze.rs

- Row 4 `after_sneeze` AI-island note (sneeze.rs:87-121) - STILL OPEN, no code change. One-line reason: finishing the species-level panda tick/AI/Brain reach audit and flipping `has_audited_tick` needs `panda/goals.rs` (outside this unit) and would still fall back to the conservative `unaudited_reach` because the shared melee/revenge goals stay unaudited and the bamboo box read exceeds `GoalReach`'s block-read bound (goals.rs:104); there is no smallest in-unit fix.
Second-pass verification of the single STILL OPEN row against decompiled vanilla at
- Row 4 `after_sneeze` AI-island note (sneeze.rs:87-121) - STILL OPEN, no code change. One-line reason:
Third pass: every row now ends FIXED, VERIFIED or BLOCKED, none left STILL OPEN. Vanilla read at

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
