# Parity rows for crates/pumpkin-core/src/entity/passive/abstract_fish.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/tentities-tadpole-3cdaf7** [partial/OWN-OPEN] own audit: 6 open, 0 fixed, 1 verified, 7 blocked-data (docs/tracker/tentities-tadpole-3cdaf7.md) docs/tracker/tentities-tadpole-3cdaf7.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)

## Result (final third pass 2026-11-18)

Independently re-verified the referenced row against the authoritative decompiled vanilla under
`.vendor/vanilla-decompiled/net/minecraft/`. Every row premise is false for this version, so current
Pumpkin code already matches vanilla and no source edit was warranted. No source file was touched, so
no rustfmt was needed. All rows closed as VERIFIED; none left STILL OPEN.

- `own/tentities-tadpole-3cdaf7` - VERIFIED (matches vanilla). Sub-tracker gaps 1-4 all closed:
  - Gap 1 (TadpoleVariant, was BLOCKED-DATA): VERIFIED no-gap. `world/entity/animal/frog/Tadpole.java`
    defines only `AGE_LOCKED` (line 45); no `TadpoleVariant` class and no `variant` synced data, NBT or
    bucket key exist anywhere in the decompiled tree (only `FrogVariant`/`FrogVariants` in the frog dir).
    There is no missing generated field to add, so BLOCKED-DATA does not apply.
  - Gap 2 (carry variant into frog): VERIFIED no-gap. `Tadpole.ageUp()` (236-250) only calls
    `frog.finalizeSpawn`, `setPersistenceRequired`, `fudgePositionAfterSizeChange` and the grow-up sound.
    Frog variant is biome-picked in `Frog#finalizeSpawn`, which Pumpkin already does. Depends on the
    non-existent gap 1.
  - Gap 3 (RunIf + RandomLookAround): VERIFIED no-gap. `TadpoleAi.java:64` is
    `Pair.of(BehaviorBuilder.triggerIf(Entity::isInWater), 5)`, a deliberate no-op; no `RunIf`/
    `RandomLookAround` in this brain. Pumpkin's `trigger_if(...is_in_water)` matches.
  - Gap 4 (squid air supply): VERIFIED no-gap. `WaterAnimal.handleAirSupply` (line 47) and
    `AgeableWaterCreature.handleAirSupply` (line 45) refill to a hardcoded `300`; only Dolphin overrides
    to 4800. Pumpkin's `water_animal.rs:10` `WATER_ANIMAL_MAX_AIR_SUPPLY = 300` and
    `abstract_fish.rs:303-306` `mob_max_air_supply` = 300 match. The stale "Fixed in this pass" note
    claiming a 4800 water-animal value was NOT applied and must not be re-applied.

Status: PASS, fixed=0, open=0, files=none (code already matches vanilla; tracker rows closed as VERIFIED).
