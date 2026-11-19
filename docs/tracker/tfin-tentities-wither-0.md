# Final rows from docs/tracker/tentities-wither.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- Row 2, 3, 5, 7, 8, 9: STILL OPEN, not attempted this pass (row 5 needs bossbar progress derivation, row 7 needs Gaussian source confirmation).
Independent verification against `.vendor/vanilla-decompiled/net/minecraft/world/entity/boss/wither/WitherBoss.java` for the STILL OPEN rows (2, 3, 5, 7, 8, 9). Source already matches vanilla; no source edits were needed this pass.
Independent re-verification of the STILL OPEN rows (2, 3, 5, 7, 8, 9) against `.vendor/vanilla-decompiled/net/minecraft/world/entity/boss/wither/WitherBoss.java` and `.../ai/control/FlyingMoveControl.java`. Every row already matches vanilla; no source edits were needed this pass.
Independent verification of every remaining STILL OPEN row (2, 3, 5, 7, 8, 9) against

## Result

Final pass: every remaining STILL OPEN row (2, 3, 5, 7, 8, 9) closed against
`.vendor/vanilla-decompiled/net/minecraft/world/entity/boss/wither/WitherBoss.java`
and `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/control/FlyingMoveControl.java`.
The fixes for these rows were applied in earlier passes; the current source already
matches vanilla, so no source edits were needed this pass. No row is left STILL OPEN.

- Row 2 (WitherDoNothingGoal during spawn): VERIFIED. Vanilla registers the goal at
  priority 0 with MOVE/JUMP/LOOK while Invul > 0 (WitherBoss.java:102, 613-620). Pumpkin
  registers only LOOK goals (LookAtEntityGoal, RandomLookAroundGoal) and disables LOOK in
  `mob_tick` while invul > 0 (wither.rs:649-661); goal_selector.rs:234-238 blocks new goals
  using a disabled control and 334-339 stops running ones. The hand-written movement tick
  still runs during invul, as vanilla `aiStep` does (WitherBoss.java:164-184). No pure unit
  test seam: needs a mob/world fixture.
- Row 3 (gravity / floating): VERIFIED (NOT A GAP). The constructor sets only
  `FlyingMoveControl(this, 10, false)` with no `setNoGravity` (WitherBoss.java:87).
  `FlyingMoveControl.tick` calls `setNoGravity(false)` in WAIT because hoversInPlace is
  false (FlyingMoveControl.java:48-51), so gravity stays on by default; adding
  `set_has_no_gravity(true)` would be wrong. A full flying-navigator/move-control port
  is a separate cross-cutting AI item, not this row.
- Row 5 (bossbar progress): VERIFIED. Spawn phase uses `1 - newCount/220`
  (WitherBoss.java:273; wither.rs:439-442); post-spawn uses `health/maxHealth`
  (WitherBoss.java:391; wither.rs:604-611); `make_invulnerable` sets 0.0
  (WitherBoss.java:401; wither.rs:169-175).
- Row 7 (idle head offsets / head index): VERIFIED. Lines 319-321 use uniform
  `Mth.nextDouble` over x+/-10, y+/-5, z+/-10, not a Gaussian; Pumpkin's
  `random_range(-10.0..10.0)` etc. (wither.rs:485-489) is the same distribution. Vanilla
  fires side heads as `performRangedAttack(i + 1, ...)` (WitherBoss.java:322, 330) with
  i = 1,2 -> heads 2,3; Pumpkin uses `i + 1` (wither.rs:492, 508). `getHeadX/Z` wrap via
  `180 * (index - 1)` (WitherBoss.java:426, 441; wither.rs:188, 208). No off-by-one gap.
- Row 8 (peaceful despawn): VERIFIED. `checkDespawn` discards the wither on Peaceful
  (WitherBoss.java:536-543); `tick_wither_lifecycle` (wither.rs:414-422) clears the bossbar
  and removes the entity on Peaceful, matching the observable shared despawn path.
- Row 9 (powered projectile immunity): FIXED (prior) and VERIFIED. Vanilla `hurtServer`
  blocks only `AbstractArrow` and `WindCharge` via `getDirectEntity()`
  (WitherBoss.java:504-508). `is_projectile_immune_to_powered_wither` (wither.rs:65-69)
  uses the `minecraft:arrows` entity tag (arrow, spectral_arrow) plus TRIDENT and
  WIND_CHARGE and correctly excludes BREEZE_WIND_CHARGE; `pre_damage` receives the direct
  projectile as `source` (arrow.rs:792-800). The tag contents are confirmed in
  `crates/pumpkin-data/src/generated/tag.rs:20953-20957`. The unit test at wither.rs:741-758
  covers the type set.

Files touched this pass: docs/tracker/tfin-tentities-wither-0.md only (no source edits).
No build run (task rules); compile verification left to the orchestrator.

## Result
- Wither rows 1, 2, 3, 5, 7, 8, 9: VERIFIED per `docs/tracker/tentities-wither.md` "Result (x_14 re-verification)" with vanilla and Rust citations. No source edits.

## Result (CODER pass, no cargo)
- Wither rows 2, 3, 5, 7, 8, 9: VERIFIED, no source edit. Re-checked against the vendored `WitherBoss.java` / `FlyingMoveControl.java` citations above.
- Status: fixed=0 verified=6 blocked=0.
