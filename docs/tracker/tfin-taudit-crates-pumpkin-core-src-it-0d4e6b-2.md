# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-it-0d4e6b.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: bucket.rs

- Row 5 (hardcoded Hand::Right, bucket.rs:783-798): STILL OPEN. Needs ItemBehaviour::use_on_block
- Row 6 (event payload lacks face/stack, bucket.rs:642-654/726-736): STILL OPEN. Additive WIT
Re-checked every STILL OPEN row against .vendor/vanilla-decompiled BucketItem.java
- Row 6 (event payload lacks face/stack): STILL OPEN, out of scope. PlayerBucketEmptyEvent /

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
