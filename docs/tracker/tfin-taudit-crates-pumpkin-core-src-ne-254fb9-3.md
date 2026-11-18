# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-ne-254fb9.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: keep_alive.rs mod.rs

- Row 3 stonecutter-only recipe packet (mod.rs:772-781): STILL OPEN, M - needs generated recipe property sets in pumpkin-data/codegen, outside this unit.
- Row 1 chat/command/session spawned off-thread (mod.rs:908-977, 1201): STILL OPEN, M - needs a per-player ordered dispatch queue in `mod.rs`, outside this unit's allowed `keep_alive.rs`.
- Row 2 config disconnect loses Component (mod.rs:651-670): STILL OPEN, M - needs a `CConfigDisconnect` protocol/component change in `pumpkin-protocol`, outside this unit.
- Row 3 stonecutter-only recipe packet (mod.rs:772-781): STILL OPEN, M - needs generated recipe property sets in `pumpkin-data`/codegen, outside this unit and generated files must not be hand-edited.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
