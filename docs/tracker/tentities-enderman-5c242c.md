# Tracker: entities / Enderman (RE-REVIEW)

Audited `crates/pumpkin-core/src/entity/mob/enderman.rs`, `entity/mob/metadata.rs` and the
goal files (`teleport_towards_player.rs`, `chase_player.rs`, `pick_up_block.rs`,
`place_block.rs`) against vanilla 1.21 `Enderman` / `EndermanLookForPlayerGoal` /
`EndermanFreezeWhenLookedAt` / `EndermanTakeBlockGoal` / `EndermanLeaveBlockGoal`.
No `.vendor/` decompiled source is present in this worktree.

## Fixed in this pass

- `enderman.rs:579` `mob_metadata()` now includes the carried block
  (`tracked_data::enderman::DATA_CARRY_STATE` as an optional block state, i.e. a VarInt with
  0 = empty). Vanilla tracks this value on the entity from the first spawn, but Pumpkin only
  sent it through `set_synced_data`, and `Entity::send_dirty_entity_data` discards dirty
  values while the entity has no viewers (`entity/mod.rs:4800`). A naturally spawned or
  NBT-loaded enderman carrying a block therefore reached clients empty-handed and stayed that
  way until the carry state next changed. Spawn metadata is built solely from
  `Mob`/`mob_metadata` (`entity/mod.rs:861`, `entity/mob/mod.rs:1880`), so the value has to
  be part of it.

## Remaining gaps

### 1. Projectile dodge keyed on damage type instead of the direct entity (S)
- File: `crates/pumpkin-core/src/entity/mob/enderman.rs:67-70`, `:588-609`
- Vanilla: `Enderman.hurtServer` cancels the hit when `source.getDirectEntity() instanceof
  Projectile` (and the enderman has no vehicle), then runs the 64-attempt teleport.
- Pumpkin: `is_projectile_damage(damage_type)` matches any damage type tagged
  `minecraft:is_projectile`, so a tagged damage type from a non-projectile source is silently
  cancelled while a projectile with an untagged damage type would not be. `source`/`cause`
  are already passed in, so the direct entity is available.
- Fix: test the damage source's direct entity for `Projectile` (a downcast helper already
  exists for splash potions via `as_splash_potion`), keep the tag check only as a fallback.

### 2. Stare detection requires line of sight, which vanilla does not (M)
- File: `crates/pumpkin-core/src/entity/mob/enderman.rs:361-413`
- Vanilla `Enderman.isLookingAtMe` is only: gaze-disguise head item check, normalized look
  vector, `dot > 1.0 - 0.025 / distance`. There is no clip/line-of-sight test and no 128-block
  cap.
- Pumpkin adds a `world.clip(...)` miss requirement (line 403-412) and a
  `LINE_OF_SIGHT_RANGE = 128` cutoff (line 65, 398-402), so a player staring at an enderman
  through glass or at long range never angers it.
- Fix: drop the clip and range checks; `CAUTION` this also affects `ChasePlayerGoal` and
  `TeleportTowardsPlayerGoal`, so confirm the intent of the extra gate before removing it.

### 3. Freeze-while-stared-at goal claims movement controls (S)
- File: `crates/pumpkin-core/src/entity/ai/goal/chase_player.rs:78-80`
- Vanilla `EndermanFreezeWhenLookedAt` overrides no `getFlags`, i.e. it claims no controls and
  does not block `MeleeAttackGoal` (priority 2) from steering. The 256 (squared distance) gate
  in `canUse` is faithful and must stay.
- Pumpkin returns `Controls::JUMP | Controls::MOVE`, so while a player stares the goal freezes
  movement for as long as it runs.
- Fix: return `Controls::empty()`; if the freeze is wanted, it needs a separate justification
  because it changes the chase behaviour.

## Checked, no gap

- `play_stare_sound` (400-tick cooldown, `EntityEndermanStare` volume 2.5, silent-aware).
- `pre_damage_with_context` returning `false` short-circuits `damage_with_context`
  (`entity/mob/mod.rs:3183-3185`), so a non-damaging splash potion produces exactly one
  64-attempt teleport, not two (contradicts the previous review's gap 2).
- `custom_server_ai_step` day de-aggro: `is_bright_outside`, 600-tick deaggression gate,
  brightness > 0.5, sky visibility and the `random * 30 < (brightness - 0.4) * 2` roll match.
- Teleport radii (32/64, 64 attempts, 16-block toward jump), packed offset level event 2018,
  game event plus both teleport sounds, water-blocked random teleport, water pathfinding malus.
- Goal priorities (swim 0, freeze 1, melee 2, endermite hunt 3, revenge 2, reset anger 4,
  wander 7, look 8, block place 10, block take 11) and the carry/creepy/starred synced data.
- Carried-block NBT (`carriedBlockState` compound plus legacy int), `requires_custom_persistence`,
  `mob_remove_nbt`, death drop through `EnchantmentProvider::ENDERMAN_LOOT_DROP`.
- Pick-up/place goals: mob-griefing gate, 1/20 and 1/2000 rolls, ENDERMAN_HOLDABLE tag, clip
  check, surface/not-bedrock/collision gates, plugin `EntityChangeBlockEvent` cancel.
