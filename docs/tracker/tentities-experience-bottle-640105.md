# entities / Experience Bottle (re-review 640105)
Coverage: RE-REVIEW-UNTRACKED. Re-audited after the source changed (hand-aware item use
`add8c3c16`, splash-sound fix `8e36d2d9a`, projectile hit refactor `c331e9a3b`). No decompiled
sources are vendored here (`.vendor/` absent), so comparison uses the vanilla classes
`net.minecraft.world.item.ExperienceBottleItem`, `net.minecraft.world.entity.projectile.ThrownExperienceBottle`,
`ThrowableItemProjectile`, `ExperienceOrb`, plus local ports (splash_potion.rs, fireball.rs,
dispenser.rs) and generated `world_event.rs` / `tracked_data` / `assets/items.json`.

Source audited:
- crates/pumpkin-core/src/entity/projectile/experience_bottle.rs
- crates/pumpkin-core/src/item/items/experience_bottle.rs
- crates/pumpkin-core/src/block/blocks/redstone/dispenser.rs:288-291,439-459,1318-1329
- crates/pumpkin-core/src/entity/experience_orb.rs:55,120-133

## Matches vanilla (no action)
- Throw sound EntityExperienceBottleThrow, Neutral, 0.5, 0.4/(rand*0.4+0.8) (item:43-49).
- shootFromRotation(xRot, yRot, -20.0F, 0.7F, 1.0F) via set_velocity_from(ROLL -20, POWER 0.7,
  DIVERGENCE 1.0) (item:15-17,57-59); trig matches Projectile.shootFromRotation.
- Projectile stack is a count-1 copy (item:55), matching ThrowableItemProjectile.setItem.
- Consume 1 unless creative, then ITEM_USED stat (item:62-64).
- No item cooldown: assets/items.json experience_bottle has no `minecraft:use_cooldown`
  component (ender_pearl and wind_charge do), so the registry's component-driven cooldown
  correctly leaves the bottle alone. No gap.
- Gravity 0.07 (entity:15,30,38) and dedicated throw power 0.7/1.0 match the projectile item
  settings for the bottle; tracked ITEM_STACK data synced from the stack (entity:57-68).
- on_hit: event 2002 ParticlesSpellPotionSplash with colour -13083194 (0x385DC6) at the floored hit
  position, then event 1053 SoundSpellPotionSplash, then 3 + rand(0..5) + rand(0..5) XP
  (entity:101-117). Both events are correct for this protocol target: generated world_event.rs:40
  has 2002 as *Particles* and world_event.rs:84 has 1053 as *Sound* (1054 instant), i.e. vanilla
  split the splash sound out of 2002 after the code that used one event.
- Hit entities take no damage; the projectile is removed by the shared hit flow
  (projectile/mod.rs:175-180), equal to vanilla discard().
- GameEvent::ProjectileLand on every hit (projectile/mod.rs:317-323) is vanilla Projectile.onHit
  behaviour, not a thrown-item deviation.
- Orb random spread is velocity, not position (experience_orb.rs:67-78).

## Fixed in this pass
- Thrown bottle did not persist its item stack: no `write_custom_nbt`/`read_custom_nbt`, so a
  save/load of an in-flight bottle (or a dispensed bottle whose stack carried components)
  fell back to a bare default stack. Added `Item` compound write/read via the existing
  EntityBase hooks (experience_bottle.rs:82-99), matching ThrowableItemProjectile saveAdditional
  and the local FireballEntity pattern (fireball.rs:109-125). Commit
  "fix(experience_bottle): persist thrown item stack in nbt".

## Remaining gaps
1. XP orbs are split but never merged. experience_bottle.rs:116 calls
   ExperienceOrbEntity::spawn (experience_orb.rs:55), which only splits via round_to_orb_size
   (experience_orb.rs:120-133). Vanilla ExperienceOrb.award(ServerLevel, Vec3, int) first calls
   tryMergeToExisting: an orb within a 1x1x1 box absorbs the amount (values summed, age reset) and
   only otherwise drops a new orb. Fix: add ExperienceOrbEntity::award with the merge step and
   route every award call site (12 callers today) through it. Size M (shared XP path).
2. Dispenser launch power/uncertainty unverified. CHECKED, no change (UNVERIFIED): no decompiled source to confirm the DispenseConfig. dispenser.rs:1318-1329 dispenses the bottle with
   POTION_PROJECTILE_POWER 1.375 / POTION_PROJECTILE_UNCERTAINTY 3.0 (dispenser.rs:288-291).
   Vanilla 1.21.2+ serves non-potion projectiles through ProjectileItem / ProjectileDispenseBehavior
   with the generic config (DEFAULT 1.1 / 6.0, already defined as
   DEFAULT_PROJECTILE_POWER/_UNCERTAINTY), while splash/lingering potions keep 1.375 / 3.0. Fix:
   switch the bottle branch to the DEFAULT constants after confirming the target version's
   ProjectileItem.DispenseConfig against decompiled vanilla. Size S, needs .vendor (not changed
   this pass: no decompiled source available to confirm).
3. FIXED (UNVERIFIED, bottle only): spawn at eye Y - 0.1 via `throw_spawn_pos` (experience_bottle.rs). Snowball/egg/pearl still use feet, shared path left for later.
   Thrown bottles spawn at the player's feet, not the vanilla eye offset. item/items/
   experience_bottle.rs:42,53 builds the entity at player.position(); vanilla ThrowableItemProjectile
   spawns at shooter.getEyeY() - 0.1. The same deviation exists for every thrown-item item
   (snowball.rs:47,56; egg.rs:47,57; ender_pearl.rs:50,60), so the fix belongs in the shared throw
   path (spawn position helper used by all of them), not in this file. Size M (shared).
4. No automated test. Cheapest useful one: spawn an ExperienceBottleEntity, call on_hit with a
   synthetic ProjectileHit::Block, assert the summed orb amount is in 3..=11, both world events
   fire once, and the entity is removed. Size M.
5. Splash potion (separate row, same cluster): splash_potion.rs:302-309 broadcasts only event
   2002 and never the 1053 splash sound, unlike this bottle. Logged as a pointer, not fixed here.


## Gap status
- 1 XP orbs never merged (M, shared XP path): STILL OPEN (size M)
- 2 dispenser launch power: STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 3 thrown bottle spawns at player feet (M, shared throw path across 4 items): STILL OPEN (size M)
- 4 no automated test (M): STILL OPEN (size M)
- 5 splash potion sound pointer: STILL OPEN (pointer only, splash_potion.rs out of scope)
