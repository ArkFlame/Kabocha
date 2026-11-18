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

## Gap status (h6 sweep)
- Gap 2 (double teleport after non-hurting splash potion): FIXED 99fa5cfb8. Post-damage only teleports for water-sensitive potions; pre-damage already handles the rest.
- Gap 1 (projectile immunity keyed on damage type): STILL OPEN. Needs a direct-entity projectile helper; not done in this sweep.
- Gap 3 (freeze look limits 30/30 and re-check): FIXED (UNVERIFIED), already in code: `goal/chase_player.rs:67` uses `look_at_with_range(.., 30.0, 30.0)`.

## Result

- Gap 1 (projectile immunity): FIXED. Verified against decompiled `Enderman.hurtServer`
  (`.vendor/vanilla-decompiled/.../monster/Enderman.java:395-399`): vanilla gates the dodge on
  `source.is(DamageTypeTags.IS_PROJECTILE)` (a damage *type* tag), not on the direct entity, so the
  tracker premise was wrong. The real defect was in Pumpkin's `is_projectile_damage`: it compared
  the tag's registry-name list (`tag.0`) against `message_id` (translation keys), which differ for
  `mob_projectile`/`mob`, `unattributed_fireball`/`onFire`, `wither_skull`/`witherSkull`,
  `wind_charge`/`mob` -> those projectiles were never dodged. Replaced with
  `damage_type.has_tag(&tag::DamageType::MINECRAFT_IS_PROJECTILE)` (canonical numeric-id check used
  in `living.rs`, includes dynamic tags), added bug-repro unit test `projectile_damage_uses_damage_type_tag`.
  Ran `rustfmt --edition 2024` on `enderman.rs`.
- Gap 2 (double splash-potion teleport): already FIXED (99fa5cfb8). Re-verified in
  `enderman.rs:613-636`: post-damage teleports only for water-sensitive potions; pre-damage handles
  the rest, so one teleport per hit. No change.
- Gap 3 (freeze look limits): already FIXED. Re-verified in `chase_player.rs:57-84`:
  `look_at_with_range(.., 30.0, 30.0)`, no 256-block gate in `can_start`, and `should_continue`
  re-checks `is_player_staring`. No change.

## Result (re-review 2026-11-18)

- All three rows are already fixed; no source changes were needed this pass.
- Gap 1 (projectile immunity, the only STILL OPEN row): confirmed FIXED and correct. Re-read decompiled
  `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Enderman.java:390-415`: vanilla gates the
  dodge on `source.is(DamageTypeTags.IS_PROJECTILE) && !this.isPassenger()`, a damage-type tag (not a
  direct-entity check), so the tracker premise was wrong. `enderman.rs:70-72` now uses
  `damage_type.has_tag(&tag::DamageType::MINECRAFT_IS_PROJECTILE)` with the `!entity.has_vehicle()` gate,
  matching the same canonical check used in `living.rs:3581`. Imports (`tag`, `tag::Taggable`, `DamageType`)
  and the six referenced `DamageType` constants all exist; `has_tag` reads the static registry-id list so the
  unit test `projectile_damage_uses_damage_type_tag` needs no live registry. No change.
- Gap 2 (double splash-potion teleport): confirmed FIXED. `entity/mob/mod.rs:3267-3269` returns early when
  `pre_damage_with_context` is false, so `post_damage_with_context` is skipped for any pre-teleported hit
  (projectiles and non-hurting potions). Water-sensitive potions teleport once in post-damage. Matches
  vanilla `hurtServer` branches. No change.
- Gap 3 (freeze look limits / re-check): confirmed FIXED. `chase_player.rs:57-84` uses
  `look_at_with_range(.., 30.0, 30.0)`, has no 256-block gate in `can_start`, and `should_continue`
  re-checks `is_player_staring`. No change.
- No .rs file modified, so rustfmt was not run.

## Result (CODER re-verify 2026-11-18)

- Only STILL OPEN row is Gap 1 (projectile immunity). Verified against decompiled vanilla
  `Enderman.hurtServer` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Enderman.java:395-399`):
  the dodge is gated on `source.is(DamageTypeTags.IS_PROJECTILE) && !this.isPassenger()`, a damage-type
  tag check, NOT a direct-entity `instanceof Projectile` check as the tracker row claimed. The premise
  was wrong; Pumpkin is already correct.
- `enderman.rs:70-72` `is_projectile_damage` uses `damage_type.has_tag(&tag::DamageType::MINECRAFT_IS_PROJECTILE)`,
  the canonical numeric-id tag check (same as `living.rs:3595`), covering dynamic tags and all projectile
  translation-key mismatches (`mob_projectile`, `unattributed_fireball`, `wither_skull`, `wind_charge`).
  `pre_damage_with_context:600` pairs it with `!entity.has_vehicle()`, which reads `Entity.vehicle` and is
  the exact analogue of vanilla `!this.isPassenger()`. Bug-repro test `projectile_damage_uses_damage_type_tag`
  (`:654-663`) is present. No change.
- Gaps 2 and 3 already marked FIXED; spot-checked again: `post_damage_with_context:620-627` teleports once
  per water-sensitive potion (pre-damage handles the rest), and `chase_player.rs` uses `look_at_with_range(.., 30.0, 30.0)`
  with no 256-gate and `is_player_staring` re-check. Both correct, no change.
- No .rs file modified, so rustfmt was not run.


## Result (CODER third pass 2026-11-18, decompiled vanilla available)

- Decompiled source is now present, so all rows were re-verified mechanically against it. No .rs source changes were needed.
- Gap 1 (projectile immunity, the only previously STILL OPEN row): FIXED, VERIFIED. Decompiled
  `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Enderman.java:396` reads
  `if (source.is(DamageTypeTags.IS_PROJECTILE) && !this.isPassenger())` - a damage-type tag check,
  NOT a direct-entity `instanceof Projectile` check. The tracker premise was wrong; Pumpkin already
  matches vanilla. `enderman.rs:70-72` `is_projectile_damage` uses
  `damage_type.has_tag(&tag::DamageType::MINECRAFT_IS_PROJECTILE)`; `has_tag`
  (`crates/pumpkin-data/src/generated/tag.rs:32508-32512`) checks the static registry-id list plus
  dynamic tags, so the tag lookup is by numeric id and needs no live registry. The constant
  `MINECRAFT_IS_PROJECTILE` exists at `tag.rs:20226`; imports `tag`, `tag::Taggable`, `DamageType`
  are present at `enderman.rs:21-24`. `pre_damage_with_context:600` pairs the check with
  `!entity.has_vehicle()`, the analogue of vanilla `!this.isPassenger()`. Bug-repro unit test
  `projectile_damage_uses_damage_type_tag` (`enderman.rs:654-663`) covers the translation-key
  mismatches (mob_projectile, unattributed_fireball, wither_skull, wind_charge).
- Gap 2 (double splash-potion teleport): FIXED, VERIFIED. `crates/pumpkin-core/src/entity/mob/mod.rs:3267-3269`
  returns early when `pre_damage_with_context` is false, so `post_damage_with_context` is skipped for
  any pre-teleported hit (projectiles and non-hurting potions). Water-sensitive potions teleport once
  in post-damage (`enderman.rs:622-627`). Matches vanilla `hurtServer` branches at `Enderman.java:396-405`.
- Gap 3 (freeze look limits / re-check): FIXED, VERIFIED. `chase_player.rs:67` uses
  `look_at_with_range(.., 30.0, 30.0)`, `can_start` (`:24-49`) has no 256-block gate, and
  `should_continue` (`:71-84`) re-checks `is_player_staring` on the same target identity. Matches
  vanilla `EndermanFreezeWhenLookedAt` canStart/tick/shouldContinue semantics.
- No .rs file modified, so rustfmt was not run (nothing to format).
