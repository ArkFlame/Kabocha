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
| creaking.rs:258-292 (`is_looking_at_me`) | `Creaking#isLookingAtMe(Player)` raytraces line of sight; Rust only does a dot-product test | add `player.has_line_of_sight(&creaking)` (verify vanilla sample points and the `0.025 / d` epsilon first) | S |
| creaking.rs:51,444-448 (`attack_animation_remaining_ticks`) | `Creaking#doHurtTarget` sets `ATTACK_ANIMATION_DURATION` (15) and the hit lands when the animation ends | set it in an attack override and gate melee damage on it; currently the counter is only decremented, never assigned | M |
| creaking.rs:120-124 | vanilla `CreakingWanderAroundGoal` cancels when `!canMove()` and idles at `SPEED_MULTIPLIER_WHEN_IDLING`; Rust uses plain `WanderAroundGoal(1.0)` | wrap wander + melee in creaking goals that check `can_move()` and use the 0.3 speed while frozen | M |
| creaking.rs:119 | vanilla `CreakingMeleeAttackGoal` passes `followingTargetEvenIfNotSeen = false` | switch `MeleeAttackGoal::new(1.0, false)` | S |
| creaking_heart.rs:83 (`set_creaking_uuid`) | never called: nothing binds a creaking to its heart, so `is_protector` treats a UUID-less heart as protecting any creaking near `home_pos` | bind on heart spawn (and clear on teardown) so only the owned creaking is protected | M |
| creaking.rs:369-372 + death flow | vanilla heart-bound creaking does not play the normal death sound/death event, only the twitch/crumble sequence | check `on_death` ordering so `EntityCreakingDeath` is not heard twice | S |
| creaking.rs:220-237,53-54,70-71 | `eyes_glowing`/`next_flicker_time`/`check_eye_blink` unused (vanilla drives a client-side eye flicker) | client-only effect; either wire a synced flag or drop the dead code | S |
| creaking.rs entire file (loot) | vanilla creaking drops no loot/XP while heart-bound | confirm the loot table wiring for `CREAKING` drops nothing | S |
