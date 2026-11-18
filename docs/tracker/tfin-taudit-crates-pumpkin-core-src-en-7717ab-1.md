# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-7717ab.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

Final re-audit of the three open rows against .vendor/vanilla-decompiled

## Result
Row 3 FIXED (neutral.rs seeded anger duration via World::loot_random). Row 4 FIXED (bee.rs:242 and wolf.rs:99 define DATA_ANGER_END_TIME). Row 5 BLOCKED-DATA: iron golem / polar bear / zombified piglin accessor needs regenerated crates/pumpkin-data/src/generated/tracked_data.rs (not re-confirmed this pass).
