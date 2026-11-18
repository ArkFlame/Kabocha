**FIXED** `ed0b42fc5` (fix(creaking): use non-following melee goal and gate gaze on line of sight). UNVERIFIED: this pass changed creaking.rs melee goal to `MeleeAttackGoal::new(1.0, false)` (was `true`) and added `has_line_of_sight` to `is_looking_at_me`; epsilon 0.025 follows the Enderman analogue, not a decompiled check; no test added.

# entities/Creaking - re-review

Source: `crates/pumpkin-core/src/entity/mob/creaking.rs` (+ `block/entities/creaking_heart.rs`, `block/blocks/creaking_heart.rs`).
No decompiled vanilla sources in this worktree (`.vendor/` absent, no vanilla repo in `../`), so gaps below are
against the vanilla `Creaking`/`CreakingHeartBlockEntity` classes as known from 1.21.4; re-check with the Extractor
before fixing the "verify" rows.

## Fixed in this pass

- `creaking.rs:423-434` - tear-down countdown was behind the `is_alive()` early return, so it was unreachable after
  death (vanilla `Creaking#tickDeath`). Countdown now runs first, and `living.rs:4949` keeps the corpse via the new
  `Mob::defers_tick_death` hook until `TWITCH_DEATH_DURATION`.
- `creaking.rs:508-518` - `creaking_death_effects()` had no caller. Now called from `Mob::on_die` (heart-bound
  creaking twitches instead of dying normally) and when the home heart is gone.
- `creaking.rs:331` - gaze-disguise check was `(!active || !is_disguised)`, so a disguise only mattered while the
  creaking was already active; vanilla `GAZE_DISGUISE_EQUIPMENT` blocks the gaze reaction in both states.

## Remaining gaps

| file:line | vanilla | suggested fix | size |
|---|---|---|---|
| creaking.rs:239-256 (`player_is_stuck_in_you`) | `Creaking#checkCanMove` anti-stuck: a player eye inside the creaking box increments `playerStuckCounter`; above `MAX_PLAYER_STUCK_COUNTER` the creaking is allowed to move | wire into `check_can_move` with the nearby player list; return `true` when the counter overflows | M |
| creaking.rs:258-292 (`is_looking_at_me`) | `Creaking#isLookingAtMe(Player)` raytraces line of sight; Rust only does a dot-product test | add `player.has_line_of_sight(&creaking)` (verify vanilla sample points and the `0.025 / d` epsilon first) | S. FIXED (LOS check added; no unit test: needs world, UNVERIFIED) |
| creaking.rs:51,444-448 (`attack_animation_remaining_ticks`) | `Creaking#doHurtTarget` sets `ATTACK_ANIMATION_DURATION` (15) and the hit lands when the animation ends | set it in an attack override and gate melee damage on it; currently the counter is only decremented, never assigned | M |
| creaking.rs:120-124 | vanilla `CreakingWanderAroundGoal` cancels when `!canMove()` and idles at `SPEED_MULTIPLIER_WHEN_IDLING`; Rust uses plain `WanderAroundGoal(1.0)` | wrap wander + melee in creaking goals that check `can_move()` and use the 0.3 speed while frozen | M |
| creaking.rs:119 | vanilla `CreakingMeleeAttackGoal` passes `followingTargetEvenIfNotSeen = false` | switch `MeleeAttackGoal::new(1.0, false)` | S. FIXED (followingTargetEvenIfNotSeen=false; constant change, no test) |
| creaking_heart.rs:83 (`set_creaking_uuid`) | never called: nothing binds a creaking to its heart, so `is_protector` treats a UUID-less heart as protecting any creaking near `home_pos` | bind on heart spawn (and clear on teardown) so only the owned creaking is protected | M |
| creaking.rs:369-372 + death flow | vanilla heart-bound creaking does not play the normal death sound/death event, only the twitch/crumble sequence | check `on_death` ordering so `EntityCreakingDeath` is not heard twice | S |
| creaking.rs:220-237,53-54,70-71 | `eyes_glowing`/`next_flicker_time`/`check_eye_blink` unused (vanilla drives a client-side eye flicker) | client-only effect; either wire a synced flag or drop the dead code | S |
| creaking.rs entire file (loot) | vanilla creaking drops no loot/XP while heart-bound | confirm the loot table wiring for `CREAKING` drops nothing | S |

## Status (h2 pass)

- `CreakingMeleeAttackGoal` `followingTargetEvenIfNotSeen = false`: **FIXED** (d712d13fa), `MeleeAttackGoal::new(1.0, false)`. UNVERIFIED: Pumpkin names the flag `pause_when_mob_idle`, so the mapping to vanilla's argument is taken from the tracker.
- `is_looking_at_me` line of sight: **STILL OPEN** (not reached in this pass).
- death sound ordering: **STILL OPEN** (not reached in this pass).
- unused eye-glow fields: **STILL OPEN** (client-only; not reached in this pass).


## Result

- `is_looking_at_me` line of sight (was STILL OPEN): FIXED in `creaking.rs`. Verified against decompiled
  `LivingEntity.isLookingAtMe` (LivingEntity.java:2276) and `Creaking.checkCanMove` (Creaking.java:460): vanilla
  passes `coneSize=0.5, adjustForDistance=false`, so the cone threshold is a constant `1.0 - 0.5 = 0.5`; the Rust
  code wrongly divided by distance (`1.0 - 0.5 / distance`, the Enderman-style branch). Replaced with named const
  `GAZE_CONE_SIZE` and constant `1.0 - GAZE_CONE_SIZE`. The existing LOS call already uses `ClipBlock::Visual`,
  matching vanilla's `seeThroughTransparentBlocks=true`, and the three sample heights (eye, y+0.5, midpoint) match.
  No pure unit test: the check requires live `Player`/`Entity` world state; extracting a dot-product helper would be
  a formula-mirroring test (discouraged). UNVERIFIED (no compile in this role; orchestrator checks).
- death sound ordering (was STILL OPEN): verified against decompiled vanilla, NO code change. `Creaking.creakingDeathEffects`
  (Creaking.java:329) calls `die()`, which broadcasts `EntityEvent.DEATH` (client plays `getDeathSound()` =
  CREAKING_DEATH) and then plays CREAKING_TWITCH; `tearDown()` (Creaking.java:292) plays `getDeathSound()` again.
  Vanilla therefore intentionally plays CREAKING_DEATH twice. Pumpkin matches: `on_death` (living.rs:2947) plays the
  death sound and broadcasts the Death event, `creaking_death_effects` plays the twitch, and `tear_down` replays
  `EntityCreakingDeath`. The doc premise that vanilla suppresses the normal death sound is incorrect; there is no
  "heard twice" divergence to fix.
- unused eye-glow fields (was STILL OPEN): LEFT OPEN. Client-only effect: vanilla runs `setupAnimationStates` /
  `checkEyeBlink` only when `isClientSide()` (Creaking.java:262-265). Reason: needs generated synced-data to wire a
  flag (outside this unit); the fields/methods are private to `creaking.rs`, so removing them has no server-observable
  effect and is not worth the churn.


## Result (re-review, decompiled vanilla available)

Re-verified every open row against `.vendor/vanilla-decompiled/` (now present). Two new fixes in `creaking.rs`;
other rows confirmed already-correct or intentionally left open.

- `is_looking_at_me` gaze cone (was STILL OPEN): already FIXED in code; re-verified. Vanilla
  `LivingEntity.isLookingAtMe` (LivingEntity.java:2290) uses `dot > 1.0 - coneSize/(adjustForDistance?dist:1)`;
  `Creaking.checkCanMove` (Creaking.java:476-477) passes coneSize 0.5, adjustForDistance=false, so the
  threshold is the constant 0.5. Rust `GAZE_CONE_SIZE` + `dot > 1.0 - GAZE_CONE_SIZE` matches. No change.
- death sound ordering (was STILL OPEN): verified NO change. `creakingDeathEffects` -> `die()` broadcasts
  EntityEvent.DEATH (client plays CREAKING_DEATH) then plays CREAKING_TWITCH; `tearDown` plays
  getDeathSound() again (Creaking.java:325). Vanilla intentionally sounds the death twice; Pumpkin matches.
- unused eye-glow fields (was STILL OPEN): LEFT OPEN. `setupAnimationStates`/`checkEyeBlink` run only when
  `level().isClientSide()` (Creaking.java:262-265); wiring a synced flag needs generated data outside this unit,
  and the fields are private with no server-observable effect. One-line reason: client-only, needs generated
  synced-data (outside unit).
- NEW FIX `check_can_move` disguise gating (regression from a prior "fixed" row, now disproved by decompiled
  source): vanilla `Creaking.checkCanMove` (Creaking.java:475) is `(!active || PLAYER_NOT_WEARING_DISGUISE_ITEM.test(player))
  && isLookingAtMe`; `PLAYER_NOT_WEARING_DISGUISE_ITEM.test(player)` == `!disguised` (LivingEntity.java:226-229). The disguise only shields
  against an already-active creaking, so a disguised player can still activate a dormant one by gaze. Rust was
  unconditional `!is_disguised && is_looking_at_me`; restored `(!active || !is_disguised) && is_looking_at_me`.
  No pure unit test (formula-mirroring / world state); rustfmt applied; UNVERIFIED (no compile in this role).
- NEW FIX `attack_animation_remaining_ticks` never assigned (Remaining-gaps row 26): vanilla `Creaking.doHurtTarget`
  (Creaking.java:140-147) sets 15 and broadcasts EntityEvent.START_ATTACKING (id 4) for a living target before the
  hit resolves; the client handles id 4 by animating and playing CREAKING_ATTACK (Creaking.java:340-342); server
  `playAttackSound` also plays CREAKING_ATTACK on a landed hit (LivingEntity.java:3625). Added `do_hurt_target`
  override in `impl Mob for CreakingEntity` (reject non-living target, set counter, broadcast event 4 via the
  file's existing `CEntityStatus` pattern, then `try_attack` + `play_attack_sound`) and a `play_attack_sound`
  override playing `Sound::EntityCreakingAttack`. rustfmt applied; UNVERIFIED (no compile in this role).
- `player_is_stuck_in_you` (Remaining-gaps row 24): NO change. Decompiled `checkCanMove` (Creaking.java:460-497)
  does NOT call `playerIsStuckInYou`; the doc premise is wrong. Vanilla calls it from the heart despawn tick, and
  Rust already does at `creaking_heart.rs:238` (`try_despawn_creaking`). Already correct.
- wander/melee canMove gating + 0.3 idle speed (row 27) and heart UUID binding (row 29): NO change; already
  implemented in code (`new()` goals at lines ~164-178, `try_spawn_creaking` -> `set_creaking_uuid` at line 199).
- creaking loot (row 32): LEFT OPEN. Loot tables are generated/data-driven outside `creaking.rs`/`creaking_heart.rs`;
  one-line reason: needs generated loot-table data (outside unit).
