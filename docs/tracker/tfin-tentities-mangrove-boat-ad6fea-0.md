# Final rows from docs/tracker/tentities-mangrove-boat-ad6fea.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: boat.rs

- **Dimensions / eye height** (was STILL OPEN): CLOSED. Vanilla `MANGROVE_BOAT` and `MANGROVE_CHEST_BOAT` are both `.sized(1.375F, 0.5625F)` with `.eyeHeight(0.5625F)` and `.clientTrackingRange(10)`. Generated `crates/pumpkin-data` `EntityType::MANGROVE_BOAT` already has `dimension: [1.375, 0.5625]`, `eye_height: 0.5625`, `client_tracking_range: 10`. No gap; generated data is correct and must not be hand-edited. Added a pure parity unit test `mangrove_boat_dimensions_match_vanilla` in `item/items/boat.rs` so drift is caught.
- **Mangrove-specific sounds / particles** (was STILL OPEN): CLOSED. Vanilla `AbstractBoat.getPaddleSound()` returns only the shared `SoundEvents.BOAT_PADDLE_WATER` / `BOAT_PADDLE_LAND` for all wood types; splash particles are the shared `ParticleTypes.SPLASH`. `Boat`, `AbstractChestBoat`, `ChestBoat` define no wood-specific sound/particle overrides. No mangrove-specific behaviour exists to port.
Re-verified both rows against decompiled vanilla; both remain CLOSED / VERIFIED, no STILL OPEN rows.
Both original unverified rows end VERIFIED; no STILL OPEN rows remain. No source fix was required.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
