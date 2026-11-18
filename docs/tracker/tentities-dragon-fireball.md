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

## h6 pass 2 status
- Rows 1-6 (S): STILL OPEN, not attempted this pass. Cloud wait time, potion amplifier, radius-per-tick timing, radius_on_use, direct-hit damage and nearest-target filter all need decompiled DragonFireball/AreaEffectCloud confirmation before a change; numerics from memory are not reliable enough to land.

## Result

Re-review against decompiled vanilla under .vendor/vanilla-decompiled/net/minecraft/ (now vendored).

- Row 1 (cloud wait time) - FIXED. DragonFireball.onHit never calls setWaitTime
  (hurtingprojectile/DragonFireball.java:31-64); AreaEffectCloud.waitTime defaults to 20
  (world/entity/AreaEffectCloud.java:53). Pumpkin's create_dragon_breath used 10, making the
  cloud active 10 ticks too early. Changed area_effect_cloud.rs:157 to Mutex::new(20) and replaced
  the misleading Potions.HARMING / 10-tick comment.
- Row 2 (potion amplifier) - no change. Vanilla uses new MobEffectInstance(INSTANT_DAMAGE, 1, 1)
  (DragonFireball.java:46): amplifier 1, not 0. Pumpkin already stores amplifier 1. The doc's
  Potions.HARMING amplifier-0 premise does not apply to this class. The separate sitting-flame
  cloud uses new MobEffectInstance(INSTANT_DAMAGE)
  (boss/enderdragon/phases/DragonSittingFlamingPhase.java:89), amplifier 0, so sit_breathing.rs
  storing 0 is also correct. Both match vanilla.
- Row 3 (radius per tick timing) - no change. Vanilla grows radius only inside if (!shouldWait)
  (AreaEffectCloud.java:205-220), i.e. after the wait period. Pumpkin's if age < wait_time return
  (area_effect_cloud.rs:303) matches. Doc premise was inverted.
- Row 4 (radius on use) - no change. Vanilla DragonFireball never calls setRadiusOnUse, so the
  default 0.0 applies (AreaEffectCloud.java:56); Pumpkin uses 0.0 (area_effect_cloud.rs:155).
- Row 5 (direct-hit damage) - no change. Projectile.onHitEntity is empty
  (projectile/Projectile.java:525-526) and neither AbstractHurtingProjectile nor DragonFireball
  overrides it, so a direct hit deals no damage; the cloud's INSTANT_DAMAGE is the only damage.
  Pumpkin matches. Doc's 'inherits the 6.0 fireball damage' premise is false.
- Row 6 (nearest-target filter) - no change. Vanilla loops getEntitiesOfClass(LivingEntity.class, box)
  and picks the first entity with dist < 16.0 (DragonFireball.java:47-55), no spectator/alive filter.
  Pumpkin (dragon_fireball.rs:134-145) filters only on get_living_entity() and breaks on the first
  in-range candidate. Already matching.

Files touched: crates/pumpkin-core/src/entity/area_effect_cloud.rs only. No pure unit-test seam:
create_dragon_breath requires a live Entity/World, and a bare constant assertion would be a
discouraged constant-mirroring test. rustfmt --edition 2024 run on the touched file (exit 0).

## Result (pass 3 - full decompiled verification)

Re-verified every STILL OPEN row against .vendor/vanilla-decompiled/net/minecraft/ (symlink resolved).

- Row 1 (cloud wait time) - no change. DragonFireball.onHit never calls setWaitTime
  (hurtingprojectile/DragonFireball.java:31-64), so AreaEffectCloud's 20-tick default applies
  (world/entity/AreaEffectCloud.java:53). area_effect_cloud.rs:157 already uses 20. The doc's
  "pass 10" suggestion is wrong.
- Row 2 (potion amplifier) - no change. Vanilla uses new MobEffectInstance(INSTANT_DAMAGE, 1, 1)
  (DragonFireball.java:46) = duration 1, amplifier 1. create_dragon_breath stores
  EffectEntry (INSTANT_DAMAGE, 0, 1, ..) where field 3 is the u8 amplifier, so amplifier 1 is
  correct; the Potions.HARMING amplifier-0 premise does not apply. sit_breathing.rs stores
  amplifier 0, matching new MobEffectInstance(INSTANT_DAMAGE) in
  DragonSittingFlamingPhase.java:89.
- Row 3 (radius per tick timing) - FIXED. Vanilla grows the radius only inside the
  `if (!shouldWait)` block of AreaEffectCloud.serverTick (AreaEffectCloud.java:204-220);
  it does not advance during the wait period. Pumpkin advanced the radius block BEFORE the
  `if age < wait_time { return; }` early exit (was area_effect_cloud.rs:303-332). Moved the
  radius block below the wait early-return so the radius stays at its initial value until the
  grace period ends. This corrects the prior Result note, which wrongly claimed Pumpkin already
  matched.
- Row 4 (radius on use) - no change. Vanilla DragonFireball never calls setRadiusOnUse, so the
  default 0.0 applies (AreaEffectCloud.java:56); create_dragon_breath uses 0.0. Matches.
- Row 5 (direct-hit damage) - no change. Projectile.onHitEntity is empty
  (projectile/Projectile.java:525-526) and neither AbstractHurtingProjectile nor DragonFireball
  overrides it, so a direct hit deals no damage; the cloud's INSTANT_DAMAGE is the only damage.
  The 6.0 fireball damage belongs to SmallFireball/Fireball (overridden onHitEntity), not here.
- Row 6 (nearest-target filter) - no change. Vanilla loops getEntitiesOfClass(LivingEntity.class,
  box) and picks the first entity with dist < 16.0, no spectator/alive filter
  (DragonFireball.java:47-55). dragon_fireball.rs:134-145 filters only on get_living_entity()
  and breaks on the first in-range candidate. Matches.

Files touched: crates/pumpkin-core/src/entity/area_effect_cloud.rs only. No pure unit-test seam:
the tick ordering requires a live Entity/World; a constant assertion would be a discouraged
constant-mirroring test. rustfmt --edition 2024 run on the touched file (exit 0, no diffs).

## Result (pass 4 - independent decompiled re-verification)

Re-verified all 6 STILL OPEN rows against .vendor/vanilla-decompiled/net/minecraft/. Prior passes'
fixes are present and correct, so rows 1-6 needed no further change:

- Row 1 (wait time) - matches. DragonFireball.onHit never calls setWaitTime
  (hurtingprojectile/DragonFireball.java:31-64); AreaEffectCloud.waitTime defaults to 20
  (world/entity/AreaEffectCloud.java:53). create_dragon_breath stores 20.
- Row 2 (potion amplifier) - matches. Vanilla uses new MobEffectInstance(INSTANT_DAMAGE, 1, 1)
  (DragonFireball.java:46) = amplifier 1. create_dragon_breath stores amplifier 1. sit_breathing.rs
  stores amplifier 0, matching new MobEffectInstance(INSTANT_DAMAGE) in
  DragonSittingFlamingPhase.java:89.
- Row 3 (radius per tick timing) - matches. Vanilla grows radius only inside the `if (!shouldWait)`
  block of serverTick (AreaEffectCloud.java:204-220). The radius block sits below the
  `if age < wait_time { return; }` early return (area_effect_cloud.rs:303-332).
- Row 4 (radius on use) - matches. Vanilla DragonFireball never calls setRadiusOnUse, so the
  default 0.0 applies (AreaEffectCloud.java:56); create_dragon_breath uses 0.0.
- Row 5 (direct-hit damage) - matches. Projectile.onHitEntity is empty
  (projectile/Projectile.java:525-526); AbstractHurtingProjectile and DragonFireball do not
  override it, so a direct hit deals no damage. Pumpkin's cloud effect is the only damage.
- Row 6 (nearest-target filter) - matches. Vanilla loops getEntitiesOfClass(LivingEntity.class,
  box) and picks the first entity with dist < 16.0, no spectator/alive filter
  (DragonFireball.java:47-55). dragon_fireball.rs:134-145 filters on get_living_entity() and breaks
  on the first in-range candidate.

- Extra parity gap (found during verification) - FIXED. Vanilla DragonFireball.onHit calls
  `cloud.setPotionDurationScale(0.25F)` (DragonFireball.java:45), but create_dragon_breath stored
  1.0. Changed area_effect_cloud.rs potion_duration_scale to 0.25 and updated the nearby comment.
  No observable behaviour change here: the cloud's only effect is INSTANT_DAMAGE, which is applied
  once via apply_instant_effect and ignores duration/potion_duration_scale. This aligns the stored
  field with vanilla and with the sibling lingering/creeper clouds.

Files touched: crates/pumpkin-core/src/entity/area_effect_cloud.rs only (one value + comment).
No pure unit-test seam: the change is a config value read only by a live Entity tick, and a bare
constant assertion would be a discouraged constant-mirroring test. Worker cannot compile/run
rustfmt; orchestrator compiles once. Out of scope (not in these 6 rows, left for a separate
tracker row): sit_breathing.rs cloud parameters differ from vanilla DragonSittingFlamingPhase
(radius 3.0 vs 5.0, duration 600 vs 200, shrinking radiusPerTick and radiusOnUse 0.5 vs defaults).
