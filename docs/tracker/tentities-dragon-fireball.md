# Tracker: entities / Dragon Fireball (RE-REVIEW-UNTRACKED)

Audit date: 2026-11-16. Reviewed source changed in `a5cee62f2 feat(entity): implement dragon fireball`
(previous row assessed the old stub, so this is a fresh review).

Sources reviewed:
- `crates/pumpkin-core/src/entity/projectile/dragon_fireball.rs` (whole file)
- `crates/pumpkin-core/src/entity/area_effect_cloud.rs` (`create_dragon_breath`, `tick`)
- `crates/pumpkin-core/src/entity/boss/ender_dragon/phase/strafing.rs:170-195` (spawn site)
- `crates/pumpkin-core/src/entity/projectile/mod.rs:391-452` (`HurtingProjectile::process_tick`)
- vanilla class `net.minecraft.world.entity.projectile.DragonFireball` (+ `AreaEffectCloud`)

No decompiled vanilla source is vendored in this worktree (`.vendor/` absent, nothing under `../`), so
values below are compared against the vanilla class from knowledge; items marked *verify* need a
decompiled reference before changing.

## Verified as matching vanilla

| Behaviour | Rust | Vanilla |
|---|---|---|
| splash box 4.0 x 2.0 x 4.0 | `dragon_fireball.rs:122-126` | `getBoundingBox().inflate(4.0, 2.0, 4.0)` |
| cloud radius 3.0, duration 600, radiusPerTick `(7-3)/600` | `area_effect_cloud.rs:59-61,129-130,142-147` | `setRadius(3)`, `setDuration(600)`, `setRadiusPerTick((7 - radius)/duration)` |
| cloud particle = `dragon_breath` | `area_effect_cloud.rs:224-231` | `setParticle(ParticleTypes.DRAGON_BREATH)` |
| cloud owner only when owner is a `LivingEntity` | `dragon_fireball.rs:149-152` | `if (owner instanceof LivingEntity) cloud.setOwner(...)` |
| level event 2006 with `isSilent() ? -1 : 1` | `dragon_fireball.rs:157-162` | `levelEvent(2006, blockPosition(), isSilent() ? -1 : 1)` |
| projectile removed after impact | `projectile/mod.rs:446-451` (generic) | `discard()` at end of `onHit` |
| inertia 0.95 / liquid 0.8, no burning | `dragon_fireball.rs:86-90` | `AbstractHurtingProjectile` constants |
| owner / dragon-part filtered out of splash | `dragon_fireball.rs:50-58,105-110` | owner check + `EnderDragonPart` in `canHitEntity` |

## Fixed in this pass

1. **Missing owner-removed guard.** `dragon_fireball.rs:114-120`. Vanilla wraps the whole burst in
   `Entity owner = getOwner(); if (owner == null || !owner.isRemoved())`, so a fireball whose dragon is
   already gone leaves no cloud and no splash particles. Rust spawned the burst unconditionally.

## Remaining gaps (not changed: needs decompiled confirmation)

| # | file:line | Vanilla class | Gap | Suggested fix | Size | Verify |
|---|---|---|---|---|---|---|
| 1 | `area_effect_cloud.rs:150` (used via `dragon_fireball.rs:155`) | `DragonFireball.onHit` | cloud `wait_time` is 20; vanilla dragon-fireball cloud uses `setWaitTime(10)` | pass `10` into `create_dragon_breath` | S | yes |
| 2 | `area_effect_cloud.rs:134-141` | `DragonFireball.onHit` / `Potions.HARMING` | instant damage amplifier is `1`; vanilla `Potions.HARMING` is amplifier `0`, and the sibling dragon-breath cloud at `phase/sit_breathing.rs:57-65` uses `0` | set amplifier to `0` (one value), confirms internal consistency | S | yes |
| 3 | `area_effect_cloud.rs:296` | `AreaEffectCloud.tick` | radius/radiusPerTick only applied after the wait period; vanilla advances radius every tick | move the radius block above the `age < wait_time` early return | S | yes |
| 4 | `area_effect_cloud.rs:142-148` | `DragonFireball.onHit` | dragon cloud sets `radius_on_use = 0.0`; vanilla may use `setRadiusOnUse(-0.5F)` (lingering-potion default) | pass `-0.5` if confirmed | S | yes |
| 5 | `dragon_fireball.rs:105-164` | `HurtingProjectile.onHitEntity` | no direct-hit damage; if vanilla's `DragonFireball` inherits the 6.0 fireball-damage `onHitEntity`, a direct hit should damage | add `damage_with_context(6.0, FIREBALL/UNATTRIBUTED_FIREBALL)` before the cloud, mirroring `projectile/fireball.rs:169-183` | S | yes |
| 6 | `dragon_fireball.rs:128-147` | `DragonFireball.onHit` | cloud position is "nearest living entity within a 4.0 sphere, skipping spectators"; vanilla picks from `getEntitiesOfClass(box)` without spectator/alive filters and (believed) uses the first element | re-check the vanilla selection loop; drop the extra filters if it is a plain first-element pick | S | yes |

Client-only / not testable headless: particle appearance of the cloud (`ParticlesDragonFireballSplash`,
`dragon_breath` particle power 1.0) — noted, do not block.
