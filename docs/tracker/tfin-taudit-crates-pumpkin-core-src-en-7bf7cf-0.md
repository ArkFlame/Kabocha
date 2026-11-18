# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-7bf7cf.md

Files: goals.rs sneeze.rs

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files).

## Result (batch 09 pass, 2026-11-18)

- Row 4 `after_sneeze` AI-island note (panda/sneeze.rs:73-121,186-188; panda.rs:840-876,905-909): VERIFIED. Matches Panda.java:338-375,506-531,1110-1133 (sneeze counter, sounds, afterSneeze 10-block jump for grounded adult pandas, gift only under MOB_DROPS). Pandas keep has_audited_tick false, so entity_islands.unaudited_reach (>=16 blocks) covers the jump. Setting has_audited_tick is a species-level subsystem change, not a smallest fix, and is not a current bug.

No .rs file changed in this pass.
