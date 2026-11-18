# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-cd7fac.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: sensing.rs types.rs

| `types.rs:121-122` (`VISIBLE_ADULT_PIGLIN_COUNT`, `VISIBLE_ADULT_HOGLIN_COUNT`) | `PiglinSpecificSensor.java:107-108` writes both counts from the visible adult lists; `PiglinAi.java:780-781` and `HoglinAi.java:179-180` read them to make the outnumbered side retreat | Nothing writes or reads the counts (the `// TODO` at `types.rs:120`) | Port `PiglinSpecificSensor` and the two `retreatFromNearestTarget` call sites | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `types.rs:118-122` (`NEARBY_ADULT_PIGLINS`, `NEAREST_VISIBLE_ADULT_PIGLINS`, `NEAREST_VISIBLE_ADULT_HOGLINS`, `NEAREST_VISIBLE_ADULT_PIGLIN`) | Written by `PiglinSpecificSensor`/`HoglinSpecificSensor` | Declared but never populated; `crates/pumpkin-core/src/entity/ai/sensing.rs` has no piglin/hoglin specific sensor | Port the two species sensors together with the counts above | L, STILL OPEN: size L, skipped per wave brief |
- VISIBLE_ADULT_PIGLIN/HOGLIN_COUNT never written or read (types.rs:120-122): STILL OPEN, M, needs PiglinSpecificSensor and HoglinAi/PiglinAi call sites.
- NEARBY_ADULT_PIGLINS / NEAREST_VISIBLE_ADULT_* never populated (types.rs:118-122): STILL OPEN, L, no piglin/hoglin specific sensor in sensing.rs.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
