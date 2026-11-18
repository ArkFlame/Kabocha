# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-7bf7cf.md

Files: goals.rs sneeze.rs

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files).

## Result (batch 09 pass, 2026-11-18)

- Row 4 `after_sneeze` AI-island note (sneeze.rs:87-121): VERIFIED against Panda.java:506-531. Pandas keep has_audited_tick false, so the conservative unaudited_reach covers the 10-block jump and PandaSneezeGoal::tick_reach reports it. No current bug; no code change.

No .rs file changed in this pass.
