# Final rows from docs/tracker/tentities-creeper-0de8b3.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

Second-pass re-review against `.vendor/vanilla-decompiled/net/minecraft/`. The three STILL OPEN tracker rows remain verified fixed in the checkout, so no change was needed for them: row 2 (`dropped_skulls` one-shot guard), row 3 (`on_mob_removed` fired in `CreeperEntity::explode`), row 4 (`SwellGoal.canUse` aliveness guard matches vanilla).
Third-pass verification against `.vendor/vanilla-decompiled/net/minecraft/`. All three STILL OPEN rows are already resolved in the checkout; no source file needed a change and no `rustfmt` run was required.
THIRD PASS verification against `.vendor/vanilla-decompiled/net/minecraft/`. All three original STILL OPEN rows are already resolved in the checkout; no source file needed a change, so no rustfmt run was required. The stale `Remaining gaps` table above has been updated to final statuses.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
