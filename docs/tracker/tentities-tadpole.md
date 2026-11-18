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

## Gap status (h6 sweep, final)
- Gap 1 (variant data): VERIFIED (false premise, matches vanilla). Cite:
  `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/frog/Tadpole.java:43-45`.
- Gap 2 (age-up variant mapping): VERIFIED (false premise, matches vanilla). Cite:
  `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/frog/Tadpole.java:236-250`.
- Gap 3 (max air 4800 for water animals): VERIFIED (false premise, matches vanilla). Cite:
  `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/fish/WaterAnimal.java:39-49`.

## Result

Verified each STILL OPEN row against the vendored decompiled vanilla (Java 26.3) and `assets/`.

- Gap 1 (tadpole variant): CLOSED, false premise, no code change. Decompiled
  `net/minecraft/world/entity/animal/frog/Tadpole.java` has NO `DATA_VARIANT` /
  `TadpoleVariant` (whole class read, lines 1-260); only `Frog` has
  `FrogVariant` (temperate/warm/cold). `assets/` has `frog_variant`, `wolf_variant`,
  `cow_variant`, etc. but no `tadpole_variant` registry, and
  `assets/entity_variants.json` has no tadpole entry. The reviewer conflated
  `FrogVariant` with a nonexistent tadpole variant. Pumpkin correctly has none.
- Gap 2 (age-up variant mapping): CLOSED, false premise, no code change. Depends on
  gap 1. Decompiled `Tadpole#ageUp` (Tadpole.java:236-250) just calls
  `convertTo(EntityTypes.FROG, ConversionParams.single(...))` with no variant
  mapping; the frog variant is chosen from the biome in `Frog.finalizeSpawn`.
  Pumpkin `tadpole.rs:120-146` matches. Also needs generated data/Extractor and
  files outside this unit (`crates/pumpkin-data/src/generated/tracked_data.rs`),
  so it could not be implemented here regardless.
- Gap 3 (water animal max air): FIXED (code change, `water_animal.rs:8-10`).
  Decompiled `WaterAnimal#handleAirSupply`
  (`net/minecraft/world/entity/animal/fish/WaterAnimal.java:39-49`) hardcodes
  `this.setAirSupply(300)`; there is NO `getMaxAirSupply()` override in
  `WaterAnimal`, so base `Entity.getMaxAirSupply()` returns `maxAirTicks` =
  `Entity.TOTAL_AIR_SUPPLY` (300, Entity.java:245,4058-4060). Only `Dolphin`
  overrides to 4800 (Dolphin.java:78,204-211). Therefore the 4800 value written by
  the previous pass was wrong: fish refilled to 4800 and took ~4 min to drown on
  land instead of vanilla ~15 s. Set `WATER_ANIMAL_MAX_AIR_SUPPLY` back to 300 and
  corrected the doc comment. `abstract_fish.rs:304-306` (`mob_max_air_supply`
  override) and `bucketable.rs:193-196` (regression assert) both reference this
  constant, so they now reflect vanilla 300 with no edits needed there.
  `breath.rs` left unchanged: it is player-only (BreathManager::tick takes
  `&Player`) and 300 is correct for players.

Note for orchestrator: `bucketable.rs:192` has a stale inline comment saying
"(4800)" that is now inaccurate; it is outside this unit and was not edited.
Also out of scope (not a STILL OPEN row): the "Fixed in this pass" from_bucket
claim is contradicted by decompiled `Tadpole.java:172-179`, where `fromBucket()`
returns `true` and `setFromBucket()` is a no-op, unlike `AbstractFish`. Pumpkin's
tadpole now stores `from_bucket`, which differs from vanilla for that mob.

Verification: decompiled vanilla + assets read directly; `rustfmt --edition 2024`
run on `water_animal.rs` (exit 0). No cargo/git run per task rules.

## Result

Re-verified all three STILL OPEN rows against vendored decompiled vanilla (Java
26.3) and the current Pumpkin source. No code changes were needed: the source
already matches vanilla on every row.

- Gap 1 (tadpole variant): CLOSED, false premise. `Tadpole.java` defines only
  `AGE_LOCKED` synched data (line 45); there is no `DATA_VARIANT` /
  `TadpoleVariant`, and `assets/` has no tadpole variant registry. `tadpole.rs`
  correctly has none. A fix would need generated data/Extractor outside this unit.
- Gap 2 (age-up variant mapping): CLOSED, false premise. `Tadpole#ageUp`
  (Tadpole.java:236-250) only calls `convertTo(FROG, ...)` plus `finalizeSpawn`;
  no variant is carried over (the frog variant is biome-selected). Pumpkin
  `tadpole.rs:120-146` matches. Also needs generated data outside this unit.
- Gap 3 (water animal max air): CLOSED, already correct. `WaterAnimal.java:39-49`
  hardcodes `setAirSupply(300)` and has no `getMaxAirSupply` override (only
  Dolphin overrides to 4800, dolphin.rs:39). `water_animal.rs:10` already sets
  `WATER_ANIMAL_MAX_AIR_SUPPLY = 300`; `bucketable.rs:192` comment now reads
  (300), with no stale 4800 anywhere in these files; `breath.rs MAX_AIR = 300` is
  player-only (`BreathManager::tick(&Player)`). No code change.

The earlier "Fixed in this pass" from_bucket change flagged in the prior Result
is already corrected in source: `tadpole.rs:260-267` overrides `is_from_bucket()`
to true and makes `set_from_bucket()` a no-op, matching Tadpole.java:172-179.

Verification: decompiled vanilla + assets read directly; no cargo/git per task
rules. No rustfmt run because no .rs file was edited.


## Result

Third re-review. Re-verified all three STILL OPEN gaps against the vendored
decompiled vanilla (Java 26.3). No .rs source was edited: Pumpkin already
matches vanilla on every row.

- Gap 1 (tadpole variant): CLOSED, false premise. Full `Tadpole.java` (260
  lines) defines only `AGE_LOCKED` synched data (line 45). There is no
  `DATA_VARIANT` accessor and no `TadpoleVariant` class in vanilla, and
  `assets/` has no tadpole variant registry. `tadpole.rs` correctly has none.
  Any fix would require generated data and Extractor assets outside this unit.
- Gap 2 (age-up variant mapping): CLOSED, false premise. `Tadpole#ageUp`
  (Tadpole.java:236-250) converts to `FROG` via `ConversionParams.single`,
  calls `finalizeSpawn`, sets persistence, fudges position and plays the
  grow-up sound; no variant is carried over (frog variant is biome-selected).
  `tadpole.rs:120-146` matches. Also needs generated data outside this unit.
- Gap 3 (water animal max air): CLOSED, already correct. `WaterAnimal.java:39-48`
  hardcodes `setAirSupply(300)` and has no `getMaxAirSupply` override. Base
  `Entity.getMaxAirSupply` returns `maxAirTicks`, initialised from
  `getDefaultMaxAirSupply` = `Entity.TOTAL_AIR_SUPPLY` = 300
  (Entity.java:245,438,4058). Only `Dolphin` overrides to 4800
  (Dolphin.java:78,114,204), and `dolphin.rs:39,342` returns 4800 to match.
  `water_animal.rs:10` is 300, `abstract_fish.rs:304-305` returns 300,
  `bucketable.rs:192` comment reads (300), and `breath.rs MAX_AIR=300` is
  player-only (`BreathManager::tick(&Player)`). All consistent.

The existing tests cover the only seams (`tadpole.rs::tadpole_is_always_from_bucket`
and `water_animal.rs::fish_keep_the_drowning_rule`); no new test added because
no behaviour changed. No unwrap/expect/panic and no println introduced. No .rs
file changed, so the compile-pitfalls table is not applicable and no rustfmt was
run. No cargo/git per task rules.

## Result

Fourth re-review. Re-checked all three STILL OPEN rows against the vendored
decompiled vanilla (Java 26.3) and the current Pumpkin source. No .rs edited:
Pumpkin matches vanilla on every row.

- Gap 1 (tadpole variant): CLOSED, false premise. Full `Tadpole.java` (260
  lines) defines only `AGE_LOCKED` synched data (line 45). No `DATA_VARIANT`
  accessor, no `TadpoleVariant` class, and no tadpole variant in `assets/`.
  `tadpole.rs` correctly has none; a fix needs generated data and Extractor
  assets outside this unit.
- Gap 2 (age-up variant mapping): CLOSED, false premise. `Tadpole#ageUp`
  (Tadpole.java:236-250) only calls `convertTo(FROG, ConversionParams.single)`
  plus `finalizeSpawn` and no variant is carried over, so frog variant is
  biome-selected. `tadpole.rs:120-146` matches. Also needs generated data
  outside this unit.
- Gap 3 (water animal max air): CLOSED, already correct. `WaterAnimal.java:39-49`
  hardcodes `setAirSupply(300)` with no `getMaxAirSupply` override. Base
  `Entity.getMaxAirSupply` returns 300; only Dolphin overrides to 4800
  (Dolphin.java, and `dolphin.rs:39` sets 4800). `water_animal.rs:10` is 300,
  `abstract_fish.rs:304-305` returns 300, `bucketable.rs:192` comment reads
  (300), and `breath.rs:9 MAX_AIR=300` is player-only. Consistent.

Also confirmed the fromBucket correction holds: `Tadpole.java:172-179` returns
true and makes `setFromBucket` a no-op, and `tadpole.rs:260-267` matches.

Existing tests cover the only seams; no new test added because no behaviour
changed. No unwrap/expect/panic or println introduced. No .rs changed, so the
compile-pitfalls table is not applicable and no rustfmt was run. No cargo/git
per task rules.


## Result

Fifth re-review (second-pass scope: unrestricted source edits). Independently
re-verified all three STILL OPEN rows against the vendored decompiled vanilla
(Java 26.3) and the current Pumpkin source. No .rs file was edited: every row is
a false premise or already matches vanilla.

- Gap 1 (tadpole variant): CLOSED, false premise. Full `Tadpole.java` (260
  lines) defines only `AGE_LOCKED` synched data (line 45). No `DATA_VARIANT`
  accessor and no `TadpoleVariant` class exist in vanilla, and `assets/` has no
  tadpole variant registry. `tadpole.rs` correctly has none; any fix would need
  generated data and Extractor assets outside the permitted files.
- Gap 2 (age-up variant mapping): CLOSED, false premise. `Tadpole#ageUp`
  (Tadpole.java:236-250) converts to `FROG` with `ConversionParams.single(...)`
  plus `finalizeSpawn`, persistence, position fudge and grow-up sound; no
  variant is carried over (frog variant is biome-selected). `tadpole.rs:120-146`
  matches. Also depends on generated data outside the permitted files.
- Gap 3 (water animal max air): CLOSED, already correct. `WaterAnimal.java:39-49`
  hardcodes `setAirSupply(300)` with no `getMaxAirSupply` override; base
  `Entity.getMaxAirSupply` returns 300 and only `Dolphin` overrides to 4800
  (Dolphin.java:78; `dolphin.rs:39` = 4800). `water_animal.rs:10` = 300,
  `abstract_fish.rs:303-305` returns 300, `bucketable.rs:192` comment reads
  (300), and `breath.rs:9 MAX_AIR` = 300 is player-only
  (`BreathManager::tick(&Player)`). No stale 4800 in fish code.

Also re-confirmed the fromBucket correction holds: `Tadpole.java:172-179`
returns true and makes `setFromBucket` a no-op, and `tadpole.rs:260-267`
matches, covered by `tadpole_is_always_from_bucket`.

Existing tests cover the only seams (`tadpole.rs::tadpole_is_always_from_bucket`
and `water_animal.rs::fish_keep_the_drowning_rule`); no new test added because
no behaviour changed. No unwrap/expect/panic or println introduced. No .rs
changed, so the compile-pitfalls table is not applicable and no rustfmt was
run. No cargo/git per task rules.


## Result

Sixth re-review (third-pass scope). Independently re-verified every previously
STILL OPEN row against the vendored decompiled vanilla (Java 26.3) and the
current Pumpkin source. No .rs file was edited: Pumpkin already matches vanilla
on every row, so no behaviour change and no new test seam exists.

- Gap 1 (tadpole variant): VERIFIED, false premise. Full
  `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/frog/Tadpole.java`
  (260 lines) defines only `AGE_LOCKED` synched data (line 45). No
  `DATA_VARIANT`, no `TadpoleVariant` class (glob: zero matches), no tadpole
  variant in `assets/`. `tadpole.rs` correctly has none.
- Gap 2 (age-up variant mapping): VERIFIED, false premise. `Tadpole#ageUp`
  (`Tadpole.java:236-250`) converts to `FROG` via
  `ConversionParams.single(this, false, false)`, calls `finalizeSpawn`,
  sets persistence, fudges position and plays the grow-up sound; no variant is
  carried over (frog variant is biome-selected). `tadpole.rs:120-146` matches.
- Gap 3 (water animal max air): VERIFIED, false premise. `WaterAnimal`
  (`.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/fish/WaterAnimal.java:39-49`)
  hardcodes `setAirSupply(300)` and has no `getMaxAirSupply` override (only
  `Dolphin` overrides to 4800). `water_animal.rs:10` = 300,
  `abstract_fish.rs:303-305` returns 300, `bucketable.rs:192` comment reads
  (300), `breath.rs:9 MAX_AIR` = 300 is player-only, `dolphin.rs:39` = 4800.
  All consistent with vanilla.

Also re-confirmed the fromBucket correction: `Tadpole.java:172-179` returns
true and makes `setFromBucket` a no-op, matched by `tadpole.rs:260-267` and
covered by `tadpole_is_always_from_bucket`. The gap status header above no
longer contains any STILL OPEN row.

Existing tests cover the only seams; no new test added because no behaviour
changed. No unwrap/expect/panic and no println introduced. No .rs changed, so
the compile-pitfalls table is not applicable and no rustfmt was run. No
cargo/git per task rules.


## Result

Third-pass scope (unrestricted source edits). Independently re-verified every
previously STILL OPEN row against vendored decompiled vanilla (Java 26.3) and
the current Pumpkin source. No .rs file was edited: Pumpkin matches vanilla on
every row, so no behaviour change and no new test seam exists.

- Gap 1 (tadpole variant): VERIFIED, false premise. Full
  `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/frog/Tadpole.java`
  (260 lines) defines only `AGE_LOCKED` synched data (line 45). No
  `DATA_VARIANT` accessor, no `TadpoleVariant` class (grep/glob: zero
  matches), and no tadpole variant in `assets/`. `tadpole.rs` correctly has
  none. A fix would need generated data/Extractor assets outside this unit.
  This is BLOCKED-DATA only if a fix were required; none is, because the
  premise is false.
- Gap 2 (age-up variant mapping): VERIFIED, false premise. `Tadpole#ageUp`
  (`Tadpole.java:236-250`) converts to FROG via
  `ConversionParams.single(this, false, false)`, calls `finalizeSpawn`,
  `setPersistenceRequired`, `fudgePositionAfterSizeChange` and
  `playSound(TADPOLE_GROW_UP, 0.15F, 1.0F)`; no variant is carried over (frog
  variant is biome-selected). `tadpole.rs:120-146` matches. Also depends on
  generated data outside this unit.
- Gap 3 (water animal max air): VERIFIED, false premise.
  `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/fish/WaterAnimal.java:39-49`
  hardcodes `setAirSupply(300)` with no `getMaxAirSupply` override; only
  Dolphin overrides (`.../animal/dolphin/Dolphin.java:78` = 4800,
  `getMaxAirSupply` at :204). `water_animal.rs:10` = 300,
  `abstract_fish.rs:303-305` returns 300, `bucketable.rs:192` comment reads
  (300), `breath.rs:9 MAX_AIR` = 300 is player-only, `dolphin.rs:39` = 4800.
  All consistent with vanilla.

Also re-confirmed fromBucket: `Tadpole.java:172-179` returns `true` and
`setFromBucket` is a no-op, matched by `tadpole.rs:260-267` and covered by
`tadpole_is_always_from_bucket`.

No STILL OPEN rows remain. Existing tests cover the only seams; no new test
added because no behaviour changed. No unwrap/expect/panic or println
introduced. No .rs changed, so the compile-pitfalls table is not applicable and
no rustfmt was run. No cargo/git per task rules.
