**FIXED** `39311be46` (fix(entity): endermite expires by generic damage in mob tick). Matched by commit subject on master; code not re-diffed in this sweep.

# entities / Endermite

Row: category=entities, name=Endermite, coverage=RE-REVIEW-UNTRACKED (re-review, source changed).
Source: `crates/pumpkin-core/src/entity/mob/endermite.rs`; spawn path `crates/pumpkin-core/src/entity/projectile/ender_pearl.rs:147-161`.
Method: no decompiled vanilla in `.vendor/`; compared against vanilla `Endermite` / `EnderPearl#onHit` behaviour and against the sibling ports (`silverfish.rs`, `ravager.rs`), plus generated data (`crates/pumpkin-data/src/generated/entity_type.rs:3958-3993`). Re-reviewed after commit `5b09270e6`, which fixed gap 2 below.

## Verified parity (no change needed)
- Goals `endermite.rs:48-61`: Swim (FloatGoal) + ClimbOnTopOfPowderSnow p1, MeleeAttack(1.0, false) p2, WaterAvoidingWander(1.0) p3, LookAtEntity(Player, 8.0) p7, RandomLookAround p8; targets Revenge(alerting others) p1, player hunt p2 = vanilla `registerGoals`.
- Step sound `EntityEndermiteStep` volume 0.15, pitch 1.0 (`endermite.rs:69-71`) matches the shared mob convention (`ravager.rs:85`, `silverfish.rs:227`); ambient sound (`73-75`); hurt/death/XP 3 from generated `EntityType` data (`entity_type.rs:3990-3993`), attributes HEALTH 8, ATTACK_DAMAGE 2, MOVEMENT_SPEED 0.25.
- Body yaw forced to yaw each tick (`endermite.rs:97`) = vanilla `Endermite#aiStep` (body rot = yaw), same shape as `silverfish.rs:251`.
- `Lifetime` int NBT round trip (`endermite.rs:81-89`) = vanilla `addAdditionalSaveData` / `readAdditionalSaveData`.
- 2400-tick lifetime (`endermite.rs:25`) now lives in `mob_tick` (= vanilla `customServerAiStep`) and applies 1.0 generic damage instead of removing the entity (`endermite.rs:91-108`), so expiry produces a normal death: death sound, animation, 3 XP. Counter still skipped while persistence-required.
- 2 portal particles per tick with vanilla random y range and velocity draw (`endermite.rs:111-136`).
- Pearl spawn `ender_pearl.rs:147-161`: 5% chance, players only, mob-spawning gated, `EntitySpawnReason::Triggered`, pearl position, pearl yaw and pitch 0 = vanilla `EnderPearl#onHit` `moveTo(x, y, z, getYRot(), 0.0F)`.

## Remaining gaps
1. OPEN `endermite.rs:122,124` - particle x/z offset is `(rng - 0.5) * dimensions.width` (about +-0.2 for width 0.4). Vanilla `Endermite#aiStep` uses `getParticleX(0.5)` / `getParticleZ(0.5)`; if those scale a centered offset by `0.5 * bbWidth`, the correct spread is about +-0.1. Cosmetic, client-visible only; needs the decompiled `Entity#getParticleX` body before changing. Vanilla class: `Endermite#aiStep`. Size S.
2. OPEN `endermite.rs:101-103` - the persistence gate (`!persistence_required`) may be a Pumpkin-only guard: the recalled vanilla `customServerAiStep` increments `life` unconditionally, which would make named/persistent endermites still expire. Not changed because the vanilla body could not be confirmed here and removing the guard would be a behaviour reversal. Vanilla class: `Endermite#customServerAiStep`. Size S.
3. OPEN `ender_pearl.rs:147` - the 5% roll uses `rand::random::<f32>()` rather than the projectile RNG, and the spawn position is the pearl's `pos` rather than `onHit`'s impact point. Statistically equivalent, not observable. Vanilla class: `EnderPearl#onHit`. Size S.
4. OPEN `endermite.rs:131-137` - particles are broadcast as server `World::spawn_particle` packets (count 0, velocity packed into the offset/spread fields, max speed 1.0), while vanilla emits them client-side with explicit per-particle velocity. Same visual class, not fixable without a per-particle particle packet. Vanilla class: `Endermite#aiStep`. Size S.

## h2 parity sweep (status)
1. FIXED 581555367 - portal particle x/z spread now `(rng - 0.5) * width * 0.5`, i.e. +-width*0.25 as vanilla `getRandomX(0.5)`. UNVERIFIED (no build).
2-4. STILL OPEN - not attempted this pass (persistence gate needs reference check, RNG/impact point not observable, client-side velocity packets are a protocol-level change).
