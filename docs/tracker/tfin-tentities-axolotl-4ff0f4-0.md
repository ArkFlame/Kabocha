# Final rows from docs/tracker/tentities-axolotl-4ff0f4.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: ai.rs axolotl.rs turtle.rs

All four STILL OPEN rows were already resolved by the prior pass; each was re-checked directly against the decompiled source and no new gap or source change was found.
Third independent pass. Re-verified all four STILL OPEN rows directly against the decompiled source now present at `.vendor/vanilla-decompiled/`; every row was already correctly resolved, so no source change was required.
Out-of-scope finding (not changed, not a STILL OPEN row): `axolotl.rs:234,238` comment and code install `Navigator::water_bound(false)` claiming vanilla uses `WaterBoundPathNavigation`, but authoritative `Axolotl.java:386-388` returns `AmphibiousPathNavigation` (axolotls move on land, cf. `SPEED_MULTIPLIER_ON_LAND` in `ai.rs`, and `Navigator::amphibious` already used by `turtle.rs:114`). Left for the orchestrator to scope: correcting it also requires rewriting the `navigation_is_water_bound_like_vanilla` regression test (`axolotl.rs:1554`).
Fourth independent pass. All four STILL OPEN rows re-verified directly against decompiled vanilla and confirmed already resolved (no source change needed for them). Fixed one genuine remaining gap in `axolotl.rs` that the prior pass flagged as out of scope: the navigator type.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
