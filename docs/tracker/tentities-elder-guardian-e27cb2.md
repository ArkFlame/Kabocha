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

## Gap status (see ## Result below for the current outcome)
- 1 beam attack, 2 move control/travel, 3 restriction goal, 4 effect gating: VERIFIED - already implemented in guardian.rs / elder_guardian.rs and match decompiled vanilla.
- 5 flop sound: LEFT OPEN - client-side only; no server action, trait change out of scope.
- 6 fluid push: VERIFIED - vanilla has no override; Pumpkin default true matches.

## Result (2026-11-18, decompiled vanilla at .vendor/vanilla-decompiled)

All actionable rows are already implemented in the two allowed source files and match vanilla
`net.minecraft.world.entity.monster.Guardian` / `ElderGuardian` (1.21.x). Verified with direct reads
of `Guardian.java`, `ElderGuardian.java` and `MobEffectUtil.java`; no source changes were required.

- Row 1 - beam attack: VERIFIED (already implemented). `GuardianAttackGoal` in `guardian.rs`
  registers at priority 3 (vanilla 4), charges `attack_time` from -10, syncs `DATA_ID_ATTACK_TARGET`
  at tick 0, broadcasts `GUARDIAN_ATTACK_SOUND` when not silent, stops on line-of-sight loss, and at
  `getAttackDuration()` (80 guardian / 60 elder) deals `indirectMagic(this, this)` with the
  1.0 +2 Hard +2 elder damage and calls `do_hurt_target` for the melee swing. `stop()` clears the
  synced target and the mob target. Matches `Guardian.GuardianAttackGoal` lines 347-429 and
  `ElderGuardian.getAttackDuration()`.

- Row 2 - move control / travel: VERIFIED (already implemented). `GuardianMoveControl` in
  `guardian.rs` lerps speed 0.125 toward `speedModifier * MOVEMENT_SPEED` with a 90 degree yaw
  cap (`MoveControl::MAX_TURN == 90.0`), applies the vanilla `sin(t*0.5)*0.05` / `sin(t*0.75)*0.05`
  wave push and lerps the look target 0.125 with the no-target snap. `travel_in_water` in both
  files uses 0.1 speed, 0.9 drag and -0.005 idle sink with no target. Matches `Guardian.java`
  lines 338-345 and 444-489. No water-only gate is needed because Pumpkin only calls this while
  in water, matching vanilla's `travelInWater` dispatch.

- Row 3 - restriction goal: VERIFIED (already implemented). Both `new()` functions register
  `MoveTowardsRestrictionGoal::new(1.0)` at priority 5 and `custom_server_ai_step` sets home
  to `block_pos` radius 16 when absent. Vanilla registers `MoveTowardsRestrictionGoal(this, 1.0)`
  at priority 5 (`Guardian.java:74,77`) and `ElderGuardian.java:76-78` sets the same home.

- Row 4 - effect application gating: VERIFIED (already implemented). `should_apply_fatigue` in
  `elder_guardian.rs` skips a player with amplifier >= 2 and duration >= 1200; the game event is
  sent only to players that pass the gate, and creative/spectator players are excluded. Matches
  `MobEffectUtil.addEffectToPlayersAround(..., 1200)` (`endsWithin(1199)` == `duration <= 1199`,
  i.e. `duration < 1200`) and `ElderGuardian.java:67-74`. The event amplitude is
  `isSilent() ? 0.0 : 1.0`, matching `ClientboundGameEventPacket` construction in vanilla; the
  earlier note claiming a random `[0.75, 1.0)` amplitude was wrong. Existing unit test
  `fatigue_application_matches_vanilla_display_limit` covers the 1199/1200 boundary.

- Row 5 - flop sound: LEFT OPEN (no server-side fix). Vanilla `Guardian.aiStep` plays
  `getFlopSound()` only inside `level().isClientSide()` (local sound from client animation state),
  so a dedicated Pumpkin server sends nothing; the vanilla client plays the sound itself. Adding a
  `CustomSound` flop hook would require a trait change touching every Mob impl (outside this unit)
  for no observable server behaviour.

- Row 6 - fluid push: VERIFIED (no change needed). Neither `Guardian.java` nor `ElderGuardian.java`
  overrides `isPushedByFluid`, so vanilla uses the default `true`; Pumpkin's
  `mob_is_pushed_by_fluids()` default is also `true`. Do not add a `false` override.

Existing pure unit tests already cover the seams (beam damage, charge duration, thorns tag,
fatigue boundary, move-control look lerp), so no new test was added.
