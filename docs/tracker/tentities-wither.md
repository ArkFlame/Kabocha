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
