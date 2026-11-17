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
| 6 | `wither.rs:552-571` block destruction | `Level.destroyBlock(pos, true, this)` drops the broken block's item | use `world.break_block(&bpos, None, BlockFlags::NOTIFY_ALL)` (no SKIP_DROPS) instead of `set_block_state(AIR)` | M |
| 7 | `wither.rs:464-478` idle head shots | side-head random offsets are `nextGaussian() * 10`, `* 5`, `* 10` | use a Gaussian sample (sum of uniforms) instead of `random_range(-10..10)` | S |
| 8 | `wither.rs:400-408` peaceful handling | vanilla despawns through `shouldDespawnInPeaceful`, keeping death/despawn bookkeeping | prefer the shared despawn path over a bare `entity.remove()` | S |
| 9 | `wither.rs:641-666` `pre_damage` projectile immunity | powered wither immunity is checked against `DamageTypeTags.IS_PROJECTILE` semantics | confirm the exact vanilla projectile set (arrows, spectral arrows, wind charges) against decompiled `hurt` | S |
