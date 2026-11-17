# entities / Endermite

Row: category=entities, name=Endermite, coverage=RE-REVIEW-UNTRACKED (re-review, source changed).
Source: `crates/pumpkin-core/src/entity/mob/endermite.rs`.

## Verified parity (no change needed)
- Goals: SwimGoal (=vanilla FloatGoal) + ClimbOnTopOfPowderSnowGoal p1, MeleeAttackGoal(1.0,false) p2, WaterAvoidingRandomStrollGoal p3, LookAtPlayer p7 8.0, RandomLookAround p8; target order HurtByTarget(alert others) p1, NearestAttackableTarget(Player, true) p2.
- Sounds: ambient/step 0.15 volume; hurt/death via generated `EntityType` data.
- XP 3, dimensions 0.4x0.3, eye height 0.13 from generated `entity_type.rs:3990-4024`.
- `Lifetime` NBT save/load, body yaw = yaw each tick, 2 portal particles/tick with vanilla random offsets, despawn at 2400 ticks when not persistence-required.

## Gaps
1. FIXED `crates/pumpkin-core/src/entity/projectile/ender_pearl.rs:138-146` - pearl-spawned endermite used the thrower's position/pitch; vanilla `EnderPearl#onHit` uses the pearl landing position and pitch 0. Size S.
2. OPEN `crates/pumpkin-core/src/entity/mob/endermite.rs:117-121` - particle offset uses `(r-0.5)*width` (about +-0.2 for width 0.4); vanilla `Entity#getRandomX(0.5)` / `getRandomY()` spread over the bounding box (+-0.5 factor for x/z). Client-visible cosmetic only. Size S, needs a bounded-scope check of Pumpkin's `Entity` random helpers before changing.
3. OPEN `crates/pumpkin-core/src/entity/mob/endermite.rs:99-111` - lifetime/despawn runs in `post_tick` gated on `!persistence_required`, while vanilla increments in `tick()`; ordering differs by one tick and the persistence gate must be confirmed against decompiled `Endermite#tick`. Size S, no observable divergence proven, do not change without vanilla source.
