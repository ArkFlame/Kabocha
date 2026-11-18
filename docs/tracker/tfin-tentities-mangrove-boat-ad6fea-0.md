# Final rows from docs/tracker/tentities-mangrove-boat-ad6fea.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: boat.rs

- Dimensions / eye height (previously open): CLOSED.
- Mangrove-specific sounds / particles (previously open): CLOSED.

## Result
- **Dimensions / eye height**: VERIFIED. `.vendor/vanilla-decompiled/net/minecraft/world/entity/EntityTypes.java` lines 674-689 define `MANGROVE_BOAT` and `MANGROVE_CHEST_BOAT` as `.sized(1.375F, 0.5625F)`, `.eyeHeight(0.5625F)`, `.clientTrackingRange(10)`. Generated `crates/pumpkin-data/src/generated/entity_type.rs` `MANGROVE_BOAT` already carries `dimension: [1.375, 0.5625]`, `eye_height: 0.5625`, `client_tracking_range: 10`. Pure unit test `mangrove_boat_dimensions_match_vanilla` in `item/items/boat.rs` pins the values. No source fix required.
- **Mangrove-specific sounds / particles**: VERIFIED. `AbstractBoat.getPaddleSound()` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java` :410-416) returns only the shared `BOAT_PADDLE_WATER` / `BOAT_PADDLE_LAND`; splash is the shared `ParticleTypes.SPLASH` (:181). `Boat`, `AbstractChestBoat`, `ChestBoat` define no wood-specific overrides. No source fix required.

Status: fixed=0 open=0. No compile run.
