# Final rows from docs/tracker/deferred-pass2.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: tag.rs

- tentities-mangrove-boat-ad6fea.md: - **Dimensions / eye height**: VERIFIED. Vanilla `MANGROVE_BOAT` and `MANGROVE_CHEST_BOAT` are both `.sized(1.375F, 0.5625F)` with `.eyeHeight(0.5625F)` and `.clientTrackingRange(10)`. Generated `crates/pumpkin-data` `EntityType::MANGROVE_BOAT` already has `dimension: [1.375, 0.5625]`
- tentities-poplar-chest-boat.md: - Row "c:boats missing poplar_chest_boat" (doc line 16): BLOCKED-DATA, needs codegen. The value lives in generated crates/pumpkin-data/src/generated/tag.rs, which must not be hand-edited; regenerate with cargo run --locked -p pumpkin-codegen. No functional gap remains because is_boat() already covers

## Result

Final pass (2026-11-18). All rows now have a terminal status; no open rows remain. Only this doc was edited (no .rs source change), so rustfmt is not applicable and no build is needed.

- tentities-mangrove-boat-ad6fea.md / "Dimensions / eye height" (previously open): VERIFIED (matches vanilla). Decompiled `.vendor/vanilla-decompiled/net/minecraft/world/entity/EntityTypes.java:674-689` declares `MANGROVE_BOAT` and `MANGROVE_CHEST_BOAT` as `.sized(1.375F, 0.5625F).eyeHeight(0.5625F).clientTrackingRange(10)`. Generated `crates/pumpkin-data/src/generated/entity_type.rs` matches: `MANGROVE_BOAT` dimension `[1.375, 0.5625]`, eye_height `0.5625`, client_tracking_range `10`; `MANGROVE_CHEST_BOAT` identical. Drift-guard test `mangrove_boat_dimensions_match_vanilla` is present in `crates/pumpkin-core/src/item/items/boat.rs`. No generated-data change needed.

- tentities-poplar-chest-boat.md / "c:boats missing poplar_chest_boat" (doc line 16, previously open): BLOCKED-DATA. Exact missing field: `EntityType::C_BOATS`, the `c:boats` tag constant at `crates/pumpkin-data/src/generated/tag.rs:20874`, whose value list (lines 20875-20897) ends at "bamboo_chest_raft" (line 20896) and omits "poplar_chest_boat". The source asset `assets/datapack/data/c/tags/entity_type/boats.json:17` already lists `minecraft:poplar_chest_boat`, and the generated vanilla `MINECRAFT_BOATS` / `MINECRAFT_CHEST_BOATS` tags already contain `poplar_chest_boat` (tag.rs:25051, 25309), so regeneration will add it. Fix requires `cargo run --locked -p pumpkin-codegen`; generated files must not be hand-edited and workers cannot run cargo. No functional gap: `crates/pumpkin-core/src/entity/vehicle/boat.rs` `is_boat` recognises poplar_chest_boat through `is_chest_boat`'s `_chest_boat` suffix.

Status: PASS fixed=0 open=0 files=docs/tracker/tfin-deferred-pass2-7.md
Pending orchestrator action: run `cargo run --locked -p pumpkin-codegen` and commit the regenerated `tag.rs` to resolve the single BLOCKED-DATA row.
