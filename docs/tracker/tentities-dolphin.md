# Tracker: entities / Dolphin (RE-REVIEW)

Audited `crates/pumpkin-core/src/entity/passive/dolphin.rs` and
`crates/pumpkin-core/src/entity/passive/dolphin/goals.rs` against vanilla `Dolphin`
(1.21 `Dolphin` + `Dolphin$DolphinSwimToTreasureGoal` / `$DolphinSwimWithPlayerGoal` /
`$DolphinJumpGoal` / `$PlayWithItemsGoal`).
No `.vendor/` decompiled source is present in this worktree, so vanilla facts are from those
classes. The change that triggered this re-review is `0a35a87c3` (entity cycles moved to
`Weak`); its dolphin part is reviewed below and is clean.

## Fixed in this pass

- None. No small, certain dolphin fix was found that did not depend on missing vanilla data
  (see gap 2) or on a shared `AgeableWaterCreature` refactor (gap 1).

## Reviewed change: `Weak` entity refactor (no gap)

- `goals.rs:282` / `:648` store `Weak<dyn EntityBase>`; every use upgrades
  (`:288`, `:334-336`, `:359`, `:688-691`, `:699-702`, `:723`) and `can_start` clears the
  slot when it does not start (`:326-329`). `retained_player` (entity-island reach, read by
  `dolphin.rs:224-233` `tick_links`) is set on start and cleared on `stop`/failed `can_start`
  (`:324-329`, `:352-356`, `:710-719`), so no stale link survives a goal.
- `dolphin.rs:173-175` `has_global_tick_reach` now uses `is_leashed()`, `:392` uses
  `get_target()`; both are equivalent to the previous lock reads.

## Remaining gaps

### 1. Dolphin is modelled as an ageable/breedable mob (M)
- File: `crates/pumpkin-core/src/entity/passive/dolphin.rs:148-156` (`impl AgeableMob`),
  `:153-155` (0.1 baby spawn chance), `:317-326` (`finalize_spawn` ->
  `finalize_ageable_spawn`), `:328-340` (`mob_init_data_tracker` writes
  `ageable_mob::DATA_BABY_ID` / `AGE_LOCKED`), `:412-428` (`mob_interact` age-up branch),
  `:158` (`impl AgeableWaterCreature`).
- Vanilla: `Dolphin extends WaterAnimal`, which is not `AgeableMob`; dolphins have no baby
  variant, no breeding, no age lock, and feeding fish only sets `GotFish` (it never ages up).
  The dolphin synched-data list is `GOT_FISH`, `MOISTNESS_LEVEL` only, so the extra
  `DATA_BABY_ID`/`AGE_LOCKED` payload is sent to clients that do not define those indices.
- Fix: give the dolphin its air/XP/spawn-obstruction helpers from `water_animal` directly
  (like `dolphin.rs:277-283` already does) instead of through `AgeableWaterCreature`, drop
  `impl AgeableMob`, `ageable_data`, the ageable data-tracker writes, the age-up branch in
  `mob_interact`, and `AgeableWaterCreature` -> keep that trait only for genuinely ageable
  water mobs (turtle). Note `squid.rs:336` has the same mistake, so both should be changed
  together or a `WaterCreature` non-ageable helper introduced.

### 2. Dolphin food uses the `fishes` tag instead of `minecraft:dolphin_food` (S)
- File: `crates/pumpkin-core/src/entity/passive/dolphin.rs:414`
- Vanilla `Dolphin.mobInteract` gates on `ItemTags.DOLPHIN_FOOD_ITEMS`
  (`cod`, `salmon`, `tropical_fish`), not `ItemTags.FISHES`.
- Pumpkin accepts `minecraft:fishes` (`crates/pumpkin-data/src/generated/tag.rs:25440-25450`:
  cooked cod, cooked salmon, pufferfish, tropical fish, raw cod, raw salmon). Feeding a
  pufferfish (or cooked fish) therefore plays `EntityDolphinEat`, consumes the stack and sets
  `GotFish` (which starts the treasure goal) where vanilla falls through to
  `super.mobInteract` (no reaction) for anything outside `dolphin_food`.
- Fix: check a `minecraft:dolphin_food` item tag (raw cod/salmon/tropical fish). Blocked on
  data: `grep 'dolphin_food\|DOLPHIN_FOOD'` in
  `crates/pumpkin-data/src/generated/tag.rs` has no match while every other mob food tag
  (`MINECRAFT_CAT_FOOD`, `MINECRAFT_OCELOT_FOOD`, ...) is generated, so the vanilla item tag
  is not extracted into `assets/` yet; needs the Extractor/codegen tag dump before the check
  can be data-driven (do not hardcode the item ids). Confirm the tag contents from the
  vanilla tag json at the same time.

### 3. `DolphinJumpGoal::should_continue` condition looks mis-transcribed (S)
- File: `crates/pumpkin-core/src/entity/passive/dolphin/goals.rs:427-435`
- Vanilla `DolphinJumpGoal.canContinueToUse`: stop when
  `dy*dy < 0.03 && (xRot == 0 || |xRot| >= 10 || isInWater())`, i.e. continue when
  `!(dy*dy < 0.03 && ...) && !onGround`.
- Pumpkin ORs the four clauses (`dy² >= 0.03 || pitch == 0.0 || |pitch| >= 10 ||
  !is_in_water()`), which keeps the goal alive on flat/steep pitch even at zero vertical
  speed and, conversely, ends it on a fast in-water tick. The likely intent was the negated
  conjunction, not a disjunction.
- Fix: rewrite as `(dy*dy >= 0.03 || (pitch != 0.0 && pitch.abs() < 10.0 &&
  !entity.is_in_water())) && !entity.on_ground.load(Relaxed)` and re-check the tick pitch
  update. Verify against a decompiled `Dolphin$DolphinJumpGoal` before changing; the goal
  pitch/velocity writes (`:459-482`) already look vanilla-correct.

## Checked, no gap
- Air supply: `TOTAL_AIR_SUPPLY` 4800, `mob_max_air_supply`/`mob_increase_air_supply`
  returning the max (vanilla `Dolphin.increaseAirSupply` refills instantly),
  `base_tick` -> `tick_air_supply`.
- Moistness: `post_tick` (`dolphin.rs:354-381`) resets to 2400 in water/rain, decrements
  otherwise, `DRY_OUT` damage at 0, and the on-ground random hop (0.2/0.5/0.2, random yaw,
  clears on-ground, marks velocity dirty) matches vanilla `aiStep`.
- Synched data values (`GOT_FISH`, `MOISTNESS_LEVEL`) and NBT keys (`GotFish`, `Moistness`)
  match vanilla `addAdditionalSaveData`/`readAdditionalSaveData`.
- Goal priorities 0/0/1/2/4/4/5/5/6 (`dolphin.rs:103-109`, `goals.rs:43-92`) and the
  guardian flee (8.0, 1.0, 1.0 at priority 9) plus `RevengeGoal` ignoring `GUARDIAN_LIKE`
  match vanilla `registerGoals`.
- `SmoothSwimmingMoveControl(85, 10, 0.02, 0.1, true)`,
  `SmoothSwimmingLookControl(10)`, water-bounded navigator with 0 water malus,
  `mob_is_pushed_by_fluids == false`, head rotation limits 1/1, ambient sound switching on
  `is_in_water`, jump/attack/death/hurt/eat/play sounds, spawn obstruction, XP reward.
- `additional_tick_movement` (`dolphin.rs:178-218`) matches the vanilla entity-island reach
  budget for the two movement goals (jump 0.6/0.7, breathe 0.015, follow 0.01, 4 moves/tick).

## Gap status
- 1 ageable/breedable model (M): CLOSED, NO GAP (verified against decompiled vanilla 26.3)
- 2 dolphin_food tag: CLOSED, NO GAP (vanilla uses FISHES; not data-blocked)
- 3 DolphinJumpGoal::should_continue (S): CLOSED, NO GAP (matches decompiled vanilla)


## Result

Verified every gap against the decompiled vanilla shipped in this worktree
(`.vendor/vanilla-decompiled/net/minecraft/`), which targets Pumpkin's protocol version 26.3.
That vanilla differs from the 1.21 classes this tracker was originally written from: `Dolphin`
extends `AgeableWaterCreature`, has babies and breeding, and gates feeding on `ItemTags.FISHES`.
All three gaps are therefore invalid for the target version and no code change was needed.

- Gap 1 (ageable/breedable model, was STILL OPEN): NO GAP, closed. Decompiled `Dolphin.java:74` is
  `public class Dolphin extends AgeableWaterCreature`, with `BABY_SCALE` (`:81`), `getBreedOffspring`
  (`:108`) and `finalizeSpawn` using `new AgeableMobGroupData(0.1F)` (`:103`). Pumpkin's
  `impl AgeableMob` + `impl AgeableWaterCreature`, `default_baby_spawn_chance` 0.1
  (`dolphin.rs:148-158`), `finalize_ageable_spawn` (`:325`) and `DATA_BABY_ID`/`AGE_LOCKED` tracker
  writes (`:328-340`) match vanilla. Removing them as the gap proposed would break parity, so no
  shared refactor was made.
- Gap 2 (dolphin_food tag, was BLOCKED-DATA): NO GAP, closed, not data-blocked. Decompiled
  `Dolphin.mobInteract` (`:321-339`) gates on `ItemTags.FISHES` (`:323`); for fish it ages up via
  `ageUp(getSpeedUpSecondsWhenFeeding(-age), true)` when `canAgeUp()` else sets `GotFish`. Pumpkin
  `dolphin.rs:412-431` checks `MINECRAFT_FISHES`, plays `EntityDolphinEat`, decrements the stack and
  uses the same age-up/else-GotFish branching. This vanilla has no `dolphin_food` tag, so the
  generated-data blocker does not apply.
- Gap 3 (DolphinJumpGoal::should_continue, no status in the original tracker): NO GAP, confirmed
  correct. Decompiled `DolphinJumpGoal.canContinueToUse` (`:56-60`) is
  `(!(yd*yd < 0.03) || xRot == 0 || !(abs(xRot) < 10) || !isInWater()) && !onGround()`; Pumpkin
  `goals.rs:427-435` is its exact boolean equivalent (including `pitch.abs() >= 10.0` as the
  negation of `abs(xRot) < 10`). The proposed negated-conjunction rewrite would be wrong, so no
  change was made.

No `.rs` file was touched, so no rustfmt run and no unit test added (no code seam changed).


## Result (re-verification, this pass)

Independently re-checked all three gaps against the shipped decompiled vanilla
(.vendor/vanilla-decompiled/net/minecraft/) and the current Pumpkin source. Every
gap remains NO GAP; no code was changed.

- Gap 1: Dolphin.java:74 is "public class Dolphin extends AgeableWaterCreature",
  with BABY_SCALE (:81), getBreedOffspring (:108) and finalizeSpawn (:98).
  Pumpkin impl AgeableMob (dolphin.rs:148), impl AgeableWaterCreature (:158),
  0.1 baby chance (:153), finalize_ageable_spawn (:325) and the
  DATA_BABY_ID/AGE_LOCKED writes (:328-340) match. Removing them as proposed
  would break parity; keep as is.
- Gap 2: Dolphin.mobInteract (:321-339) gates on ItemTags.FISHES (:323);
  this vanilla has no dolphin_food tag. Pumpkin dolphin.rs:414 uses
  MINECRAFT_FISHES with the same age-up/else-GotFish branch. Not data-blocked.
- Gap 3: decompiled ai/goal/DolphinJumpGoal.java:56-59 is
  (!(yd*yd<0.03) || xRot==0 || !(abs(xRot)<10) || !isInWater()) && !onGround().
  Pumpkin goals.rs:427-435 is its exact boolean equivalent. Keep as is.

No .rs file was touched, so no rustfmt run and no unit test added (no code seam changed).
