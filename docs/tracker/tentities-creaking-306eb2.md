# entities/Creaking - re-review (row tentities-creaking-306eb2)

Source: `crates/pumpkin-core/src/entity/mob/creaking.rs` (+ `crates/pumpkin-core/src/block/entities/creaking_heart.rs`,
`crates/pumpkin-core/src/block/blocks/creaking_heart.rs`). No decompiled sources in this worktree (`.vendor/` absent),
so the comparison below is against the vanilla `Creaking` / `CreakingHeartBlockEntity` classes as known from 1.21.4.
Line numbers are post-fix.

## Fixed in this pass

- `creaking.rs:262-286` (`is_looking_at_me`) - was sampling three body points (eye, feet + 0.5, midpoint) with a
  `0.5 / d` epsilon, 20x more lenient than vanilla. Vanilla `Creaking#isLookingAtMe` samples only the creaking eye
  (`this.getEyeY()`) and accepts `dot > 1.0 - 0.025 / d`; both are now matched (`GAZE_EPSILON`).
- `creaking.rs:397-422` (NBT) - `home_pos` was written as a `{x, y, z}` compound. Vanilla
  `Creaking#addAdditionalSaveData` writes it with `NbtUtils.writeBlockPos`, i.e. an int array `[x, y, z]`. Now written
  as `IntArray`; the old compound form is still read for backwards compatibility.

## Remaining gaps

| file:line | vanilla | suggested fix | size |
|---|---|---|---|
| creaking.rs:52,69,446-455 (`attack_animation_remaining_ticks`) | `Creaking#doHurtTarget` sets `ATTACK_ANIMATION_DURATION` (15) and the damage lands when the animation ends | no code assigns the counter (and `ATTACK_ANIMATION_DURATION` is unused), so melee is instantaneous; add an attack override that sets it and gates the hit on expiry | M |
| creaking.rs:240 (`player_is_stuck_in_you`) | `Creaking#checkCanMove` keeps a `playerStuckCounter`; a player eye inside the creaking box increments it and above `MAX_PLAYER_STUCK_COUNTER` (4) the creaking may move | the helper exists but nothing calls it; wire it into `check_can_move` with the nearby player list and return `true` when the counter overflows | M |
| creaking.rs:119-120 | `CreakingMeleeAttackGoal` / `CreakingWanderAroundGoal` cancel while `!canMove()` and idle at `SPEED_MULTIPLIER_WHEN_IDLING` (0.3) | Rust uses plain `MeleeAttackGoal::new(1.0, true)` and `WanderAroundGoal::new(1.0)`, so a frozen creaking still paths at full speed; wrap both goals with a `can_move()` gate and the 0.3 speed | M |
| creaking.rs:119 | `CreakingMeleeAttackGoal` passes `followingTargetEvenIfNotSeen = false` (LOS required) | Pumpkin's flag is `pause_when_mob_idle`, used as `pause_when_mob_idle \|\| has_line_of_sight` at `melee_attack.rs:239`; confirm the mapping against upstream before flipping to `false` | S |
| creaking_heart.rs:83 (`set_creaking_uuid`) | the heart stores the UUID of the creaking it spawned (`CreakingHeartBlockEntity#createCreaking`) and `isProtector` only accepts that one | never called: nothing spawns/binds a creaking, so a UUID-less heart protects any creaking whose `home_pos` matches; bind on heart spawn and clear on teardown (also needs the heart `tick` that spawns the creaking) | M |
| creaking.rs:54-55,71-72,225-238 | vanilla drives the eye flicker client-side from the active/death state | `eyes_glowing` / `next_flicker_time` / `check_eye_blink` are dead code: wire the synced flag or drop them | S |
| creaking.rs:38-39 | `CREAKING_ORANGE` / `CREAKING_GRAY` are the vanilla debug render colours | unused constants; drop them | S |
| creaking.rs:40,525 | `INVULNERABILITY_ANIMATION_DURATION` (8) | `pre_damage` stores the literal `8`; use the constant | S |
| creaking.rs (loot) | a heart-bound creaking drops no loot or XP; only the unbound/tear-down path yields | verify the `CREAKING` loot-table wiring drops nothing while heart-bound | S |
