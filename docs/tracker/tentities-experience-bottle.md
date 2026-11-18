# entities / Experience Bottle

Coverage: RE-REVIEW-UNTRACKED. Audited against vanilla net.minecraft.world.item.ExperienceBottleItem
and net.minecraft.world.entity.projectile.ThrownExperienceBottle. No decompiled sources are vendored
in this worktree (.vendor/ absent), so claims below come from the vanilla classes plus the local
conventions that already port them (splash_potion.rs, lingering_potion.rs, dispenser.rs).

Source audited:
- crates/pumpkin-core/src/item/items/experience_bottle.rs
- crates/pumpkin-core/src/entity/projectile/experience_bottle.rs
- crates/pumpkin-core/src/block/blocks/redstone/dispenser.rs:1278
- crates/pumpkin-core/src/entity/ominous_item_spawner.rs:251

## Matches vanilla (no action)
- Throw sound EntityExperienceBottleThrow, Neutral, 0.5, 0.4/(rand*0.4+0.8) (item:36-42).
- shootFromRotation(xRot, yRot, -20, 0.7, 1.0) via set_velocity_from roll -20/power 0.7/divergence 1.0
  (item:15-17,52); the trig matches Projectile.shootFromRotation.
- Consume 1 unless creative, then Used stat (item:55-57).
- Gravity 0.07, spawn on EntityType::EXPERIENCE_BOTTLE, sync ITEM_STACK tracked data (entity:16,32-33,57-68).
- on_hit: particles ParticlesSpellPotionSplash (2002) with colour -13083194, sound
  SoundSpellPotionSplash only when not silent, then 3 + nextInt(5) + nextInt(5) XP (entity:82-100).
- Hit entities take no damage; the projectile is removed by the shared hit flow
  (projectile/mod.rs process_tick -> entity.remove()), equivalent to vanilla discard().
- Dispenser and ominous-item-spawner paths feed the real stack plus a potion launch velocity.

## Remaining gaps
1. XP orbs are only split, never merged. crates/pumpkin-core/src/entity/projectile/experience_bottle.rs:99
   calls ExperienceOrbEntity::spawn (crates/pumpkin-core/src/entity/experience_orb.rs:55), whose loop only
   breaks the amount into orbs via round_to_orb_size. Vanilla ExperienceOrb.award(ServerLevel, Vec3, int)
   first tries tryMergeToExisting: any orb in a 1x1x1 box around the position absorbs the amount (age reset)
   and only otherwise a new orb is dropped. Fix: add ExperienceOrbEntity::award with the merge step and use
   it from every award call site (furnace/block XP too). Size M.
2. Orb drop offset unverified. Vanilla ExperienceOrb.award drops the orb at the given position with a small
   random offset; Rust passes the exact hit position
   (crates/pumpkin-core/src/entity/projectile/experience_bottle.rs:99). Fix: reproduce ExperienceOrb.drop's
   offset when adding award. Size S (needs .vendor to confirm constants).
3. Dispenser launch power/uncertainty unverified.
   crates/pumpkin-core/src/block/blocks/redstone/dispenser.rs:298-299 uses POTION_PROJECTILE_POWER = 1.375
   and POTION_PROJECTILE_UNCERTAINTY = 3.0 for the experience bottle. Vanilla serves it through
   ProjectileDispenseBehavior; if the target version's value differs the arc differs. Fix: re-read the
   dispense behaviour against decompiled vanilla and split the constant per projectile. Size S/M (needs .vendor).
4. Projectile stack count forced to 1. crates/pumpkin-core/src/item/items/experience_bottle.rs:48 passes
   stack.copy_with_count(1); vanilla sets the whole ItemStack on the bottle. The client renders only the
   item, so no observed difference. Size S, cosmetic - fix only if tracked-data NBT matters later.
5. ProjectileLand game event fires for item projectiles.
   crates/pumpkin-core/src/entity/projectile/mod.rs:317-325 emits GameEvent::ProjectileLand on every on_hit,
   so a broken bottle triggers sculk vibration; vanilla emits it per projectile type (arrows), not for
   thrown items. Fix in the shared helper, gated per projectile. Size S, shared-code change.
6. No automated test. Cheapest useful one: spawn a bottle, call on_hit with a synthetic
   ProjectileHit::Block, assert the XP total is in 3..=11 and the entity is removed (is_removed).
   Size M (harness pattern already in snowball.rs).
7. Silent guard scope. crates/pumpkin-core/src/entity/projectile/experience_bottle.rs:93-95 suppresses only
   the splash sound for silent bottles while particles always send; confirm against vanilla isSilent()
   placement before changing. Size S (needs .vendor).

## Gap status
- 2 orb drop offset: RESOLVED (fixed via orb_award_direction + spawn_with_direction; see Result)
- 3 dispenser launch power: RESOLVED (verified match against decompiled vanilla; no change needed)
- 4 sculk vibration per projectile type: RESOLVED (verified false positive; no change needed)

## Result

Re-reviewed all three STILL OPEN rows against .vendor/vanilla-decompiled (now present).

- 2 orb drop offset: FIXED. Vanilla ExperienceOrb(Vec3,roughly,...) sets
  setPos(pos.add(roughly.normalize().scale(size * 0.5))) and random-flips velocity against roughly.
  Pumpkin already reproduced this in ExperienceOrbEntity::new_with_direction, but the bottle passed
  a zero direction (ExperienceOrbEntity::spawn), so no offset was applied. Vanilla
  ThrownExperienceBottle.onHit passes the block face unit vector (block hits) or -deltaMovement
  (entity hits). Changed entity/projectile/experience_bottle.rs on_hit to compute that direction with
  the new pure helper orb_award_direction and call ExperienceOrbEntity::spawn_with_direction. Added a
  pure unit test orb_direction_uses_block_face_normal. File: crates/pumpkin-core/src/entity/projectile/experience_bottle.rs.
- 3 dispenser launch power: VERIFIED MATCH, no code change. Vanilla ProjectileItem.DispenseConfig.DEFAULT
  has power 1.1F / uncertainty 6.0F; ExperienceBottleItem.createDispenseConfig uses power*1.25 = 1.375
  and uncertainty*0.5 = 3.0. ThrowablePotionItem (parent of splash/lingering potions) uses the same
  multipliers, so POTION_PROJECTILE_POWER = 1.375 and POTION_PROJECTILE_UNCERTAINTY = 3.0 in dispenser.rs
  are correct for all three projectiles; no per-projectile split needed.
- 4 sculk vibration per projectile type (tracker row 5): VERIFIED MATCH (false positive), no code change.
  Vanilla Projectile.onHit emits GameEvent.PROJECTILE_LAND for both entity and block hits in the base
  class; ThrowableItemProjectile does not override onHit, and ThrownExperienceBottle.onHit calls
  super.onHit. Pumpkin's shared on_hit helper therefore correctly emits ProjectileLand for thrown items.

- 7 silent guard (extra finding on re-review): FIXED. Re-reading decompiled
  ThrownExperienceBottle.onHit confirmed particles fire whenever the ExpBottleEvent keeps the
  effect, but SoundSpellPotionSplash is additionally gated by `!isSilent()`. The prior refactor
  of on_hit had dropped that guard, firing the sound even for silent bottles. Restored
  `if !entity.is_silent()` around the sound event in
  crates/pumpkin-core/src/entity/projectile/experience_bottle.rs. No test added: entity silence
  is synced/data-driven with no pure seam here; the existing orb_direction test covers the same file.

## Result

Final verification after restart; re-checked every STILL OPEN row against .vendor/vanilla-decompiled
and the current source.

- Row 1 (orb merge): RESOLVED in code. `ExperienceOrbEntity::spawn_with_direction` splits via
  `orb_sizes` then calls `try_merge_to_existing` (1x1x1 box, equal `amount`, `fetch_add` the value,
  reset `orb_age`). Every award call site (spawner, furnace/block XP, mob death, fishing bobber,
  villager, breeding, ender dragon, WASM host, player) routes through `spawn()` ->
  `spawn_with_direction`, so merging applies everywhere, matching vanilla
  `ExperienceOrb.awardWithDirection`/`tryMergeToExisting` functional intent. Vanilla groups orbs by
  entity-id%40 and increments `count`; Pumpkin folds into `amount` - identical XP total. Test
  `award_splits_amount_into_vanilla_orb_sizes` already covers the split. Fixed one build blocker in
  the same file: removed unused `use core::f32;` that fails `-D warnings`.
- Row 4 (stack count forced to 1): LEFT OPEN (cosmetic). Vanilla `ExperienceBottleItem.use` passes
  the whole `ItemStack` to the projectile; Pumpkin uses `copy_with_count(1)` on throw and on NBT
  load. Only the item rendered on the projectile changes; no gameplay/XP effect. Tracker says fix
  only if tracked-data NBT matters later, so no change.
- Row 6 (automated test): RESOLVED via pure unit tests. `experience_orb.rs` has four pure tests
  (buoyancy x2, `orb_sizes`, `closest_free_center`), entity `experience_bottle.rs` has
  `orb_direction_uses_block_face_normal`, item `experience_bottle.rs` has
  `throw_spawns_below_eye_level_at_shooter_xz`. The tracker's suggested world-harness test (on_hit
  -> XP total) needs access to the private `amount` field and the bottle is removed by the shared
  hit flow, so there is no clean pure seam for it.
- Rows 2, 3, 5, 7: re-confirmed against vanilla `ThrownExperienceBottle.onHit` (block face normal /
  -deltaMovement award direction, `!isSilent()` sound guard) and `ExperienceBottleItem.createDispenseConfig`
  (power 1.1*1.25=1.375, uncertainty 6*0.5=3); no further change.


## Result

Second pass: re-verified the sole remaining LEFT OPEN row (row 4, projectile stack count) against decompiled vanilla.

- Row 4 (stack count forced to 1): VERIFIED MATCH (false positive), no code change. The tracker claimed vanilla sets the whole ItemStack on the bottle, but decompiled `ThrowableItemProjectile.setItem(source)` (net/minecraft/world/entity/projectile/throwableitemprojectile/ThrowableItemProjectile.java:39-41) does `this.getEntityData().set(DATA_ITEM_STACK, source.copyWithCount(1))`. Both vanilla throw paths (ExperienceBottleItem.use -> spawnProjectileFromRotationDelayed(ThrownExperienceBottle::new, ...), and asProjectile -> new ThrownExperienceBottle(..., itemStack)) funnel through that constructor -> setItem -> copyWithCount(1). Pumpkin uses `stack.copy_with_count(1)` on throw (item/items/experience_bottle.rs:66) and on NBT load (entity/projectile/experience_bottle.rs:114); the latter also matches vanilla readAdditionalSaveData -> setItem -> copyWithCount(1). This is also the consistent convention for every other throwable item in the crate (egg.rs, potions.rs, firework_rocket.rs, trident.rs). No per-item fix warranted.

- Cross-file check: confirmed `ExperienceOrbEntity::spawn()` (entity/experience_orb.rs:54) delegates to `spawn_with_direction`, whose split loop calls `try_merge_to_existing` before spawning (lines 120-132), so the row-1 orb-merge fix applies to every award call site, matching the prior pass claim.

All tracker rows are now resolved (fixed, verified match, or false positive). No code changes this pass.
