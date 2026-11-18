# Final rows from docs/tracker/deferred-pass2.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: ageable.rs armadillo.rs

- tblocks-creaking-heart-0a1e6a.md: | 6 piston push-reaction | STILL OPEN | needs a push-reaction field on generated `pumpkin-data` `Block` plus piston push code (generated data / cross-crate), nothing per-block to fix here. |
- tentities-armadillo-d4b8f2.md: All six STILL OPEN rows verified against decompiled vanilla (`.vendor/vanilla-decompiled/net/minecraft/`). No generated data, WIT or cross-crate changes were needed. Only `armadillo.rs` was edited; `ageable.rs` was read only.
- tentities-bamboo-chest-raft-451500.md: - Row 8 (fluid push, L): STILL OPEN. Engine-wide client-authority difference; skip unless server
- tentities-command-block-minecart-5e81b3.md: - Row 4 (hardcoded `JavaMinecraftVersion::V_26_3`): STILL OPEN (BLOCKED). Needs a cross-crate

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
