# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-it-073ff6.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- Gap 1: STILL OPEN: needs the 26.3 `spawnOffspringFromSpawnEgg` age guard; no decompiled vanilla in .vendor to confirm.
- Gap 2: STILL OPEN: needs vanilla `Entity.load` handling of `Pos`/`UUID` to decide if stripping is deliberate.
- Gap 3: STILL OPEN: needs vanilla `TrialSpawner.setEntityId` supported-type check and config regeneration source.
- Gap 4: STILL OPEN: needs vanilla SpawnEggItem spawner branch (shrink vs consume, sendBlockUpdated flags) to confirm.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
