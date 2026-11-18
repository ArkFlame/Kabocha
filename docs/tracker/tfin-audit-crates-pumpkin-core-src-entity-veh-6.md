# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-entity-veh.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: block_effects.rs minecart.rs tnt.rs

- block_effects 2 (endpoint double dispatch): STILL OPEN, M. Same shared-pass change, out of scope.
- block_effects 6 (gametest): STILL OPEN, M. Needs a pumpkin-gametest harness outside this unit's files.
`.../minecart/tnt.rs` only. Every other STILL OPEN row touches `block_effects.rs`, `minecart.rs`
- block_effects 1,2,3,4,5,6: STILL OPEN, out of scope (block_effects.rs / gametest harness).

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
