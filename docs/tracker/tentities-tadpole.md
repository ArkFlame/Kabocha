# entities / Tadpole

Coverage: RE-REVIEW (reviewed source changed)
Reviewed: 2026-11-16 against vanilla `net.minecraft.world.entity.animal.Tadpole` (Java 26.3).

## Fixed in this pass
- `crates/pumpkin-core/src/entity/passive/tadpole.rs:260` — `Bucketable::is_from_bucket` was hardcoded `true` and `set_from_bucket` was a no-op. Vanilla `AbstractFish#fromBucket` is a real field: `FromBucket` NBT was always written as true, bucket release never set it, and `removeWhenFarAway` therefore never despawned frogspawn tadpoles. Now stored in `FishData::from_bucket` and synced through `abstract_fish::FROM_BUCKET`, like every other fish.
- `crates/pumpkin-core/src/entity/passive/tadpole.rs:254` — missing `CustomSound::sound_volume`; vanilla `Tadpole#getSoundVolume()` is 0.15 (already used for the grow-up sound). Flop/hurt/death sounds played at volume 1.0. Added the override.

## Remaining gaps
1. `crates/pumpkin-core/src/entity/passive/tadpole.rs:49` — no tadpole variant. Vanilla `Tadpole` has `DATA_VARIANT` (`TadpoleVariant`: temperate/warm/cold), synced data, `finalizeSpawn` picks the biome spawn variant and NBT/bucket store `variant`. `pumpkin-data` generates no tadpole variant at all (`crates/pumpkin-data/src/generated/tracked_data.rs:17973`). Vanilla class: `Tadpole` / `TadpoleVariant`. Fix: extract variants in `assets/` via the Extractor, add `DATA_VARIANT` to codegen, add a `TadpoleVariant` data module, field + setter/getter, `finalize_spawn` biome selection. Size: L (blocked on codegen/assets).
2. `crates/pumpkin-core/src/entity/passive/tadpole.rs:131` — `age_up` converts to `EntityType::FROG` with the default frog variant. Vanilla maps the tadpole variant to the matching frog variant (`TadpoleVariant#frogVariant`) before spawning. Vanilla class: `Tadpole#ageUp`. Fix: after gap 1, set `Frog#setVariant(variant.frogVariant())` in the conversion callback. Size: M (depends on gap 1).
3. `crates/pumpkin-core/src/entity/passive/water_animal.rs:23` — `handle_air_supply` is called with `MAX_AIR` (300), so tadpoles (and all fish) drown after 15 s on land. Vanilla `WaterAnimal#getMaxAirSupply()` returns 4800 and `increaseAirSupply` refills to it. Vanilla class: `WaterAnimal`. Fix: add `WATER_ANIMAL_MAX_AIR = 4800`, use it in `handle_air_supply`, and let `breath.rs` clamp/sync against the entity's max air instead of the fixed 300. Size: M (shared by all water animals; touches `breath.rs`).
