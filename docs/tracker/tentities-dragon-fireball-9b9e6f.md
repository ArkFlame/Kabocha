**FIXED** `ae8c43e05` (fix(entity): drop non-vanilla entity damage vibration on dragon fireball burst). Matched by commit subject on master; code not re-diffed in this sweep.

# Tracker: entities / Dragon Fireball (RE-REVIEW-UNTRACKED)

Audit date: 2026-11-17 (third pass). Source changed again in
`979aa8a83 feat(entity): emit entity damage vibration when a dragon fireball bursts`, so the previous
assessment was re-done; that commit's change is itself part of this review.

Sources reviewed:
- `crates/pumpkin-core/src/entity/projectile/dragon_fireball.rs` (whole file)
- `crates/pumpkin-core/src/entity/area_effect_cloud.rs` (`create_dragon_breath`, `tick`)
- `crates/pumpkin-core/src/entity/projectile/mod.rs:260-324,391-452` (generic hit handling,
  `HurtingProjectile::process_tick`)
- `crates/pumpkin-core/src/entity/projectile/owner.rs` (`EntityReference`)
- `crates/pumpkin-core/src/entity/boss/ender_dragon/phase/strafing.rs:150-190` (spawn site)
- vanilla classes `net.minecraft.world.entity.projectile.DragonFireball`,
  `AbstractHurtingProjectile`, `AreaEffectCloud`

No decompiled vanilla source is vendored in this worktree (`.vendor/` absent), so the comparison is
against the vanilla classes from knowledge; items marked *verify* still need a decompiled reference.

## Verified as matching vanilla

| Behaviour | Rust | Vanilla |
|---|---|---|
| splash box 4.0 x 2.0 x 4.0 around the fireball | `dragon_fireball.rs:119-123` | `getBoundingBox().inflate(4.0, 2.0, 4.0)` |
| candidate list is every `LivingEntity` in the box, no owner/spectator/alive filter | `dragon_fireball.rs:128-145` | `getEntitiesOfClass(LivingEntity.class, box)` |
| cloud snaps to the first candidate within 4.0 | `dragon_fireball.rs:140-145` | `distanceToSqr(entity) < 16.0 -> setPos(entity.pos)` |
| cloud radius 3.0, duration 600, radiusPerTick `(7-3)/600` | `area_effect_cloud.rs:59-61,129-130,142-147` | `setRadius(3)`, `setDuration(600)`, `setRadiusPerTick((7 - radius)/duration)` |
| cloud particle = `dragon_breath` | `area_effect_cloud.rs:217-223` | `setParticle(ParticleTypes.DRAGON_BREATH)` |
| cloud effect `HARM` duration 1, amplifier 1 | `area_effect_cloud.rs:134-141` | `addEffect(new MobEffectInstance(MobEffects.HARM, 1, 1))` |
| cloud owner only when the owner is a `LivingEntity` | `dragon_fireball.rs:146-149` | `if (owner instanceof LivingEntity) cloud.setOwner(...)` |
| no `setWaitTime` / `setRadiusOnUse` overrides (defaults 20 / 0.0) | `area_effect_cloud.rs:142-150` | dragon fireball uses `AreaEffectCloud` defaults |
| level event 2006 at `blockPosition()`, power `isSilent() ? -1 : 1` | `dragon_fireball.rs:155-160` | `levelEvent(2006, blockPosition(), isSilent() ? -1 : 1)` |
| burst then discard | `projectile/mod.rs:446-451` (generic) | `discard()` at the end of `onHit` |
| no direct-hit damage | `dragon_fireball.rs:105-113` | `DragonFireball` has no `onHitEntity` damage (unlike `LargeFireball`) |
| burst skipped when the owner is already removed | `dragon_fireball.rs:114-118` | `if (owner == null || !owner.isRemoved())` |
| inertia 0.95 / liquid 0.8, never set on fire | `dragon_fireball.rs:84-88` | `AbstractHurtingProjectile` constants; `displayFireAnimation()` false |
| drag of 0.1 along the shot direction, spawn at head minus view vector | `strafing.rs:155-187`, `dragon_fireball.rs:42-49` | `DragonStrafePlayerPhase.shootFireball`, `DragonFireball(Level, Mob, Vec3)` |
| launch sound 1017 when the dragon is not silent | `strafing.rs:171-177` | `levelEvent(1017, dragon.blockPosition(), 0)` |

## Fixed in this pass

1. **Splash position skipped the owner, spectators and dying entities.** `dragon_fireball.rs:134-144`.
   Vanilla's `getEntitiesOfClass(LivingEntity.class, box)` returns all of them, so the cloud could snap
   to the dragon itself or to a spectator; Rust only collected plain alive non-spectator mobs and
   deliberately excluded the owner. Now only the `LivingEntity` type is filtered.
2. **Non-vanilla `GameEvent.ENTITY_DAMAGE` at the burst.** `dragon_fireball.rs` (removed).
   `979aa8a83` emitted an entity-damage vibration when the cloud spawned; vanilla `DragonFireball.onHit`
   deals no damage and emits no such game event (ENTITY_DAMAGE comes from the damage path, i.e. later
   from the cloud's Instant Damage ticks, not from the burst). The generic projectile landing event
   (`projectile/mod.rs:317-323`) is unchanged.

## Remaining gaps

| # | file:line | Vanilla class | Gap | Suggested fix | Size |
|---|---|---|---|---|---|
| 1 | `crates/pumpkin-core/src/entity/projectile/mod.rs:393-452` (called from `dragon_fireball.rs:82-90`) | `AbstractHurtingProjectile.tick` | No discard when the owner is removed. Vanilla discards the projectile when the resolved owner `isRemoved()`; Pumpkin only discards on an unloaded chunk (`:419-422`) and `EntityReference::get` (`owner.rs:51-74`) hides removed entities by returning `None`, so the burst guard added in `222f05757` never fires for a dead dragon. A fireball outlives its dragon and bursts. | Add an `EntityReference` accessor that reports "uuid resolves to a removed entity" (cache the resolved `Arc`, not a `Weak`, or check `world.get_entity_by_uuid` without the `is_removed` filter), then `entity.remove(); return;` at the start of `process_tick` when it is true. Applies to every hurting projectile, as in vanilla. | M |
| 2 | `crates/pumpkin-core/src/entity/area_effect_cloud.rs:296-324` | `AreaEffectCloud.tick` | `radius_per_tick` is applied only after the `age < wait_time` early return, so dragon breath stays at radius 3.0 for the first 20 ticks. Vanilla's ordering (*verify* on decompiled `AreaEffectCloud.tick`) may advance the radius while waiting. | Move the radius update above the wait-time early return, leaving effect application and radius-on-use below it. | S |

Client-only / not testable headless: cloud particle colour and the 2006 splash particle power are sent
but rendering is untested.
