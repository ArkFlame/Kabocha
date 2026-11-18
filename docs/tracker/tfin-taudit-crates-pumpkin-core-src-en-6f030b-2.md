# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-6f030b.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: spawn_util.rs

- All other audit rows are marked FIXED and were not re-opened. No remaining STILL OPEN row: nothing needs generated data, WIT changes, a cross-crate refactor or BLOCKED-DATA. No pure unit-test seam: each gate is one comparison on a freshly created Mob needing a live World, and spawn_util.rs already unit-tests the pure SpawnStrategy::can_spawn_on.
Final pass (unit scope widened to any source file). All six gaps are FIXED and VERIFIED directly against decompiled vanilla; no STILL OPEN row remains and no source change was needed, so no .rs file was edited and no rustfmt was needed.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
