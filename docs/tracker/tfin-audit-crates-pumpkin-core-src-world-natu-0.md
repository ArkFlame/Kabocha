# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-world-natu.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: mod.rs

- `canSpawnEntitiesInChunk` check (world/mod.rs:3295 still `// TODO this.level.canSpawnEntitiesInChunk`): STILL OPEN (M, not changed in this pass).
Re-verified every STILL OPEN / LEFT OPEN row against
- Row 7 (chunk-gen vs live adjust_spawn_position, narrow case): STILL OPEN, out
Row 7 (chunk-gen vs live adjust_spawn_position) was the only STILL OPEN row. FIXED

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
