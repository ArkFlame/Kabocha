# Tracker: entities / Dolphin (RE-REVIEW)

Audited `crates/pumpkin-core/src/entity/passive/dolphin.rs` and
`crates/pumpkin-core/src/entity/passive/dolphin/goals.rs` at `0a35a87c3`.
Target `CURRENT_MC_VERSION` = 26.3. No `.vendor/` decompiled source in this worktree, so
vanilla facts below are cross-checked against generated vanilla data (`pumpkin-data`)
and the known `Dolphin` / `Dolphin$*Goal` / `JumpGoal` classes.
The change that triggered the re-review is `0a35a87c3` (entity cycles and AI targets moved
to `Weak`); its dolphin part is reviewed and clean (every stored `Weak` is upgraded before
use, `retained_player` is set on start and cleared on `stop`/failed `can_start`, `tick_links`
still sees the id).

## Fixed in this pass

- None. Every candidate left below needs a decompiled vanilla body to confirm.

## Refuted: the three gaps recorded in `docs/tracker/tentities-dolphin.md`

The previous review's "remaining gaps" are wrong; do not implement them.

1. "Dolphin is mistakenly modelled as an ageable/breedable mob" — refuted.
   `crates/pumpkin-data/src/generated/tracked_data.rs` generates `DATA_BABY_ID` (id 16) and
   `AGE_LOCKED` (id 17) for `dolphin` *and* `squid`, while `cod` (`AbstractFish`) has neither.
   `AGE_LOCKED` only exists on `AgeableMob`, so in 26.3 `WaterAnimal` is an `AgeableMob` and
   the dolphin really does carry baby/age-lock synched data. `impl AgeableMob`,
   `finalize_ageable_spawn`, the tracker writes and the `can_attack` baby gate all stay.
2. "Food should be `minecraft:dolphin_food`, not `ItemTags.FISHES`" — refuted.
   The generated item tags contain 30 `minecraft:*_food` tags (`axolotl_food`, `cat_food`,
   `nautilus_food`, `zombie_horse_food`, ...) and no `dolphin_food`
   (`grep DOLPHIN_FOOD crates/pumpkin-data/src/generated/tag.rs` empty, no `dolphin_food`
   in `assets/`), i.e. every vanilla item tag is generated and vanilla has no dolphin food
   tag. `ItemTags.FISHES` at `dolphin.rs:414` is the vanilla gate.
3. "`DolphinJumpGoal::should_continue` looks mis-transcribed (disjunction vs conjunction)" —
   refuted. `goals.rs:427-435` is exactly the shared vanilla base class
   `net.minecraft.world.entity.ai.goal.JumpGoal.canContinueToUse`:
   stop when `!(dy*dy >= 0.03 || xRot == 0 || |xRot| >= 10 || !isInWater())`, i.e. the Rust
   disjunction AND `!onGround`. The disjunction is vanilla, not a transcription error.

## Remaining gaps (unverified, need the decompiled body)

1. **Baby spawn chance 0.1 vs `AgeableMob` default 0.05** —
   `crates/pumpkin-core/src/entity/passive/dolphin.rs:153-155`.
   Vanilla `Dolphin.finalizeSpawn` only sets air supply and `xRot` and then calls super, so
   natural spawns should use `AgeableMobGroupData`'s default `0.05f`
   (`crate::entity::ageable::DEFAULT_BABY_SPAWN_CHANCE`), not 0.1. If confirmed, delete the
   override (the trait default already returns 0.05) or return the constant. Size S.

2. **Missing `Moistness` tag resets to 2400** — `dolphin.rs:405`.
   Port uses `nbt.get_int("Moistness").unwrap_or(2400)`; `Dolphin.readAdditionalSaveData` is
   believed to be `setMoistnessLevel(compound.getInt("Moistness"))`, which yields 0 when the
   key is absent (26.3 may use `getIntOr("Moistness", 2400)` instead — check before editing).
   The test at `dolphin.rs:575-577` encodes the current choice and would need to change with
   it. Size S.

3. **Jump clearance test may over-constrain** — `crates/pumpkin-core/src/entity/passive/dolphin/goals.rs:416-425`.
   Port requires water AND `!BLOCKS_DOLPHIN_JUMP` AND two air blocks for every
   `STEPS_TO_CHECK` step. Vanilla `DolphinJumpGoal.canUse` is believed to accept a step when
   either `waterIsClear(...)` or a second `shoalIsClear(...)` predicate holds (an OR, not an
   AND), which allows jumps over shallow water/one-block shoals. Confirm the `shoalIsClear`
   body before changing. Size M.

4. **Feeding a baby uses `can_age_up()`** — `dolphin.rs:417`.
   Vanilla `Dolphin.mobInteract` branches on `isBaby()` and calls
   `ageUp(getSpeedUpSecondsWhenFeeding(-getAge()), true)` regardless of the age lock;
   `can_age_up()` is `is_baby() && !is_age_locked()`, so an age-locked baby takes the
   `set_got_fish(true)` branch instead. Check whether `AgeableMob.ageUp` honours the lock in
   26.3; if it does not, drop the lock from the condition. Size S.

## Checked, no gap

- `TOTAL_AIR_SUPPLY` 4800 / `mob_max_air_supply` / `mob_increase_air_supply` refill, no-AI
  refill in `post_tick`, moistness 2400 on `is_in_water_or_rain`, decrement, `DRY_OUT` at 0,
  on-ground hop (0.2/0.5/0.2, random yaw, clears on-ground, marks velocity dirty).
- `GOT_FISH` / `MOISTNESS_LEVEL` ids 18/19 in generated data, default writes in
  `mob_init_data_tracker`, NBT keys `GotFish` / `Moistness`, `finalize_spawn` air + `xRot(0)`.
- Goal priorities 0/0/1/2/4/4/5/5/6/7/8/8/8 and the guardian flee (8.0/1.0/1.0, priority 9)
  and `RevengeGoal` ignoring `GUARDIAN_LIKE`; treasure search
  (`DOLPHIN_LOCATED`, radius 50, status 38, re-broadcast `get_tick_count(80)`, 1.3 speed).
- `SmoothSwimmingMoveControl(85, 10, 0.02, 0.1, true)`, `SmoothSwimmingLookControl(10)`,
  water-bounded navigator with 0 water malus, `travel_in_water` (input speed, `*0.9`,
  `-0.005` sink without target), `mob_is_pushed_by_fluids false`, head limits 1/1,
  ambient sound switch on `is_in_water`, jump/attack/death/hurt/eat/play sounds, spawn
  obstruction, XP reward 1-3, `additional_tick_movement` reach budget.
