# entities / Mangrove Boat re-review (RE-REVIEW-UNTRACKED)

Scope: Rust source for the Mangrove Boat (and its chest variant, shared paths).

Checked paths:
- `crates/pumpkin-core/src/item/items/boat.rs`: `item_to_entity` maps `Item::MANGROVE_BOAT` and `MANGROVE_CHEST_BOAT` to their entity types; both ids are in `ids()`; `entity_to_item` covers the reverse drop mapping.
- `crates/pumpkin-core/src/entity/type.rs:342-343`: both mangrove entity types are built as `BoatEntity`.
- `crates/pumpkin-core/src/entity/vehicle/boat.rs:53`: `MANGROVE_CHEST_BOAT` is in `is_chest_boat`, so it gets the 27-slot inventory and 1 passenger limit; the plain boat gets 2 passengers.
- `crates/pumpkin-core/src/entity/player.rs:4412`: the mangrove boat counts toward the boat movement statistic.

Gaps found: none that are small and certain. The source matches the shared boat behaviour in `docs/VANILLA_REF.md:32`.

Unverified (no `.vendor/` decompiled class was read, and the generated entity dimensions in `crates/pumpkin-data` were not compared to vanilla):
- Mangrove boat entity dimensions and eye height vs vanilla `EntityType` values. Size: S. Fix: compare against decompiled `EntityType.MANGROVE_BOAT` once a reference is available.
- Mangrove-specific sounds and particles, if any differ from other wood types. Size: S.

## Gap status
- No small certain gap recorded; both unverified items CLOSED (verified against .vendor decompiled vanilla; see Result below)


## Result

Verification against decompiled vanilla (`net/minecraft/world/entity/EntityTypes.java`) completed 2026-11-17.

- **Dimensions / eye height** (was STILL OPEN): CLOSED. Vanilla `MANGROVE_BOAT` and `MANGROVE_CHEST_BOAT` are both `.sized(1.375F, 0.5625F)` with `.eyeHeight(0.5625F)` and `.clientTrackingRange(10)`. Generated `crates/pumpkin-data` `EntityType::MANGROVE_BOAT` already has `dimension: [1.375, 0.5625]`, `eye_height: 0.5625`, `client_tracking_range: 10`. No gap; generated data is correct and must not be hand-edited. Added a pure parity unit test `mangrove_boat_dimensions_match_vanilla` in `item/items/boat.rs` so drift is caught.
- **Mangrove-specific sounds / particles** (was STILL OPEN): CLOSED. Vanilla `AbstractBoat.getPaddleSound()` returns only the shared `SoundEvents.BOAT_PADDLE_WATER` / `BOAT_PADDLE_LAND` for all wood types; splash particles are the shared `ParticleTypes.SPLASH`. `Boat`, `AbstractChestBoat`, `ChestBoat` define no wood-specific sound/particle overrides. No mangrove-specific behaviour exists to port.

No source fix was required; both rows were verification-only. One test added.

Status: PASS fixed=0 open=0 files=crates/pumpkin-core/src/item/items/boat.rs
