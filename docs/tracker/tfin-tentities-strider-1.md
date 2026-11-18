# Final rows from docs/tracker/tentities-strider.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: mod.rs strider.rs

- Row `strider.rs:178` passenger attachment: STILL OPEN. UNVERIFIED offset, no vanilla source in tree.
- STILL OPEN `strider.rs:35` navigation: needs a per-mob `create_navigation` hook (Mob trait) plus `AmphibiousNodeEvaluator` wiring in `entity/ai/pathfinder/mod.rs`, both outside this unit's files.
- STILL OPEN `strider.rs:96` jockey spawn: needs a `finalizeSpawn` hook, `SpawnGroupData` baby-chance override and spawning separate jockey entities (zombified piglin / baby strider), cross-cutting spawn infrastructure outside this unit.
- STILL OPEN `strider.rs:178` passenger attachment: vanilla applies the rider offset only when `level().isClientSide()`, i.e. client-only visual effect; no server behaviour to port here.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
