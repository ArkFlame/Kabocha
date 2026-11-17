# Elder Guardian - vanilla parity audit (RE-REVIEW, 2026-11-17)

Source: `crates/pumpkin-core/src/entity/mob/elder_guardian.rs` and shared `guardian.rs`.
No decompiled vanilla tree vendored here; references cite `net.minecraft.world.entity.monster.Guardian` / `ElderGuardian` (1.21.x).

## Verified matching
- Attributes (MAX_HEALTH 80, ATTACK_DAMAGE 8, MOVEMENT_SPEED 0.3), experience 10, size/hitbox: data-driven from `pumpkin-data/src/generated/entity_type.rs:3569` (`ELDER_GUARDIAN`).
- `customServerAiStep` order: restrict to `block_pos`/16 when no restriction, then `(tickCount + id) % 1200 == 0` -> mining fatigue amplifier 2, 6000 ticks, 50 block radius + `GUARDIAN_ELDER_EFFECT`. Matches `ElderGuardian#customServerAiStep`.
- Goal list (swim 4 / look-player 5 / random-look 6), no float goal; prey set player/squid/glow squid/axolotl, `> 3 blocks` gate (`guardian.rs:21-40`) matches `Guardian$GuardianAttackSelector`.
- Hurt/death/ambient water-vs-land sound split and 160 tick ambient interval.

## Fixed in this pass
- Thorns reflection (`guardian.rs`, `elder_guardian.rs`): previously used the causing entity, ignored the `minecraft:avoids_guardian_thorns` tag, and did not exclude guardian direct entities; reflected damage was attributed to the attacker. Now `Guardian#hurt` semantics: direct entity (`cause`), living, non-guardian, tag check, source = guardian.
- Elder guardian game event value: constant 1.0 -> random `[0.75, 1.0)` as in vanilla (client shake amplitude).

## Remaining gaps
1. **No guardian beam attack** (L). `Guardian#GuardianAttackGoal` and the synced beam state (`DATA_ID_ATTACK_TARGET`, `clientSideAttackTime`, 80 tick charge, ATTACK_DAMAGE damage, stop on line-of-sight loss) have no counterpart; `elder_guardian.rs` / `guardian.rs` only implement target selection. Main damage source of the mob is missing. Fix: new goal file + synced metadata index.
2. **No `GuardianMoveControl` / guardian `travel`** (M). Vanilla moves only in water, hovers toward `block_pos().above(10)` near the target (retarget 80/160 ticks), scales delta 0.9 and sinks -0.005 y with no target; Pumpkin uses the generic `LivingEntity::travel_in_water` (drag 0.8). Fix: water-only move control shared by both mobs.
3. **Restriction goal** (M, unverified). Home position is set (`elder_guardian.rs:110`), but no `MoveTowardsRestrictionGoal(1.0)` at priority 5 exists, so pathing never steers back to the monument; only `random_pos`/targeting consult the range. Fix: port the goal.
4. **Effect application gating** (S). `elder_guardian.rs:113-131` applies mining fatigue and sends the game event to every player in 50 blocks. Vanilla `MobEffectUtil.addEffectToPlayersAround(..., 1200)` skips players that already have DIG_SLOWDOWN with duration > 1200 and amplifier >= 2, and only those players receive the game event. Fix: check the player's mining fatigue instance first and use the returned subset.
5. **Guardian flop sound** (S). No `CustomSound`/`Mob` hook for `getFlopSound` (`ELDER_GUARDIAN_FLOP`); vanilla plays it when the guardian flops outside water. Fix: add a flop hook (trait change touches every impl) or leave as client audio.
6. **Fluid push** (S, unverified). Vanilla `Guardian#isPushedByFluid` -> confirm against a decompiled source before adding `mob_is_pushed_by_fluids() -> false`.

## Gap status
- 1 guardian beam attack (L): STILL OPEN (size L)
- 3 restriction goal priority 5 (M, unverified): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 6 fluid push (S, unverified): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 2, 4, 5: STILL OPEN (unverified: no .vendor decompile in h6; not changed) (not individually re-checked in this sweep)
