# Tracker: entities / Enderman (RE-REVIEW, 2026-11-17)

Re-audited `crates/pumpkin-core/src/entity/mob/enderman.rs` and its goals
(`chase_player.rs`, `teleport_towards_player.rs`, `pick_up_block.rs`, `place_block.rs`)
against vanilla `Enderman` / `EndermanFreezeWhenLookedAt` / `EndermanLookForPlayerGoal`.
No `.vendor/` decompiled tree in this worktree; vanilla facts from the 1.21 classes.

The source changed since the last review: the damage pipeline now runs
`pre_damage_with_context` -> damage -> `post_damage_with_context`
(`entity/mob/mod.rs:3195-3214`), and carried-block spawn metadata was fixed.

## Fixed in this pass

- `entity/ai/goal/chase_player.rs` (`EndermanFreezeWhenLookedAt`):
  - tick used `look_control.look_at` (default max yaw/pitch change), so the head snapped.
    Vanilla `EndermanFreezeWhenLookedAt.tick` uses `setLookAt(target, 30.0F, 30.0F)`.
    Now `look_at_with_range(..., 30.0, 30.0)`.
  - `can_start` had a `squared_distance > 256.0` gate (16 blocks) that vanilla does not
    have; vanilla selects the nearest player within 256 blocks and only requires
    `isLookingAtMe`. Gate removed.
  - Added `should_continue` re-checking target liveness and `isLookingAtMe`, matching
    vanilla continuation semantics.

## Resolved since last review (no gap)

- Double teleport after a non-damaging splash potion: `pre_damage_with_context` returning
  `false` makes `damage_with_context` return before `post_damage_with_context`
  (`entity/mob/mod.rs:3201`), so one 64-try teleport per hit. Confirmed no gap.

## Remaining gaps

### 1. Projectile dodge keyed on damage type, not the direct projectile entity (M)
- File: `crates/pumpkin-core/src/entity/mob/enderman.rs:67-71`, `:599-602`
- Vanilla `Enderman.hurtServer`: clear the hit when
  `source.getDirectEntity() instanceof Projectile` (and not a passenger), then run the
  64-attempt teleport. Pumpkin's `is_projectile_damage` checks the
  `minecraft:is_projectile` damage-type tag instead: a tagged damage type from a
  non-projectile source is wrongly dodged, and a projectile whose damage type is not
  tagged is not dodged.
- Fix: add an `EntityBase::is_projectile()` marker (no `Projectile` trait exists yet;
  `projectile/*` are separate structs) or match `entity.get_entity().entity_type`
  against the projectile types, and check that instead. Keep the tag only as a fallback.

### 2. Stare line-of-sight hard-capped at 128 blocks (S)
- File: `crates/pumpkin-core/src/entity/mob/enderman.rs:65`, `:399-403`
- Vanilla `Enderman.isLookingAtMe` uses `Player.hasLineOfSight(this)`, an unbounded block
  clip; there is no 128-block cap.
- Fix: drop `LINE_OF_SIGHT_RANGE` and rely on `world.clip` (the cone test already limits
  practical range). S.

## Checked, no gap

- `custom_server_ai_step` day burn (600-tick gate, brightness > 0.5, sky visibility,
  `random * 30 < (brightness - 0.4) * 2`).
- Teleport radii (32/64), 64 attempts, 16-block toward jump, packed offset level event
  2018, game event + both sounds, water-avoiding pathfinding malus, enderman-dne-teleport
  block tag.
- Goal priorities: swim 0, freeze 1, melee 2, endermite hunt 3, teleport-towards 1,
  revenge 2, reset anger 4, wander 7, look 8, place 10, pick up 11.
- Creepy/starred/carry synced data, stare scream at 400-tick cooldown, carried-block NBT,
  `requires_custom_persistence`, death drop via `EnchantmentProvider::ENDERMAN_LOOT_DROP`,
  ambient scream when creepy, water sensitivity, projectile side effects.

## Gap status
- 1 projectile dodge keyed on damage type (M): STILL OPEN (size M)
- 2 stare line-of-sight 128-block cap (S): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
