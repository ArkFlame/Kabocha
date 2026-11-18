# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-entity-pas.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: panda.rs roll.rs sneeze.rs

Verified each STILL OPEN row against decompiled vanilla `net/minecraft/world/entity/animal/panda/Panda.java`
Fresh pass against decompiled vanilla at `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/panda/Panda.java` (protocol 777). The current main-checkout source already matches vanilla on every STILL OPEN row; no source edits were needed. The note's claimed roll fix (keep rollDelta.x/z on hops) and its mobInteract `canPerformAction()` variant are not present in the checkout and do not match this decompiled source.
Every STILL OPEN row was checked line by line; no source edit is warranted in this unit's files
Resumed pass: verified all STILL OPEN rows against decompiled vanilla at `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/panda/Panda.java` (protocol 777). No source edits made to `panda.rs`, `roll.rs`, or `sneeze.rs`.

## Result
FINAL PASS closed. Every row VERIFIED against decompiled vanilla
`.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/panda/Panda.java` (Paper, protocol 777).
No source edit needed: the only editable source file, `roll.rs`, already matches vanilla; the claimed
roll fix would break parity and was correctly not applied. No BLOCKED-DATA rows.

- Fixed-in-note roll claim (`panda/roll.rs:60-98`): VERIFIED, no edit. Vanilla `handleRoll`
  (`Panda.java:485-504`) sets `(rollDelta.x, movement.y, rollDelta.z)` on normal steps and zeroes the
  horizontal push to `(0, onGround ? 0.27 : movement.y, 0)` on hop steps 7/15/23. `roll_velocity`
  matches exactly; step 1 uses `yaw * DEG_TO_RAD`, baby multiplier 0.1 / adult 0.2, and +0.27 vertical,
  and the counter ends the roll past 32. The note's suggested change (keep `rollDelta.x/z` on hops)
  contradicts `Panda.java:500` and was not applied. `PandaRollGoal` (`roll.rs:153-199`) matches vanilla
  `PandaRollGoal` (`Panda.java:1001-1043`): bounds 60/500 through `reducedTickDelay`, abs>0.5 step with
  Mth.sign, air check at offset (x,-1,z), MOVE|LOOK|JUMP flags, `canContinueToUse=false`,
  `isInterruptable=false`, and roll(true) on start.
- Fixed-in-note thunder claim (`panda.rs:181-192`): VERIFIED, no edit (outside this unit's editable
  source file; `thunder_scares` already gates on skylight and no ceiling).
- Row 1 genes (`panda.rs:122-171`): VERIFIED against `Panda.java:566-594,755-770`. Two-parent
  `setGeneFromParents` RNG order and two independent `nextInt(32)` mutation rolls match
  `getOneOfGenesRandomly`/`Gene.getRandom`; covered by `foal_genes_come_from_the_parents_and_mutate_independently`.
- Row 2 bamboo range (`panda/goals.rs:134-159`): VERIFIED against `Panda.java:834-852`. `canFindBamboo`
  walks r in 0..8, yOff in 0..3; union x,z in -7..=7, y in 0..=2 matches `BAMBOO_SEARCH_RADIUS=7` and
  `BAMBOO_HEIGHT=2`, so bamboo at +/-7 is in range in both.
- Row 3 mobInteract food guard (`panda.rs:624-638`): VERIFIED against `Panda.java:635-657`. Vanilla
  gates the adult food branch on `isSitting() || isInWater()` after the `isBaby()`/breeding passes;
  `sit_and_eat` checks `is_baby() || is_sitting() || is_in_water()`. The `canPerformAction()` variant
  is not present in this decompiled source.
- Row 4 playAttackSound (`panda.rs:878-881`, `goals.rs`): VERIFIED against `Panda.java:324-329`. `didBite`
  is set on every swing and the bite sound fires only on a landed hit via `LivingEntity.doHurtTarget`;
  Rust calls `on_attack`/`EntityPandaBite` only when `try_attack` succeeds.
- Row 5 sneeze reach (`sneeze.rs:96-121,186-188`): VERIFIED against `Panda.java:506-531`. `after_sneeze`
  inflates 10, jumps non-baby/grounded/no-water/`canPerformAction` pandas and drops gift with
  `mob_drops` on; `PandaSneezeGoal::tick_reach` reports `SNEEZE_JUMP_RANGE=10`. `has_audited_tick`
  correctly stays false; the conservative entity-island reach already covers the 10-block jump.
