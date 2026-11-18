# Final rows from docs/tracker/tentities-dolphin.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- Gap 1 (ageable/breedable model, was STILL OPEN): NO GAP, closed. Decompiled `Dolphin.java:74` is `public class Dolphin extends AgeableWaterCreature`, with `BABY_SCALE` (`:81`), `getBreedOffspring` (`:108`) and `finalizeSpawn` using `new AgeableMob.AgeableMobGroupData(0.1F)` (`:103`). Pumpkin matches, so no code change was needed.

## Result

All rows from `docs/tracker/tentities-dolphin.md` were re-verified against the decompiled vanilla shipped in this worktree (`.vendor/vanilla-decompiled/net/minecraft/`). That vanilla has `Dolphin extends AgeableWaterCreature` with babies/breeding and gates feeding on `ItemTags.FISHES`, so every gap from the original 1.21-based tracker is NO GAP. No `.rs` file needed a change, so no rustfmt was run and no unit test was added (no code seam changed).

- Gap 1 (ageable/breedable model): VERIFIED - no gap. `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/dolphin/Dolphin.java:74` is `public class Dolphin extends AgeableWaterCreature`; `BABY_SCALE` at `:81`, `getBreedOffspring` at `:108`, `finalizeSpawn` at `:98-104` uses `new AgeableMob.AgeableMobGroupData(0.1F)`. Pumpkin `impl AgeableMob` (`dolphin.rs:148`) with `default_baby_spawn_chance` 0.1 (`:153`), `impl AgeableWaterCreature` (`:158`), `finalize_ageable_spawn` (`:325`) and the `DATA_BABY_ID`/`AGE_LOCKED` data-tracker writes (`:328-334`) match. The tracker's proposed removal would break parity.

- Gap 2 (dolphin_food tag): VERIFIED - no gap, not data-blocked. `Dolphin.java:321-339` `mobInteract` gates on `ItemTags.FISHES` (`:323`) with `canAgeUp` -> `ageUp(getSpeedUpSecondsWhenFeeding(-age), true)` else `setGotFish(true)`; this vanilla defines no `dolphin_food` tag. Pumpkin `dolphin.rs:412-431` checks `tag::Item::MINECRAFT_FISHES` with the same age-up/else-GotFish branch. No missing generated field.

- Gap 3 (DolphinJumpGoal::should_continue): VERIFIED - no gap. `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/goal/DolphinJumpGoal.java:56-59` is `(!(yd*yd<0.03F) || getXRot()==0.0F || !(abs(getXRot())<10.0F) || !isInWater()) && !onGround()`. Pumpkin `goals.rs:427-435` `(velocity.y.powi(2) >= 0.03 || pitch == 0.0 || pitch.abs() >= 10 || !is_in_water()) && !on_ground` is its exact boolean equivalent. The proposed negated-conjunction rewrite would be wrong.
