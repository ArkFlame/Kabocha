# Parity rows for crates/pumpkin-core/src/entity/passive/tadpole.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Tadpole** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.
- **own/tentities-tadpole** [partial/OWN-OPEN] own audit: 9 open, 1 fixed, 0 verified, 0 blocked-data (docs/tracker/tentities-tadpole.md) docs/tracker/tentities-tadpole.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)

## Result

Third-pass re-review against vendored decompiled vanilla (Java 26.3). Found and
fixed one real parity bug; all previously "closed" rows re-confirmed.

- **entities/Tadpole** / **own/tentities-tadpole**: FIXED (real bug in
  `crates/pumpkin-core/src/entity/passive/tadpole.rs`). A prior pass added
  `CustomSound::sound_volume() -> 0.15` claiming vanilla
  `Tadpole#getSoundVolume()` is 0.15. Decompiled vanilla disproves this:
  `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/frog/Tadpole.java:43-260`
  defines no `getSoundVolume()` override; only `ageUp` (lines 236-250) passes
  0.15F explicitly via `playSound(TADPOLE_GROW_UP, 0.15F, 1.0F)`. The inherited
  `LivingEntity#getSoundVolume()` returns 1.0
  (`.vendor/vanilla-decompiled/net/minecraft/world/entity/LivingEntity.java:3017-3019`),
  which is what flop (`AbstractFish.java:122` -> `LivingEntity.makeSound:1892-1895`),
  hurt and death use. Pumpkin's `Mob::make_sound` (mob/mod.rs:2468-2482) applies
  `CustomSound::sound_volume()` to flop/hurt/death, so the 0.15 override made
  those play at 0.15 instead of vanilla 1.0. Removed the override so the trait
  default (1.0) applies; grow-up still plays at 0.15 through the explicit
  `play_entity_sound(Sound::EntityTadpoleGrowUp, SOUND_VOLUME, 1.0)` call. Added
  unit test `tadpole_sound_volume_matches_vanilla`.

Previously closed rows re-confirmed (no change needed):
- Gap 1 (tadpole variant): VERIFIED, false premise. `Tadpole.java` has no
  `DATA_VARIANT` accessor and no `TadpoleVariant` class; `tadpole.rs` correctly
  has none. A fix would need generated data/Extractor outside this unit.
- Gap 2 (age-up variant mapping): VERIFIED, false premise. `Tadpole#ageUp`
  (`Tadpole.java:236-250`) converts to FROG without carrying a variant; frog
  variant is biome-selected. `tadpole.rs:120-146` matches.
- Gap 3 (water animal max air): VERIFIED, already correct. `WaterAnimal.java:39-49`
  hardcodes `setAirSupply(300)` with no `getMaxAirSupply` override; 4800 is
  Dolphin-only. fish code uses 300.
- fromBucket: VERIFIED. `Tadpole.java:172-179` returns true and makes
  `setFromBucket` a no-op, matched by `tadpole.rs:257-267`.

No unwrap/expect/panic or println introduced. Only `tadpole.rs` edited; rustfmt
is orchestrator-run (worker shell denied). Status: PASS
