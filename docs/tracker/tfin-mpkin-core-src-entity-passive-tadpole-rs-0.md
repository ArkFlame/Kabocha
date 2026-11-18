# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-entity-passive-tadpole-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

All rows closed (final pass). Per-row results are recorded in the Result section below.

## Result

Final pass. Independently re-verified both rows against the vendored decompiled
vanilla (Java 26.3) and the current Pumpkin source. No .rs file was edited: every
row is already in parity with vanilla, so no behaviour change and no new test
seam exists.

- **entities/Tadpole** [partial/AUDITED]: VERIFIED. Full
  `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/frog/Tadpole.java`
  (260 lines) defines only `AGE_LOCKED` synched data (line 45); there is no
  `DATA_VARIANT` accessor and no `TadpoleVariant` class, so the audited variant
  gap is a false premise. `Tadpole#ageUp` (`Tadpole.java:236-250`) converts to
  FROG via `ConversionParams.single(this, false, false)` with no variant carried
  over, matched by `tadpole.rs:122-148`. `fromBucket`/`setFromBucket`
  (`Tadpole.java:172-179`) return true/no-op, matched by `tadpole.rs:257-264`.
  Tadpole has no `getSoundVolume` override; only the grow-up sound is 0.15F
  (`Tadpole.java:242`), matched by `SOUND_VOLUME` and `tadpole.rs:243-255` (no
  `sound_volume` override, so the trait default 1.0 from `custom_sound.rs:8-10`
  applies). The previously flagged sound-volume regression is already fixed.
- **own/tentities-tadpole** [partial/OWN-OPEN]: VERIFIED. All three own-audit
  gaps are closed. Gap 1 (variant) and Gap 2 (age-up variant) are false premises
  citing `.../animal/frog/Tadpole.java:45,236-250`. Gap 3 (water animal max air)
  is a false premise:
  `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/fish/WaterAnimal.java:39-49`
  hardcodes `setAirSupply(300)` with no `getMaxAirSupply` override (only Dolphin
  overrides to 4800), matched by `water_animal.rs:10` = 300 and `abstract_fish.rs`
  max-air = 300. The stale bucketable "(4800)" comment was already corrected.
  The fromBucket and sound-volume fixes from earlier passes are present in
  source. No BLOCKED-DATA: no generated field is required because vanilla has no
  tadpole variant field and water-animal max air is the constant 300.

Existing tests cover the only seams (`tadpole.rs::tadpole_is_always_from_bucket`,
`tadpole.rs::tadpole_sound_volume_matches_vanilla`,
`water_animal.rs::fish_keep_the_drowning_rule`); no new test added because no
behaviour changed. No unwrap/expect/panic and no println introduced. No .rs file
changed, so the compile-pitfalls table is not applicable and no rustfmt was run.
No cargo/git per task rules. Status: PASS
