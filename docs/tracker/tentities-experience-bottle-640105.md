# entities / Experience Bottle (re-review 640105)

Coverage: RE-REVIEW-UNTRACKED. Re-audited after the source changed. No decompiled sources are
vendored in this worktree (`.vendor/` absent), so comparisons use the vanilla classes
`net.minecraft.world.item.ExperienceBottleItem` and
`net.minecraft.world.entity.projectile.ThrownExperienceBottle`, plus local ports
(splash_potion.rs, dispenser.rs) and generated `world_event`/`tracked_data`.

Source audited:
- crates/pumpkin-core/src/entity/projectile/experience_bottle.rs
- crates/pumpkin-core/src/item/items/experience_bottle.rs
- crates/pumpkin-core/src/block/blocks/redstone/dispenser.rs:298-299,1309-1329
- crates/pumpkin-core/src/entity/projectile/mod.rs:131-181,287-324 (shared hit flow)
- crates/pumpkin-core/src/entity/experience_orb.rs:55-57,120-133

## Matches vanilla (no action)
- Throw sound EntityExperienceBottleThrow, Neutral, 0.5, 0.4/(rand*0.4+0.8) (item:43-49).
- shootFromRotation(xRot, yRot, -20.0F, 0.7F, 1.0F) through set_velocity_from(roll -20, power 0.7,
  divergence 1.0) (item:15-17,57-59); trig matches Projectile.shootFromRotation.
- Projectile stack is a copy with count 1 (item:55); vanilla ThrowableItemProjectile.setItem stores
  `stack.copy()` with count 1, so count forcing is correct.
- Consume 1 unless creative, then ITEM_USED stat (item:62-64).
- Gravity 0.07 (entity:16,30,38), spawn as EntityType::EXPERIENCE_BOTTLE, ITEM_STACK tracked data
  synced from the stack (entity:57-68); tracked_data id 8 matches vanilla DATA_ITEM_STACK.
- on_hit: event 2002 ParticlesSpellPotionSplash with colour -13083194 (0x385DC6) at the floored hit
  position, then event 1053 SoundSpellPotionSplash, then 3 + rand(0..5) + rand(0..5) XP
  (entity:82-99). Both events unconditionally is vanilla: ThrownExperienceBottle never consults
  isSilent(), and the current protocol target splits the splash sound out of event 2002.
- Hit entities take no damage; the projectile is removed by the shared hit flow
  (projectile/mod.rs:175-180), equal to vanilla discard().
- GameEvent::ProjectileLand on every hit (projectile/mod.rs:317-323) is vanilla Projectile.onHit
  behaviour, not a thrown-item deviation; the previous report's gap 5 was wrong.
- Orb random spread is velocity, not position (experience_orb.rs:67-78); the previous report's
  gap 2 (drop offset) is not a gap.

## Fixed in this pass
- Silent guard on the splash sound (entity:93-95) removed: vanilla sends both level events for a
  silent bottle too. Commit "fix(experience_bottle): always send splash sound level event".

## Remaining gaps
1. XP orbs are split but never merged. crates/pumpkin-core/src/entity/projectile/experience_bottle.rs:99
   calls ExperienceOrbEntity::spawn (crates/pumpkin-core/src/entity/experience_orb.rs:55), whose loop
   only splits via round_to_orb_size (experience_orb.rs:120-133,170-190). Vanilla
   ExperienceOrb.award(ServerLevel, Vec3, int) first calls tryMergeToExisting: an orb within a
   1x1x1 box around the position absorbs the amount (value summed, age reset) and only otherwise a
   new orb is dropped. Fix: add ExperienceOrbEntity::award with the merge step plus
   ExperienceOrb::tryMergeToExisting, and route every award call site through it.
   Size M (shared XP path).
2. Dispenser launch power/uncertainty unverified.
   crates/pumpkin-core/src/block/blocks/redstone/dispenser.rs:298-299 applies
   POTION_PROJECTILE_POWER = 1.375 / POTION_PROJECTILE_UNCERTAINTY = 3.0 to the experience bottle
   (dispenser.rs:1309-1329). Since 1.21.2 vanilla serves the bottle through ProjectileItem /
   ProjectileDispenseBehavior with the generic projectile configuration (power 1.1F, uncertainty
   6.0F), while splash/lingering potions keep their own config. Fix: give the bottle its own
   power/uncertainty constants after confirming the target version's ProjectileDispenseBehavior
   values against decompiled vanilla. Size S/M (needs .vendor).
3. No automated test. Cheapest useful one: spawn an ExperienceBottleEntity, call on_hit with a
   synthetic ProjectileHit::Block, assert the summed orb amount is in 3..=11 and that the entity is
   removed. Size M.
4. World-event coverage is unverified end to end: no test asserts that both event 2002 and 1053 are
   broadcast once per bottle break, so a regression to the old silent/one-event behaviour would go
   unnoticed. Suggest alongside gap 3. Size S/M.
