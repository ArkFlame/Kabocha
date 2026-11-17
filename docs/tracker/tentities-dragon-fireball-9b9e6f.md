# Tracker: entities / Dragon Fireball (RE-REVIEW-UNTRACKED)

Audit date: 2026-11-16 (second pass). Source changed in `a5cee62f2 feat(entity): implement dragon fireball`
and `222f05757 feat(entity): skip dragon fireball burst when owner is removed`, so the previous
assessment was re-done from scratch.

Sources reviewed:
- `crates/pumpkin-core/src/entity/projectile/dragon_fireball.rs` (whole file)
- `crates/pumpkin-core/src/entity/area_effect_cloud.rs` (`create_dragon_breath`, `tick`, particle meta)
- `crates/pumpkin-core/src/entity/projectile/mod.rs:160-200,336-452` (`HurtingProjectile::process_tick`)
- `crates/pumpkin-core/src/entity/projectile/owner.rs` (`EntityReference`)
- `crates/pumpkin-core/src/entity/boss/ender_dragon/phase/strafing.rs:151-195` (spawn site)
- vanilla classes `net.minecraft.world.entity.projectile.DragonFireball`,
  `AbstractHurtingProjectile`, `AreaEffectCloud`

No decompiled vanilla source is vendored in this worktree (`.vendor/` absent), so the comparison is
against the vanilla classes from knowledge; items marked *verify* still need a decompiled reference.

## Verified as matching vanilla

| Behaviour | Rust | Vanilla |
|---|---|---|
| splash box 4.0 x 2.0 x 4.0 around the fireball | `dragon_fireball.rs:122-126` | `getBoundingBox().inflate(4.0, 2.0, 4.0)` |
| cloud radius 3.0, duration 600, radiusPerTick `(7-3)/600` | `area_effect_cloud.rs:59-61,129-130,142-147` | `setRadius(3)`, `setDuration(600)`, `setRadiusPerTick((7 - radius)/duration)` |
| cloud particle = `dragon_breath` | `area_effect_cloud.rs:217-223` | `setParticle(ParticleTypes.DRAGON_BREATH)` |
| cloud effect Instant Damage, duration 1, amplifier 1 | `area_effect_cloud.rs:134-141` | `addEffect(new MobEffectInstance(MobEffects.HARM, 1, 1))` |
| no `setWaitTime` / `setRadiusOnUse` overrides | `area_effect_cloud.rs:145-150` | dragon fireball uses the `AreaEffectCloud` defaults (20 / 0.0); `waitTime 10` and `radiusOnUse -0.5` are lingering-potion-only |
| cloud owner only when the owner is a `LivingEntity` | `dragon_fireball.rs:149-152` | `if (owner instanceof LivingEntity) cloud.setOwner(...)` |
| cloud position snaps to the nearest candidate within 4.0 | `dragon_fireball.rs:128-147` | `distanceToSqr(entity) < 16.0 -> setPos(entity.pos)` |
| level event 2006 with `isSilent() ? -1 : 1` at `blockPosition()` | `dragon_fireball.rs:159-165` | `levelEvent(2006, blockPosition(), isSilent() ? -1 : 1)` |
| burst then discard | `projectile/mod.rs:446-451` (generic) | `discard()` at the end of `onHit` |
| no direct-hit damage | `dragon_fireball.rs:105-164` | `DragonFireball` has no `onHitEntity` damage (unlike `LargeFireball`) |
| inertia 0.95 / liquid 0.8, never set on fire | `dragon_fireball.rs:86-90` | `AbstractHurtingProjectile` constants; `displayFireAnimation()` false |
| owner and dragon parts excluded from splash | `dragon_fireball.rs:50-58,105-110` | `ownedBy(owner)` / `EnderDragonPart` handling |
| spawn at head minus view vector, `head.getY(0.5) + 0.5`, aimed at target centre, speed 0.1 | `strafing.rs:155-187` | `DragonStrafePlayerPhase.shootFireball` |
| launch sound 1017 when the dragon is not silent | `strafing.rs:171-177` | `levelEvent(1017, dragon.blockPosition(), 0)` |

## Fixed in this pass

1. **Missing `GameEvent.ENTITY_DAMAGE` vibration.** `dragon_fireball.rs:149-163`. Vanilla
   `DragonFireball.onHit` emits `gameEvent(GameEvent.ENTITY_DAMAGE, position(), Context.of(this))`
   after building the cloud and before the 2006 level event, so sculk sensors / the warden react to
   the burst. Rust emitted only the level event.

## Remaining gaps

| # | file:line | Vanilla class | Gap | Suggested fix | Size |
|---|---|---|---|---|---|
| 1 | `crates/pumpkin-core/src/entity/projectile/mod.rs:393-452` (`dragon_fireball.rs:82-91` caller) | `AbstractHurtingProjectile.tick` | No discard when the owner is removed. Vanilla ticks the projectile only while `owner == null \|\| !owner.isRemoved()`, else `discard()`. Pumpkin only discards on an unloaded chunk (`:419-422`), so a dragon fireball whose dragon is gone keeps flying and bursts on impact. The guard added in `222f05757` (`dragon_fireball.rs:116-120`) is unreachable for mob owners: `EntityReference::get` (`owner.rs:51-74`) never hands back a removed entity, it returns `None`, and `None` must keep flying (vanilla only discards when the owner resolves *and* is removed). | Add an `EntityReference`/`ProjectileOwner` accessor that distinguishes "removed" from "unresolvable" (e.g. `fn resolved_removed(&self) -> bool`), then `if owner_removed { entity.remove(); return; }` after `self.owner.tick(entity)` in `process_tick`. Affects every `AbstractHurtingProjectile` subclass (fireball, small fireball, wither skull, wind charge), which is what vanilla does. | M |
| 2 | `crates/pumpkin-core/src/entity/area_effect_cloud.rs:296-324` | `AreaEffectCloud.tick` | `radius_per_tick` is applied only after the `age < wait_time` early return. Vanilla advances the radius every server tick, entering the waiting state only to gate the particle burst / effect application. Dragon breath has `radius_per_tick > 0`, so the cloud currently stays at 3.0 for the first 20 ticks. | Move the radius block above the wait-time early return (keep effect application and radius-on-use below it). *verify* against decompiled `AreaEffectCloud.tick` first. | S |
| 3 | `crates/pumpkin-core/src/entity/projectile/dragon_fireball.rs:134-147` | `DragonFireball.onHit` | Candidate list skips non-alive and spectator entities; vanilla iterates `getEntitiesOfClass(LivingEntity.class, box)` unfiltered, so a spectator (or a still-listed dead entity) closest to the impact is chosen as the cloud position. | Drop the `is_alive()` / `is_spectator()` filters, keep the owner and `< 16.0` checks. Low gameplay impact; behaviour change is visible only in that edge case. | S |

Client-only / not testable headless: cloud particle colour and the 2006 splash particle power
(`ParticlesDragonFireballSplash`, `dragon_breath` power 1.0) are sent but rendering is untested.
