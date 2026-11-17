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
