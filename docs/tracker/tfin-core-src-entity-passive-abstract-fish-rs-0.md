# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-entity-passive-abstract-fish-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

no rustfmt was needed. All rows closed as VERIFIED; none left STILL OPEN.

## Result
FINAL PASS 2026-11-18. Independently re-verified the single row against the authoritative decompiled
vanilla under `.vendor/vanilla-decompiled/net/minecraft/`. The row premise is false for this version, so
current Pumpkin code already matches vanilla and no source edit was warranted. No file was touched, so no
rustfmt was needed. All rows closed; none left STILL OPEN.

- `own/tentities-tadpole-3cdaf7` - VERIFIED (matches vanilla). Sub-tracker gaps 1-4 are all false premises:
  - Gap 1 (add `TadpoleVariant`): VERIFIED no-gap. `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/frog/Tadpole.java:43-45` defines only `AGE_LOCKED`; no `TadpoleVariant` class and no `variant` synced data, NBT or bucket key exist anywhere in the decompiled tree (only `FrogVariant`/`FrogVariants` in the frog dir). No missing generated field exists, so BLOCKED-DATA does not apply.
  - Gap 2 (carry variant into frog): VERIFIED no-gap. `Tadpole.ageUp()` only calls `frog.finalizeSpawn`, `setPersistenceRequired`, `fudgePositionAfterSizeChange` and the grow-up sound. Frog variant is biome-picked in `Frog#finalizeSpawn`, which Pumpkin already does; depends on the non-existent gap 1.
  - Gap 3 (RunIf + RandomLookAround gate): VERIFIED no-gap. `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/frog/TadpoleAi.java:64` is `Pair.of(BehaviorBuilder.triggerIf(Entity::isInWater), 5)`, a deliberate no-op; no `RunIf`/`RandomLookAround` exists in this brain. Pumpkin's `trigger_if(...is_in_water)` matches.
  - Gap 4 (water-animal max air supply): VERIFIED no-gap. `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/fish/WaterAnimal.java:47` refills to a hardcoded `setAirSupply(300)`; only Dolphin overrides to 4800. Pumpkin's `water_animal.rs:10` `WATER_ANIMAL_MAX_AIR_SUPPLY = 300` and `abstract_fish.rs:303-306` `mob_max_air_supply` = 300 match. The stale note claiming a 4800 water-animal value was not applied and must not be re-applied.

Status: PASS, fixed=0, open=0, files=none (code already matches vanilla; tracker row closed as VERIFIED).
