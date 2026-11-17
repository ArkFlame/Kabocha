# Tracker: entities / Enderman (RE-REVIEW)

Audited `crates/pumpkin-core/src/entity/mob/enderman.rs` plus its goals
(`teleport_towards_player.rs`, `chase_player.rs`) against vanilla `Enderman`.
No `.vendor/` decompiled source is present in this worktree, so vanilla facts come from
the 1.21 `Enderman` / `EndermanLookForPlayerGoal` / `EndermanFreezeWhenLookedAt` classes.

## Fixed in this pass

- `enderman.rs:411` `play_stare_sound()` added (vanilla `Enderman.playStareSound`:
  `EntityEndermanStare` at volume 2.5, pitch 1.0, eye height, silenced by `isSilent`,
  400-tick per-entity cooldown), called from `teleport_towards_player.rs:134` on goal start,
  the analogue of vanilla `LookForPlayerGoal.start()`.

## Remaining gaps

### 1. Projectile immunity keyed on damage type instead of the direct entity (S)
- File: `crates/pumpkin-core/src/entity/mob/enderman.rs:63-68`, `:570-573`
- Vanilla: `Enderman.hurtServer` clears the hit when `source.getDirectEntity() instanceof Projectile`
  (and the enderman is not a passenger), then runs the 64-attempt teleport.
- Pumpkin: `is_projectile_damage(damage_type)` matches any damage type tagged
  `minecraft:is_projectile`, so a tagged damage type from a non-projectile source would be
  silently cancelled, while a projectile whose damage type is not tagged would not be.
- Fix: test the damage source's direct entity for `Projectile` (a `get_projectile()`/downcast
  helper already exists for splash potions via `as_splash_potion`), keep the tag check only as
  a fallback if that is impossible.

### 2. Possible double teleport after a non-damaging splash potion (S)
- File: `crates/pumpkin-core/src/entity/mob/enderman.rs:574-593`
- Vanilla teleports the enderman once per thrown-potion hit (in `hurtServer`).
- Pumpkin teleports in `pre_damage_with_context` (`!potion.hurts_water_sensitive_entities()`,
  returns `false`) and again in `post_damage_with_context`, whose doc says it also runs when
  the damage was blocked - so the same hit appears to attempt two 64-try random teleports
  (up to 128 position rolls, two sound/particle sets).
- Fix: make `post_damage_with_context` skip when the pre-damage hook already teleported
  (return a flag or check the potion branch), keeping one teleport per hit.
- Confirm the pipeline first: `Entity::damage_with_context` in `crates/pumpkin-core/src/entity/`.

### 3. Freeze-while-stared-at goal lacks vanilla look limits and re-check semantics (S)
- File: `crates/pumpkin-core/src/entity/ai/goal/chase_player.rs:88-100`
- Vanilla `EndermanFreezeWhenLookedAt`: `canStart` only requires a player target that is
  currently stared at (`isLookingAtMe`), `tick` looks with the 30 deg yaw / 30 deg pitch
  limits, and `shouldContinue` re-checks the target identity plus `isLookingAtMe`.
- Pumpkin: `can_start` adds a `dist <= 256` gate (not in vanilla), and `tick` calls
  `look_control.look_at(...)` without the 30/30 limits, so the head snaps faster than vanilla.
- Fix: pass 30.0/30.0 through `LookControl::look_at` (add a limited variant if absent) and move
  the distance gate out of `can_start`; low impact (targeting only).

## Checked, no gap
- `custom_server_ai_step` (`:517`) day burn: `is_bright_outside()` == vanilla `Level.isDay()`
  (`sky_darken < 4`), 600-tick deaggression gate, light magic value > 0.5, sky visibility and
  the `random * 30 < (brightness - 0.4) * 2` roll all match.
- Teleport radii (32/64, 64 attempts, 16-block toward jump), packed offset level event 2018,
  game event + both teleport sounds, water-avoiding pathfinding malus, goal priorities
  (swim 0, freeze 1, melee 2, endermite hunt 3, revenge 2, reset anger 4, wander 7, look 8,
  block place 10, block take 11), creepy/starred/carry synced data, carried-block NBT +
  `requires_custom_persistence`, death drop through `EnchantmentProvider::ENDERMAN_LOOT_DROP`,
  scream ambient sound, scream Bedrock/Java metadata, water sensitivity and water-bottle gating.
