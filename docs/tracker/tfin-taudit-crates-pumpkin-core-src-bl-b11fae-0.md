# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-bl-b11fae.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: mod.rs

- 1 STILL OPEN: needs a shape-list API; Size M, not attempted in sweep.
- 2 STILL OPEN: no live difference. `block/mod.rs:827` divides by item max; vanilla divides by min(container max, item max). Default container max is 99, which is never below an item max (<=64), so the result matches. Only a custom container max under 64 would differ.
- 3 STILL OPEN: vanilla `pushEntitiesUp` not confirmed (no decompiled source in repo).
- 4 STILL OPEN: `BlockDropItemEvent` player threading is M, not attempted.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
