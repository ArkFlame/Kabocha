# entities / Wither — parity audit (re-review)

Source: `crates/pumpkin-core/src/entity/boss/wither.rs`, `crates/pumpkin-core/src/entity/projectile/wither_skull.rs`.
Vanilla reference class: `net.minecraft.world.entity.boss.wither.WitherBoss` (+ `WitherSkull`, `WitherSkullBlock`).

## Fixed in this pass
- `wither_skull.rs:167` wither effect duration: Easy and Normal are both 10 s, only Hard is 40 s (previously Easy got no effect).
- `wither_skull.rs:179` wither effect amplifier was 1; vanilla applies amplifier 0 (`Wither I`).
- `wither_skull.rs:193` explosion interaction is `MOB` only for dangerous skulls; plain skulls use `NONE` (no block griefing).
- `wither.rs:378` / `wither.rs:489` head shots aim at `target.getY(0.5)` (half the bounding-box height), not `eye_height * 0.5`.

## Remaining gaps
| # | Location | Vanilla | Suggested fix | Size |
|---|---|---|---|---|
| 1 | `wither.rs:336-360` `tick_wither_movement` | `WitherBoss.aiStep` lifts only while the target is above (`target.getY() > this.getY()`); there is no `!isPowered() && y < target.y + 5.0` clause | drop the extra +5 clause after checking the decompiled `aiStep` | S |
| 2 | `wither.rs:102-110` goals | `WitherBoss` registers `WitherDoNothingGoal` at priority 0 with MOVE/JUMP/LOOK flags while `getInvulnerableTicks() > 0` | add an equivalent goal / guard so no goal runs during the spawn animation | S |
| 3 | `wither.rs:336` | `WitherBoss` floats (flying move control, no gravity) | verify whether the wither needs `set_has_no_gravity(true)`; the hand-written movement tick never disables gravity | M |
| 4 | `wither.rs:363-398` main head attack | `RangedAttackGoal(mob, 1.0, 40, 20.0)` requires line of sight (`getSensing().hasLineOfSight`) and only fires while the mob is not already moving to a new target | add the LOS check via `has_line_of_sight` like the side heads do | S |
| 5 | `wither.rs:425-452` bossbar progress | vanilla `BossEvent` progress tracks `health / maxHealth`; the spawn animation fills the bar through `heal(10)` every 10 ticks | derive `bossbar_progress` from health instead of the linear `1 - ticks/220` formula | S |
| 6 (FIXED UNVERIFIED: `break_block` without SKIP_DROPS) | `wither.rs:552-571` block destruction | `Level.destroyBlock(pos, true, this)` drops the broken block's item | use `world.break_block(&bpos, None, BlockFlags::NOTIFY_ALL)` (no SKIP_DROPS) instead of `set_block_state(AIR)` | M |
| 7 | `wither.rs:464-478` idle head shots | side-head random offsets are `nextGaussian() * 10`, `* 5`, `* 10` | use a Gaussian sample (sum of uniforms) instead of `random_range(-10..10)` | S |
| 8 | `wither.rs:400-408` peaceful handling | vanilla despawns through `shouldDespawnInPeaceful`, keeping death/despawn bookkeeping | prefer the shared despawn path over a bare `entity.remove()` | S |
| 9 | `wither.rs:641-666` `pre_damage` projectile immunity | powered wither immunity is checked against `DamageTypeTags.IS_PROJECTILE` semantics | confirm the exact vanilla projectile set (arrows, spectral arrows, wind charges) against decompiled `hurt` | S |

## h6 pass 2 status
- Row 1 (lift clause +5 when unpowered): NOT A GAP. Vanilla WitherBoss.aiStep keeps the unpowered "y < target.y + 5" hover clause (UNVERIFIED against decompiled source, from memory).
- Row 4 (main head needs line of sight): FIXED 7bdd7bd9a (wither.rs tick_main_head_attack; unseen target holds cooldown at zero). No unit test: needs a world fixture.
- Row 2, 3, 5, 7, 8, 9: STILL OPEN, not attempted this pass (row 5 needs bossbar progress derivation, row 7 needs Gaussian source confirmation).

## Result

Verified against `.vendor/vanilla-decompiled/net/minecraft/world/entity/boss/wither/WitherBoss.java`.

- Row 1 (lift clause): NOT A GAP. `aiStep` line 170 keeps `!isPowered() && getY() < entity.getY() + 5.0`; `tick_wither_movement` matches.
- Row 2 (goals during spawn): FIXED. Vanilla registers `WitherDoNothingGoal` at priority 0 with MOVE/JUMP/LOOK while `Invul > 0`. The wither only registers LOOK goals, so `mob_tick` disabling the LOOK control while `invulnerable_ticks > 0` reproduces the guard and releases it after the animation. No unit test: needs a mob/world fixture.
- Row 3 (floating / no gravity): NOT A GAP. The constructor only sets `FlyingMoveControl(this, 10, false)`; `FlyingMoveControl.tick` calls `setNoGravity(false)` in WAIT because `hoversInPlace` is false, so gravity stays on. `set_has_no_gravity(true)` would be wrong. (A full `Navigator::flying()` + move control port is still missing, left open as an M-sized AI item.)
- Row 4: fixed in 7bdd7bd9a (`has_line_of_sight` gate in `tick_main_head_attack`).
- Row 5 (bossbar progress): NOT A GAP. `customServerAiStep` uses `1 - newCount/220` during the spawn animation (line 273) and `health / maxHealth` afterwards (line 391); `tick_wither` does exactly that, and the spawn fill comes from the existing `heal(10)` every 10 ticks.
- Row 6: fixed (`break_block` with drops, no SKIP_DROPS).
- Row 7 (idle head offsets): NOT A GAP. Lines 319-321 use `Mth.nextDouble(random, x-10, x+10)`, a uniform range, not a Gaussian; `random_range(-10.0..10.0)` is correct.
- Row 8 (peaceful): NOT A GAP. `MobEntity::tick` runs the shared `check_despawn` before `mob_tick`, and the wither is a `MONSTER`, so vanilla `checkDespawn` discards it through the shared path; `tick_wither_lifecycle` also clears the bossbar and removes the entity.
- Row 9 (powered projectile immunity): FIXED. Vanilla `hurtServer` blocks `AbstractArrow` (arrow, spectral arrow, trident; tipped arrows reuse the ARROW type) and `WindCharge`. `BreezeWindCharge` extends `AbstractWindCharge`, not `WindCharge`, so a breeze charge still damages a powered wither. Replaced the hand-written type list with `is_projectile_immune_to_powered_wither` (the `minecraft:arrows` tag plus `TRIDENT` and `WIND_CHARGE`), dropped `BREEZE_WIND_CHARGE`, and added a unit test over the type set.

Files touched: `crates/pumpkin-core/src/entity/boss/wither.rs`. `rustfmt --edition 2024` passed. No build run (task rules); compile verification left to the orchestrator.

## Result (re-review 2026-11-18)

Independent verification against `.vendor/vanilla-decompiled/net/minecraft/world/entity/boss/wither/WitherBoss.java` for the STILL OPEN rows (2, 3, 5, 7, 8, 9). Source already matches vanilla; no source edits were needed this pass.

- Row 2 (WitherDoNothingGoal): MATCH. Vanilla registers the goal at priority 0 with MOVE/JUMP/LOOK while Invul > 0 (lines 102, 613-620). Pumpkin gates its only LOOK-flagged goals via `goals_selector.disable_control(Controls::LOOK)` in `mob_tick` (wither.rs:647-659); the ranged attack runs only in the invul==0 branch (wither.rs:469). `disable_control` sets an idempotent bitflag (goal_selector.rs:419-431) and LookAtEntityGoal declares LOOK control (look_at_entity.rs:88). No test: needs a mob/world fixture.
- Row 3 (gravity): NOT A GAP. Constructor only sets `FlyingMoveControl(this, 10, false)` (line 87); no `setNoGravity`, so `set_has_no_gravity(true)` would be wrong. A full flying-navigator/move-control port remains an out-of-scope cross-cutting AI item.
- Row 5 (bossbar progress): MATCH. Spawn phase uses `1 - newCount/220` (line 273; wither.rs:441-442); post-spawn uses `health/maxHealth` (line 391; wither.rs:604-609); `make_invulnerable` sets 0.0 (line 401; wither.rs:171).
- Row 7 (idle head offsets): NOT A GAP. Lines 319-321 use `Mth.nextDouble` uniform `x +/-10`, `y +/-5`, `z +/-10`; `random_range(-10.0..10.0)` (wither.rs:486-488) is correct.
- Row 8 (peaceful): MATCH. `tick_wither_lifecycle` (wither.rs:414-422) clears the bossbar and removes the entity on Peaceful, matching the observable shared despawn path.
- Row 9 (projectile immunity): MATCH. `hurtServer` lines 504-508 block `AbstractArrow`/`WindCharge`. `is_projectile_immune_to_powered_wither` (wither.rs:65-69) uses the arrows tag plus TRIDENT and WIND_CHARGE; BREEZE_WIND_CHARGE is correctly not immune. The existing unit test (wither.rs:739-756) covers the type set.

Files touched this pass: `docs/tracker/tentities-wither.md` only (no source edits). No build run (task rules); compile verification left to the orchestrator.


## Result (re-review pass 3, 2026-11-18)

Independent re-verification of the STILL OPEN rows (2, 3, 5, 7, 8, 9) against `.vendor/vanilla-decompiled/net/minecraft/world/entity/boss/wither/WitherBoss.java` and `.../ai/control/FlyingMoveControl.java`. Every row already matches vanilla; no source edits were needed this pass.

- Row 2 (WitherDoNothingGoal): MATCH. Vanilla registers the goal at priority 0 with MOVE/JUMP/LOOK while Invul > 0 (lines 102, 613-620), but its own hand-written `aiStep` movement still runs regardless of invul (lines 164-184). Pumpkin gates LOOK via `disable_control(Controls::LOOK)` while invul > 0 and runs `tick_wither_movement` unconditionally (wither.rs:651-662). The goal's MOVE/JUMP flags only suppress goal-driven navigation; Pumpkin has none, so the guard is reproduced.
- Row 3 (gravity): NOT A GAP. The constructor sets `FlyingMoveControl(this, 10, false)` (line 87) with no `setNoGravity`. `FlyingMoveControl.tick` calls `setNoGravity(false)` in WAIT when `hoversInPlace` is false (FlyingMoveControl.java:48-51), so gravity stays on; adding `set_has_no_gravity(true)` would be wrong.
- Row 5 (bossbar progress): MATCH. Spawn phase uses `1 - newCount/220` (line 273; wither.rs:441-442); post-spawn uses `health/maxHealth` (line 391; wither.rs:602-609).
- Row 7 (idle head offsets): NOT A GAP. Lines 319-321 use uniform `Mth.nextDouble` over x+/-10, y+/-5, z+/-10; Pumpkin's `random_range(-10.0..10.0)` etc. (wither.rs:485-489) is the same distribution. Vanilla fires side shots as `performRangedAttack(i+1, ...)` (lines 322, 330) while Pumpkin uses head `i`; these are geometrically identical because `getHeadX/Z` use `180*(index-1)` (lines 421-428), so vanilla head 3 wraps to head 1. No off-by-one gap.
- Row 8 (peaceful): MATCH. `tick_wither_lifecycle` (wither.rs:414-422) clears the bossbar and removes the entity on Peaceful, matching the observable shared despawn path.
- Row 9 (projectile immunity): MATCH. `hurtServer` lines 504-508 block only `AbstractArrow` and `WindCharge`. `is_projectile_immune_to_powered_wither` (wither.rs:65-69) uses the arrows tag plus TRIDENT and WIND_CHARGE and correctly excludes BREEZE_WIND_CHARGE; the unit test (wither.rs:739-756) covers the type set.

Files touched: `docs/tracker/tentities-wither.md` only (no source edits). No build run (task rules); compile verification left to the orchestrator.


## Result (re-review pass 3 final, 2026-11-18)

Independent verification of every remaining STILL OPEN row (2, 3, 5, 7, 8, 9) against
`.vendor/vanilla-decompiled/net/minecraft/world/entity/boss/wither/WitherBoss.java` and
`.../ai/control/FlyingMoveControl.java`. All rows match vanilla in the current source;
no source edits were needed this pass. Each row is now closed with a verdict.

- Row 2 (WitherDoNothingGoal during spawn): VERIFIED. Vanilla registers the goal at
  priority 0 with MOVE/JUMP/LOOK while Invul > 0 (WitherBoss.java lines 102, 613-620).
  Pumpkin gates its only LOOK goals via `disable_control(Controls::LOOK)` in `mob_tick`
  while invul > 0 (wither.rs:651-659); `disable_control` sets an idempotent bitflag
  (goal_selector.rs:419-431) and LookAtEntityGoal declares LOOK (look_at_entity.rs:88).
  The hand-written movement tick still runs while invul > 0, as vanilla aiStep does
  (lines 164-184). No unit test: needs a mob/world fixture.
- Row 3 (gravity / floating): VERIFIED (NOT A GAP). The constructor sets only
  `FlyingMoveControl(this, 10, false)` (WitherBoss.java:87) with no `setNoGravity`.
  `FlyingMoveControl.tick` calls `setNoGravity(false)` in WAIT when hoversInPlace is
  false, so gravity stays on; adding set_has_no_gravity(true) would be wrong.
- Row 5 (bossbar progress): VERIFIED. Spawn phase uses `1 - newCount/220`
  (WitherBoss.java:273; wither.rs:441-442); post-spawn uses `health/maxHealth`
  (WitherBoss.java:391; wither.rs:602-609); make_invulnerable sets 0.0 (wither.rs:171).
- Row 7 (idle head offsets): VERIFIED (NOT A GAP). Lines 319-321 use uniform
  `Mth.nextDouble` over x+/-10, y+/-5, z+/-10, matching Pumpkin's
  `random_range(-10.0..10.0)` etc. (wither.rs:486-488). Vanilla fires side shots with
  head i+1 (lines 322, 330) but getHeadX/Z wrap geometrically, so there is no
  off-by-one gap.
- Row 8 (peaceful despawn): VERIFIED. `tick_wither_lifecycle` (wither.rs:414-422)
  clears the bossbar and removes the entity on Peaceful, matching the observable
  shared despawn path.
- Row 9 (powered projectile immunity): FIXED and VERIFIED. Vanilla `hurtServer` lines
  504-508 block only AbstractArrow and WindCharge. The prior pass replaced the
  hand-written type list with `is_projectile_immune_to_powered_wither` (wither.rs:65-69),
  using the arrows tag plus TRIDENT and WIND_CHARGE and correctly excluding
  BREEZE_WIND_CHARGE; the unit test (wither.rs:736-756) covers the type set.

Files touched this pass: docs/tracker/tentities-wither.md only (no source edits).
No build run (task rules); compile verification left to the orchestrator.


## Result (final verification pass, 2026-11-18)

Independent verification of every remaining gap row (1-9) against
.vendor/vanilla-decompiled/net/minecraft/world/entity/boss/wither/WitherBoss.java
and .vendor/vanilla-decompiled/net/minecraft/world/entity/ai/control/FlyingMoveControl.java.
All rows are now closed; no row is left STILL OPEN. One source fix was applied this pass (row 7).

- Row 1 (lift clause): VERIFIED (NOT A GAP). WitherBoss.java:170 keeps
  "!isPowered() && getY() < entity.getY() + 5.0"; wither.rs:354 matches exactly.
- Row 2 (WitherDoNothingGoal during spawn): VERIFIED. WitherBoss.java:102 and 613-620
  register the goal at priority 0 with MOVE/JUMP/LOOK while Invul > 0. Pumpkin gates its
  only LOOK goals via disable_control(Controls::LOOK) in mob_tick (wither.rs:651-659);
  goal_selector.rs:234-238 excludes any goal using a disabled control from starting or
  continuing, and look_at_entity.rs:88 declares LOOK. The ranged attack runs only in the
  invul==0 branch. The hand-written movement tick still runs while invul > 0, as vanilla
  aiStep does (WitherBoss.java:164-184). No unit test: needs a mob/world fixture.
- Row 3 (gravity / floating): VERIFIED (NOT A GAP). WitherBoss.java:87 constructor sets
  only FlyingMoveControl(this, 10, false) with no setNoGravity. FlyingMoveControl.java
  calls setNoGravity(false) in WAIT because hoversInPlace is false, so gravity stays on;
  adding set_has_no_gravity(true) would be wrong. A full flying-navigator port is a
  separate cross-cutting AI item, not this row.
- Row 4 (main head line of sight): FIXED (prior 7bdd7bd9a). wither.rs:383 gates the main
  head attack on has_line_of_sight, matching vanilla RangedAttackGoal.
- Row 5 (bossbar progress): VERIFIED. Spawn phase uses 1 - newCount/220
  (WitherBoss.java:273; wither.rs:441-442); post-spawn uses health/maxHealth
  (WitherBoss.java:391; wither.rs:602-609); make_invulnerable sets 0.0
  (WitherBoss.java:401; wither.rs:171).
- Row 6 (block destruction drops): FIXED (prior). wither.rs:580 uses
  break_block(..., BlockFlags::NOTIFY_ALL) with no SKIP_DROPS, matching vanilla
  Level.destroyBlock(pos, true, this) at WitherBoss.java:377.
- Row 7 (idle head offsets and head index): FIXED this pass. Offsets are uniform
  (WitherBoss.java:319-321 uses Mth.nextDouble over x+/-10, y+/-5, z+/-10), matching
  random_range(-10.0..10.0) at wither.rs:486-488. Corrected the side-head index:
  WitherBoss.java:322 and 330 call performRangedAttack(i + 1, ...) for two heads 2 and 3,
  but Pumpkin passed head i; changed wither.rs:492 and 507 to i + 1. Head 3 wraps to the
  head-1 angle via 180 * (index - 1) (WitherBoss.java:421-428), so the emitted head now
  matches vanilla. No pure unit-test seam: get_head_x/y/z require a world/entity fixture.
- Row 8 (peaceful despawn): VERIFIED. WitherBoss.java:536-543 checkDespawn discards the
  wither on Peaceful; wither.rs:414-422 tick_wither_lifecycle clears the bossbar and
  removes the entity, matching the observable shared despawn path.
- Row 9 (projectile immunity): FIXED (prior) and VERIFIED. WitherBoss.java:504-508 blocks
  only AbstractArrow and WindCharge. is_projectile_immune_to_powered_wither (wither.rs:65-69)
  uses the arrows tag plus TRIDENT and WIND_CHARGE and correctly excludes
  BREEZE_WIND_CHARGE; the unit test (wither.rs:739-757) covers the type set.

Files touched this pass: crates/pumpkin-core/src/entity/boss/wither.rs and this doc.
No build run (task rules); compile verification left to the orchestrator.
