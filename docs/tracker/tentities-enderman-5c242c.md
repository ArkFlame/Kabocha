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

## Result

Re-verified both STILL OPEN rows against the decompiled source now present under .vendor/vanilla-decompiled/net/minecraft/. The previous audit had no decompiled tree and guessed vanilla from memory; both rows rest on incorrect premises. Pumpkin already matches the real source. No code changes were needed and no test was added (no behaviour change to test; a constant mirror test would be a forbidden formula/constant test).

### Row 1 (M) projectile dodge: NO GAP, closed
- Vanilla Enderman.hurtServer (world/entity/monster/Enderman.java:390-412) uses source.is(DamageTypeTags.IS_PROJECTILE) with !this.isPassenger(), i.e. the damage-type TAG. It does NOT use getDirectEntity() instanceof Projectile. The only direct-entity check there is getDirectEntity() instanceof AbstractThrownPotion for the water-sensitivity splash branch. The tracker premise was wrong.
- Pumpkin is_projectile_damage (enderman.rs:70-72) checks has_tag(MINECRAFT_IS_PROJECTILE); pre_damage_with_context (:600) gates it on !entity.has_vehicle() then calls repeatedly_try_to_teleport (:249, 64 attempts). has_vehicle() is exactly vanilla isPassenger() (vehicle.is_some()). This matches vanilla. Keep the tag check as is.

### Row 2 (S) stare line-of-sight 128-block cap: NO GAP, closed
- Vanilla LivingEntity.hasLineOfSight (world/entity/LivingEntity.java:4121-4132) DOES cap at 128 blocks: to.distanceToSqr(from) > Mth.square(128.0) fails the check. The tracker "unbounded block clip" claim is wrong (the Paper patch only rewrote it to squared distance).
- Pumpkin LINE_OF_SIGHT_RANGE = 128.0 check (enderman.rs:65, :402-404) matches that cap. The cone check also matches isLookingAtMe(player, 0.025, true, false, getEyeY()) (LivingEntity.java:2276-2296), endpoints and eye heights included.


### 2026-11-18 independent re-verification (both rows remain closed)
- Row 1 (M): confirmed at Enderman.java:396 `source.is(DamageTypeTags.IS_PROJECTILE) && !this.isPassenger()`; Pumpkin enderman.rs:70-72 + :600 match. No gap.
- Row 2 (S): confirmed at LivingEntity.java:4131 `to.distanceToSqr(from) > Mth.square(128.0)`; Pumpkin enderman.rs:65 + :402-404 match. No gap.
- No code changes or tests: both rows already match vanilla; no behaviour change to test.

### 2026-11-18 second independent re-verification (both rows closed)
- Row 1 (M): re-read the decompiled Enderman.hurtServer. Line 396 gates the teleport on `source.is(DamageTypeTags.IS_PROJECTILE) && !this.isPassenger()` (the damage-type tag), exactly what Pumpkin enderman.rs:70-72 (`has_tag(MINECRAFT_IS_PROJECTILE)`) + :600 (`!entity.has_vehicle()`) do. `has_vehicle()` is `isPassenger()`. The sole direct-entity check in that method is `getDirectEntity() instanceof AbstractThrownPotion` at line 395, which Pumpkin handles separately through `as_splash_potion`/`hurts_water_sensitive_entities` at :604-609. The tracker's "instanceof Projectile" premise is wrong. No gap.
- Row 2 (S): re-read decompiled LivingEntity.hasLineOfSight. Line 4131 rejects `to.distanceToSqr(from) > Mth.square(128.0)`, a hard 128-block cap. Pumpkin LINE_OF_SIGHT_RANGE = 128.0 (enderman.rs:65) with the squared-distance check at :402-404 matches. isLookingAtMe (LivingEntity.java:2276-2296) cone/eye-height/endpoint logic also matches enderman.rs:390-414. No gap.
- No code changes and no test added: both rows already match vanilla; adding a constant/clip-mirror test would be a forbidden formula/constant test.


### 2026-11-18 third independent re-verification (both rows VERIFIED, no gap)

Row 1 (M, projectile dodge): VERIFIED, matches vanilla. Decompiled Enderman.hurtServer at .vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Enderman.java:396 gates the teleport on `source.is(DamageTypeTags.IS_PROJECTILE) && !this.isPassenger()`, i.e. the damage-type TAG plus isPassenger, NOT `getDirectEntity() instanceof Projectile` as the tracker row claimed. Pumpkin enderman.rs:70-72 (`has_tag(MINECRAFT_IS_PROJECTILE)`) and :600 (`!entity.has_vehicle()`) match; `has_vehicle()` (crates/pumpkin-core/src/entity/mod.rs:6092-6097) is `vehicle.is_some()` == `getVehicle()!=null` == vanilla `isPassenger()`. The sole direct-entity check in hurtServer is `getDirectEntity() instanceof AbstractThrownPotion` at :395, handled separately by Pumpkin via `as_splash_potion`/`hurts_water_sensitive_entities` at enderman.rs:604-609 and post_damage at :622-627. No code change.

Row 2 (S, stare line-of-sight 128-block cap): VERIFIED, matches vanilla. Decompiled LivingEntity.hasLineOfSight at .vendor/vanilla-decompiled/net/minecraft/world/entity/LivingEntity.java:4117-4132 caps range at `to.distanceToSqr(from) > Mth.square(128.0)` (line 4131). Pumpkin `LINE_OF_SIGHT_RANGE = 128.0` (enderman.rs:65) with the squared-distance rejection at :402-404 matches; there is no missing cap. The cone/eye-height logic also matches: isLookingAtMe (LivingEntity.java:2276-2299), called from Enderman.java:235 with coneSize=0.025, adjustForDistance=true, seeThroughTransparentBlocks=false, gazeHeight=getEyeY(), is mirrored by Pumpkin enderman.rs:380-414 (STARE_CONE_SIZE=0.025, dist-normalized dot test at :395, ClipBlock::Collider clip at :408-413). No code change.

No tests added: verification-only pass; both rows already match vanilla and a constant/clip mirror test would be a forbidden formula/constant test. None of the five unit files were modified.


### 2026-11-18 fourth independent re-verification (both rows VERIFIED, no gap)

Row 1 (M, projectile dodge): VERIFIED, matches vanilla. Decompiled Enderman.hurtServer at .vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Enderman.java:396 gates the teleport on `source.is(DamageTypeTags.IS_PROJECTILE) && !this.isPassenger()` (the damage-type TAG plus isPassenger), NOT `getDirectEntity() instanceof Projectile` as the tracker row claimed. Pumpkin enderman.rs:70-72 (`has_tag(MINECRAFT_IS_PROJECTILE)`) and :600 (`!entity.has_vehicle()`) match; has_vehicle() is vehicle.is_some() == isPassenger(). The only direct-entity check in hurtServer is `getDirectEntity() instanceof AbstractThrownPotion` at :395, handled separately by as_splash_potion/hurts_water_sensitive_entities at :604-609 + post_damage :622-627. No code change.

Row 2 (S, stare line-of-sight 128-block cap): VERIFIED, matches vanilla. Decompiled LivingEntity.hasLineOfSight at .vendor/vanilla-decompiled/net/minecraft/world/entity/LivingEntity.java:4121-4132 caps range with `!(to.distanceToSqr(from) > Mth.square(128.0))` (line 4131); MAX_LINE_OF_SIGHT_TEST_RANGE = 128.0 at :189. Pumpkin LINE_OF_SIGHT_RANGE = 128.0 (enderman.rs:65) with the squared-distance rejection at :402-404 matches. Block clip with ClipContext.Block.COLLIDER / Fluid.NONE at :408-413 matches line 4132. No code change.

No tests added: verification-only pass; both rows already match vanilla and a constant/clip mirror test would be a forbidden formula/constant test. None of the five unit files were modified.
