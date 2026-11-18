# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-ne-254fb9.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: keep_alive.rs mod.rs

- 1 SChatMessage/SChatCommand/SPlayerSession spawned off-thread (net/java/mod.rs:908-977, 1201): STILL OPEN, M, needs per-player ordered queue.
- 2 config disconnect loses Component (mod.rs:651-670): STILL OPEN, M, protocol change.
- 3 stonecutter-only recipe packet (mod.rs:772-781): STILL OPEN, M, data/codegen.
- Row 1 (chat/command/session spawned off-thread, mod.rs): STILL OPEN, M, needs a per-player ordered dispatch queue in `mod.rs`, outside this unit's `keep_alive.rs`.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
