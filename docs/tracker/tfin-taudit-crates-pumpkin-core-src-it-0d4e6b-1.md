# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-it-0d4e6b.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: bucket.rs dispenser.rs

- 5 hardcoded Hand::Right (bucket.rs:783-798): STILL OPEN, M, trait change.
- 6 event payload lacks face/stack (bucket.rs:642-654, 726-736): STILL OPEN, M, additive WIT change.
- 7 dispenser break_block without SKIP_DROPS (redstone/dispenser.rs:910-923): STILL OPEN, S.
Verified all seven STILL OPEN rows against .vendor/vanilla-decompiled (BucketItem,

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
