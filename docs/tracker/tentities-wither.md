# entities / Wither - parity audit (re-review)

Source: `crates/pumpkin-core/src/entity/boss/wither.rs`, `crates/pumpkin-core/src/entity/projectile/wither_skull.rs`, `crates/pumpkin-core/src/block/blocks/wither_skull.rs`.
Vanilla reference class: `net.minecraft.world.entity.boss.wither.WitherBoss` (+ `WitherSkull`, `WitherSkullBlock`). No decompiled tree under `.vendor/` in this worktree; vanilla behaviour below is from the class source as known (1.21.x).

## Fixed in this pass
- `wither.rs:367-379` main head now requires `has_line_of_sight` before firing, matching `RangedAttackGoal` (commit c67e9abc9).
- `wither.rs:160-168`, `wither.rs:427-455` boss bar progress is now `health / maxHealth` during the 220-tick spawn animation (starts at 1/3, rises with the `heal(10)` ticks) instead of a synthetic `1 - ticks/220` ramp (commit a72a83a95).
- `wither.rs:555-575` block destruction uses `World::break_block` without `SKIP_DROPS`, matching vanilla `Level.destroyBlock(pos, true, this)` which drops the broken block's loot (commit f6dd0d8ea).

## Fixed in the previous pass (still present)
- `wither_skull.rs:171-185` wither effect: Easy and Normal 10 s, Hard 40 s, amplifier 0.
- `wither_skull.rs:198-204` explosion interaction is `MOB` only for dangerous skulls.
- `wither.rs:380-390` head shots aim at `target.getY(0.5)` (half the bounding-box height).

## Remaining gaps
| # | Location | Vanilla | Suggested fix | Size |
|---|---|---|---|---|
| 1 | `wither.rs:102-110` goals | `WitherBoss` registers `WitherDoNothingGoal` at priority 0 with MOVE/JUMP/LOOK flags while `getInvulnerableTicks() > 0` | add an equivalent goal so look/random-look goals do not rotate the wither during the spawn animation | S |
| 2 | `wither.rs:176-203` `get_head_x`/`get_head_z` | head offsets use `yBodyRot`, not `yRot` | read the body yaw instead of `entity.yaw` for the two side heads | S |
| 3 | `wither.rs:557-560` destroy extents | fixed loop `x -1..=1`, `z -1..=1`, `y 0..=3` from the block position; not derived from the bounding box | replace the bounding-box maths with the fixed offsets (equal today, diverges if the box changes) | S |
| 4 | `wither.rs:404-412` peaceful | despawn goes through `shouldDespawnInPeaceful` -> normal despawn path, keeping entity bookkeeping | use the shared despawn path instead of a bare `entity.remove()` (remove_all_bossbar is already done) | S |
| 5 | `wither.rs:367-403` main head timer | `RangedAttackGoal` needs `seeTime >= 20` and resets `attackTime = -1` (restarting the 40-tick countdown) when the target leaves range or LOS | model the goal (see-time gate, reset on loss) instead of decrementing a bare 40-tick counter | M |
| 6 | `wither.rs:653-677` `pre_damage` | vanilla rejects damage whose `source.getEntity()` is a `WitherBoss` and, while powered, projectile damage | resolve the projectile owner/wither source (wither skulls fired by another wither currently hurt it) and check `DamageTypeTags.IS_PROJECTILE` rather than the four concrete projectile types | S |
| 7 | `wither_skull.rs:198-204` plain skulls | `WitherSkull.onHit` explodes with `ExplosionInteraction.MOB` for both skull types; `shouldBlockExplode` only makes dangerous skulls ignore blast resistance of non-`WITHER_IMMUNE` blocks | verify against decompiled `WitherSkull`/`WitherBoss.shouldBlockExplode`; if confirmed, plain skulls must still break blocks | M |
| 8 | `wither.rs:272-320` boss bar membership | `ServerBossEvent` tracks exactly the players tracking the entity (add on start tracking, remove on stop tracking); no distance poll | drive boss bar add/remove from player tracking events instead of the 50-block poll in `update_bossbar` | M |
