# entities / Tadpole

Coverage: RE-REVIEW (reviewed source changed).
Re-reviewed 2026-11-17 against vanilla `net.minecraft.world.entity.animal.Tadpole`, `AbstractFish` and
`WaterAnimal`, after commits `556733f82` (tadpole port), `bbbb755a8` (bucket state, sound volume) and
`0505dd44e` (air supply for all living entities), which invalidated the previous water-animal assessment.

## Fixed in this pass
- `crates/pumpkin-core/src/entity/passive/water_animal.rs:11-27` - `WaterAnimal::handle_air_supply` used
  `breath::MAX_AIR` (300) both as the refill value and, through `EntityBase::get_max_air_supply`, as the
  entity maximum. Vanilla `WaterAnimal.getMaxAirSupply()` / `handleAirSupply` use
  `WATER_ANIMAL_MAX_AIR_SUPPLY = 4800`, so tadpoles and all fish refilled to 300 and drowned on land in
  ~15 s instead of ~4 min. Added `WATER_ANIMAL_MAX_AIR_SUPPLY: i32 = 4800` and used it for the refill.
- `crates/pumpkin-core/src/entity/passive/abstract_fish.rs:304-306` - the `fish_mob_hooks!` macro now
  overrides `Mob::mob_max_air_supply` with 4800, so spawn air (`entity/type.rs:369-376`), NBT clamping and
  client sync all see the vanilla maximum. Regression assert added in `water_animal.rs:90`.

## Verified matching (no gap)
- `tadpole.rs:41-47` constants: 24000 ticks to frog, sound volume 0.15, SmoothSwimmingMoveControl
  85/10/0.02/0.1(true), SmoothSwimmingLookControl 10. Match `Tadpole` field initialisers.
- `tadpole.rs:160-239` `fish_mob_hooks!` + `AbstractFish`: silent step, no ambient sound, flop sound, no
  fluid push, no leash, `removeWhenFarAway = !fromBucket && !hasCustomName`,
  `requiresCustomPersistence = fromBucket || vehicle || leashed` - matches `AbstractFish`.
- `abstract_fish.rs:52-55` goals: panic 1.25, FISH_FLEES avoid, RandomSwimming(1.0, 40).
- `tadpole/ai.rs:37-96` CORE/IDLE activities, 2.0 panic, 0.5 idle swim, 1.25 temptation, look-at-player
  6.0/30..60, gate ORDERED/TRY_ALL weights 2/3/5 - matches `Tadpole#makeBrain` except gap 3 below.
- `tadpole.rs:184-194`, `279-296`: NBT/bucket `Age` + `AgeLocked` and `FromBucket`, matching
  `Tadpole#addAdditionalSaveData` / `saveToBucketTag`.
- `tadpole.rs:254-257` `sound_volume = 0.15` (vanilla `getSoundVolume`), used by the grow-up sound at
  `tadpole.rs:143`.
- `tadpole.rs:260-277` bucket item `TADPOLE_BUCKET` / `ItemBucketFillTadpole` match
  `Tadpole#getBucketItemStack` / `getPickupSound`.
- `age_up` (`tadpole.rs:120-146`) converts on server, finalizes spawn with `Conversion`, persistence
  required, size-change fudge, grow-up sound - matches `Tadpole#ageUp` apart from gap 2.
- `Mob::base_tick` now runs the living-entity air rules for every mob (`entity/mob/mod.rs:2134-2138`),
  and fish override it with `fish_base_tick`/`handle_air_supply` as vanilla `WaterAnimal` does.

## Remaining gaps
1. `crates/pumpkin-core/src/entity/passive/tadpole.rs:49-55` - no `TadpoleVariant` (temperate/warm/cold):
   no field, no synced `DATA_VARIANT`, no biome pick in `finalize_spawn`, no NBT/bucket `variant` key.
   Vanilla class: `Tadpole` / `TadpoleVariant`. `pumpkin-data` generates no tadpole variant
   (`crates/pumpkin-data/src/generated/tracked_data.rs:17973-17978` has only `AGE_LOCKED`), so tracked
   data cannot be hand-written. Fix: Extractor entry for `tadpole_variant`, codegen `TadpoleVariant`
   (id, name, `frog_variant`) plus `DATA_VARIANT`, then field + getter/setter + `finalize_spawn` biome
   pick + NBT/bucket persistence. Size: L (blocked on assets/codegen).
2. `crates/pumpkin-core/src/entity/passive/tadpole.rs:131-145` - `age_up` does not carry the tadpole
   variant into the frog; the frog variant is only derived from the biome at conversion time
   (`crates/pumpkin-core/src/entity/passive/frog.rs:146-148`, called from `frog.rs:267`/`274`), so a
   tadpole grown outside its spawn biome yields the wrong frog. Vanilla `Tadpole#ageUp` uses
   `TadpoleVariant#frogVariant()`. Fix: after gap 1, call
   `frog.set_variant(get_variant().frog_variant())` in the conversion callback after `finalize_spawn`
   so it wins. Size: M (depends on gap 1).
3. `crates/pumpkin-core/src/entity/passive/tadpole/ai.rs:85-91` - the gate weight-5 entry is a no-op
   `trigger_if` (`is_in_water`); vanilla `Tadpole#makeBrain` wraps `RandomLookAround(0.001F)` in
   `RunIf(Entity::isInWater, ...)`, so idle tadpoles never pick a random look target in water. The same
   placeholder remains at `frog/ai.rs:107` and `axolotl/ai.rs:215,221`, and no `RunIf` combinator exists
   under `crates/pumpkin-core/src/entity/ai/brain/behavior/` (`RandomLookAround` itself does exist,
   `random_look_around.rs:16`). Fix: add `run_if(predicate, Box<dyn BehaviorControl>)` and use
   `run_if(is_in_water, RandomLookAround::new(0.001))` at weight 5 for all three mobs.
   Size: M (shared helper).
4. `crates/pumpkin-core/src/entity/passive/ageable_water_creature.rs:16-26` - same 300-vs-4800 bug for
   squids: `AgeableWaterCreature::handle_air_supply` still passes `breath::MAX_AIR`, so squid/glow squid
   refill to 300 and `Mob::mob_max_air_supply` stays 300 for them (dolphin overrides both correctly at
   `dolphin.rs:342-347`). Vanilla `WaterAnimal`. Fix: pass `WATER_ANIMAL_MAX_AIR_SUPPLY` there and
   override `mob_max_air_supply` in the squid `Mob` macros. Size: S.

## Gap status
- 1 BLOCKED-DATA: `tadpole_variant` not generated in `pumpkin-data` (`generated/tracked_data.rs:17973-17978` only has AGE_LOCKED); needs Extractor.
- 2 STILL OPEN: depends on gap 1.
- 3 STILL OPEN: RunIf combinator missing under `entity/ai/brain/behavior/`, M.
- 4 STILL OPEN: squid max air supply, S, not attempted in sweep (vanilla WaterAnimal value UNVERIFIED).

## Result

Re-verified every STILL OPEN row against the authoritative decompiled vanilla under
`.vendor/vanilla-decompiled`. No code change was warranted: all three row premises are false for this
vanilla version. No source file was touched, so no rustfmt run was needed.

- Row 2 (carry tadpole variant into frog on age_up) - NO GAP, premise false. Vanilla
  `net/minecraft/world/entity/animal/frog/Tadpole.java` has no `TadpoleVariant`, no variant synched
  data, no `variant` NBT or bucket key; its `ageUp()` callback (lines 236-249) only calls
  `frog.finalizeSpawn`, `frog.setPersistenceRequired`, `fudgePositionAfterSizeChange` and `playSound`.
  No `TadpoleVariant.java` exists anywhere under `.vendor/vanilla-decompiled`. The frog variant is
  chosen from the biome inside `Frog#finalizeSpawn` (`Frog.java:263`, `VariantUtils.selectVariantToSpawn`),
  which Pumpkin already does (`frog.rs:146-148`). Left open: it depends on BLOCKED-DATA gap 1, whose
  tracked data does not exist (`tracked_data.rs:17973+` has no tadpole variant); generating it needs
  an Extractor entry and codegen, and `crates/pumpkin-data/src/generated` must not be hand-edited.
- Row 3 (RunIf + RandomLookAround gate) - NO GAP, premise false. Vanilla
  `net/minecraft/world/entity/animal/frog/TadpoleAi.java:64` uses
  `Pair.of(BehaviorBuilder.triggerIf(Entity::isInWater), 5)`, a no-op, exactly matching Pumpkin's
  `trigger_if(... is_in_water)` at `tadpole/ai.rs:85-90`. There is no `RandomLookAround` or `RunIf`
  in the tadpole brain in this version. Left open also because the proposed fix needs files outside
  this unit: a new `run_if.rs` under `entity/ai/brain/behavior/` and edits to `tadpole/ai.rs`,
  `frog/ai.rs` and `axolotl/ai.rs` (basename `ai.rs` is not an allowed file for this unit).
- Row 4 (squid max air supply) - NO GAP, premise false. Vanilla
  `net/minecraft/world/entity/animal/AgeableWaterCreature.java:37-47` `handleAirSupply` refills to
  `300`, and squid does not override `getMaxAirSupply`; Pumpkin's default
  `Mob::mob_max_air_supply` is `MAX_AIR = 300` (`mob/mod.rs:2078-2080`). `ageable_water_creature.rs:16-26`
  passes `MAX_AIR = 300`, which is correct. The 4800 value is Dolphin-only (`Dolphin.java:78`
  `TOTAL_AIR_SUPPLY = 4800`), already handled in `dolphin.rs:342-347`. Current
  `water_animal.rs:10` and `abstract_fish.rs:303-306` use `WATER_ANIMAL_MAX_AIR_SUPPLY = 300`, matching
  vanilla, so the "Fixed in this pass" note claiming 4800 is stale and must not be re-applied.
