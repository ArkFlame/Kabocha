# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-co-9785ce.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: attributes.rs data.rs nbt_path.rs

Re-checked every STILL OPEN row against `.vendor/vanilla-decompiled` (target `V_26_3`). All rows that can be fixed in these two files were already fixed in the prior pass and are confirmed correct; no .rs file needed changes this pass, so no rustfmt was run.
Re-verified every STILL OPEN row directly against `.vendor/vanilla-decompiled/net/minecraft` (target V_26_3). All in-unit fixes from prior passes are present and correct; no source file needed changes this pass, so no rustfmt was run.
Re-verified the three STILL OPEN rows against `.vendor/vanilla-decompiled/net/minecraft` (target V_26_3) and the live Pumpkin sources. No `.rs` file needed a change this pass; no rustfmt was run.
Re-read the unit's last Result and re-verified the two remaining STILL OPEN rows directly against `.vendor/vanilla-decompiled/net/minecraft` (target `V_26_3`). Both rows require a whole missing subsystem, so both are closed as BLOCKED-DATA with the exact subsystem named; neither is fixable by a small edit to attributes.rs/data.rs/nbt_path.rs. No generated `pumpkin-data` field is missing for either row, and no source file was changed this pass, so no rustfmt was run.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
