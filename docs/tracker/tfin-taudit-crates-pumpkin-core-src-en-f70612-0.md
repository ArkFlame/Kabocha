# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-f70612.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: creaking.rs melee_attack.rs

- 1 async search failure has no +15 retry backoff (melee_attack.rs:263): STILL OPEN, M.
- 2 stop() dead ForgotTarget block (melee_attack.rs:204-214): STILL OPEN, S.
- 4 constructor arg naming and inverted meaning (melee_attack.rs:19,38; creaking.rs:119): STILL OPEN, S, creaking.rs is out of scope this pass.
- 5 attack_interval_ticks unused (melee_attack.rs:26,86): STILL OPEN, S.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
