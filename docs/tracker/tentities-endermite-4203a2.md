# entities / Endermite

Row: category=entities, name=Endermite, coverage=RE-REVIEW-UNTRACKED (re-review, source changed).
Source: `crates/pumpkin-core/src/entity/mob/endermite.rs`; spawn path `crates/pumpkin-core/src/entity/projectile/ender_pearl.rs:134-151`.
Method: no decompiled vanilla in `.vendor/`; compared against vanilla `Endermite` / `EnderPearl#onHit` behaviour and against the sibling ports (`silverfish.rs`, `ravager.rs`). No code changed this pass: no gap was both certain and observable (WORKER.md rule 12).

## Verified parity (no change needed)
- Goals `endermite.rs:48-60`: Swim (FloatGoal) + ClimbOnTopOfPowderSnow p1, MeleeAttack(1.0, false) p2, WaterAvoidingWander(1.0) p3, LookAtEntity(Player, 8.0) p7, RandomLookAround p8; targets Revenge(alerting others) p1, player hunt p2 = vanilla `registerGoals`.
- Step sound `EntityEndermiteStep` volume 0.15, pitch 1.0 (`endermite.rs:68-70`) matches the shared mob convention (`ravager.rs:85`, `silverfish.rs:227`); ambient sound (`72-74`); hurt/death from generated `EntityType` sound data.
- Body yaw forced to yaw each tick (`endermite.rs:96`) = vanilla `Endermite#tick` (body rot = yaw), same shape as `silverfish.rs:251`.
- `Lifetime` int NBT round trip (`endermite.rs:80-88`) = vanilla `addAdditionalSaveData` / `readAdditionalSaveData`.
- 2400-tick limit (`endermite.rs:25`), counter skipped while persistence-required (`105-107`), 2 portal particles per tick with vanilla random offset ranges (`116-134`).
- Pearl spawn `ender_pearl.rs:134-151`: 5% chance, players only, mob-spawning gated, `EntitySpawnReason::Triggered`, pearl position, pearl yaw and pitch 0 = vanilla `EnderPearl#onHit` `moveTo(x, y, z, getYRot(), 0.0F)`. The previous review's gap 1 is fixed (verified at current lines).

## Remaining gaps
1. OPEN `endermite.rs:116-121` - particle x/z offset is `(rng - 0.5) * dimensions.width` (about +-0.2 for width 0.4). Vanilla `Entity#getRandomX(0.5)` / `getRandomZ(0.5)` also scale by the bounding width and give about +-0.1 if the helper is `(r - 0.5) * factor * bbWidth`. Cosmetic, client-visible only; needs the decompiled `Entity#getRandomX` body before changing. Vanilla class: `Endermite#aiStep`. Size S.
2. OPEN `endermite.rs:99-111` - lifetime counter and despawn run in `post_tick` (after the living tick) and are gated on `!persistence_required`; vanilla increments in `customServerAiStep` (mob tick, before `super.aiStep`) and ends the endermite by taking generic damage (death sound, drops, XP) rather than a silent `Entity::remove()`. A named/persistent endermite never expiring, and the silent removal, are the two observable differences; both need the decompiled `Endermite#customServerAiStep` to confirm. Vanilla class: `Endermite`. Size S.
3. OPEN `ender_pearl.rs:136-140` - the 5% roll uses `rand::random::<f32>()` rather than the projectile RNG, and the spawn position is the pearl's `pos` rather than `onHit`'s impact point. Statistically equivalent, not observable. Vanilla class: `EnderPearl#onHit`. Size S.
4. OPEN `endermite.rs:127-133` - particles are broadcast as server `World::spawn_particle` packets (count 0, velocity packed into the offset/spread fields, max speed 1.0), while vanilla emits them client-side with explicit per-particle velocity. Same visual class, not fixable without a per-particle particle packet. Vanilla class: `Endermite#aiStep`. Size S.
