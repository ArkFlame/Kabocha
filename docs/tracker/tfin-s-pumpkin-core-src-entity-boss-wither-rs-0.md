# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-entity-boss-wither-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- **own/tentities-wither**: VERIFIED. The referenced audit (docs/tracker/tentities-wither.md) has all nine rows closed; no STILL OPEN rows remain there. Its stale claim in "Fixed in this pass" that the aim uses `getY(0.5)` (half bounding-box height) is superseded by this pass's fix against WitherBoss.java:460.

## Result (FINAL PASS, 2026-11-18)

All rows closed. Independent re-verification against
`.vendor/vanilla-decompiled/net/minecraft/world/entity/boss/wither/WitherBoss.java`,
`.vendor/vanilla-decompiled/net/minecraft/world/entity/projectile/hurtingprojectile/WitherSkull.java`
and `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/control/FlyingMoveControl.java`.
No source edits were needed this pass: `crates/pumpkin-core/src/entity/boss/wither.rs`
and `crates/pumpkin-core/src/entity/projectile/wither_skull.rs` already match vanilla.

- **own/tentities-wither** (this unit's only row): VERIFIED. The referenced audit
  (`docs/tracker/tentities-wither.md`) has all nine rows closed (FIXED or VERIFIED);
  no STILL OPEN row remains. Each is re-checked below against the decompiled class.
- **entities/Wither** (parent `tpar-...wither-rs.md`): FIXED (head-shot aim height) and
  VERIFIED. Vanilla `performRangedAttack(int, LivingEntity)` aims at
  `target.getY() + target.getEyeHeight() * 0.5` (WitherBoss.java:460). Both the main head
  (wither.rs:395) and targeted side heads (wither.rs:506) use `get_eye_height() * 0.5`,
  not half the bounding-box height. The 0.001 dangerous chance (WitherBoss.java:460;
  wither.rs:392) matches.

Deep-audit rows (from `tentities-wither.md`), all closed against WitherBoss.java:

- Row 1 (lift clause): VERIFIED. WitherBoss.java:170 keeps
  `!isPowered() && getY() < entity.getY() + 5.0`; wither.rs:354 matches exactly.
- Row 2 (WitherDoNothingGoal during spawn): VERIFIED. WitherBoss.java:102 and 613-620
  register the priority-0 MOVE/JUMP/LOOK goal while Invul > 0. Pumpkin gates its LOOK
  goals via `disable_control(Controls::LOOK)` (wither.rs:653-656); ranged attacks run
  only in the invul==0 branch. The hand-written movement tick still runs while invul > 0,
  as vanilla `aiStep` does (lines 164-184). No unit test: needs a mob/world fixture.
- Row 3 (gravity / floating): VERIFIED (NOT A GAP). WitherBoss.java:87 sets only
  `FlyingMoveControl(this, 10, false)` with no `setNoGravity`; `FlyingMoveControl.tick`
  clears no-gravity in WAIT because `hoversInPlace` is false, so adding
  `set_has_no_gravity(true)` would be wrong. A full flying-navigator port is a separate
  cross-cutting item.
- Row 4 (main-head line of sight): FIXED (prior 7bdd7bd9a) and VERIFIED. wither.rs:383
  gates the main-head shot on `has_line_of_sight`, matching vanilla `RangedAttackGoal`.
- Row 5 (bossbar progress): VERIFIED. Spawn uses `1 - newCount/220` (WitherBoss.java:273;
  wither.rs:441-442); post-spawn uses `health/maxHealth` (WitherBoss.java:391;
  wither.rs:604-611); `make_invulnerable` sets 0.0 (WitherBoss.java:401; wither.rs:171).
- Row 6 (block destruction drops): FIXED (prior) and VERIFIED. wither.rs:582 uses
  `break_block(&bpos, None, BlockFlags::NOTIFY_ALL)` with no SKIP_DROPS, matching
  `level.destroyBlock(blockPos, true, this)` (WitherBoss.java:377).
- Row 7 (idle head offsets / head index): FIXED (prior) and VERIFIED. Uniform offsets
  via `Mth.nextDouble` (WitherBoss.java:319-321) match `random_range(-10.0..10.0)` etc.
  (wither.rs:486-488); side heads use `i + 1` (WitherBoss.java:322, 330; wither.rs:492,
  508); `getHeadX/Z` wrap via `180 * (index - 1)` (WitherBoss.java:426, 441).
- Row 8 (peaceful despawn): VERIFIED. WitherBoss.java:536-543 discards the wither on
  Peaceful; `tick_wither_lifecycle` (wither.rs:414-422) clears the bossbar and removes
  the entity, matching the observable shared despawn path.
- Row 9 (powered projectile immunity): FIXED (prior) and VERIFIED. WitherBoss.java:504-508
  blocks only `AbstractArrow` and `WindCharge`. `is_projectile_immune_to_powered_wither`
  (wither.rs:65-69) uses the arrows tag + TRIDENT + WIND_CHARGE and correctly excludes
  BREEZE_WIND_CHARGE; the unit test (wither.rs:738-756) covers the type set.

Wither-skull correction (WitherSkull.java:78-88): VERIFIED. Wither II (amplifier 1) for
10 s on Normal and 40 s on Hard, no effect on Easy/Peaceful; wither_skull.rs:169-181
matches. The conditional explosion interaction (dangerous => MOB, plain => NONE) is a
known deferred item needing a per-block explosion-resistance model, not a row here.

No BLOCKED-DATA: no generated pumpkin-data field or missing subsystem is required.
Files touched this pass: this doc only. No build run (task rules); rustfmt/compile left
to the orchestrator.
