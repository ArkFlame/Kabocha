# Final rows from docs/tracker/deferred-pass2.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: tag.rs

- tentities-mangrove-boat-ad6fea.md: - **Dimensions / eye height** (was STILL OPEN): CLOSED. Vanilla `MANGROVE_BOAT` and `MANGROVE_CHEST_BOAT` are both `.sized(1.375F, 0.5625F)` with `.eyeHeight(0.5625F)` and `.clientTrackingRange(10)`. Generated `crates/pumpkin-data` `EntityType::MANGROVE_BOAT` already has `dimension: [1.375, 0.5625]`
- tentities-poplar-chest-boat.md: - Row "c:boats missing poplar_chest_boat" (doc line 16): STILL OPEN, needs codegen. The value lives in generated crates/pumpkin-data/src/generated/tag.rs, which must not be hand-edited; regenerate with cargo run --locked -p pumpkin-codegen. No functional gap remains because is_boat() already covers

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
