# Parity rows for crates/pumpkin-core/src/entity/area_effect_cloud.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/tentities-dragon-fireball** [partial/VERIFIED] own audit: 0 open, 4 fixed, 3 verified, 0 blocked-data (docs/tracker/tentities-dragon-fireball.md) docs/tracker/tentities-dragon-fireball.md

## Result (third pass - all rows closed)

Unit row `own/tentities-dragon-fireball` - VERIFIED (matches vanilla). No code change needed this pass:
every prior FIXED value is present in `crates/pumpkin-core/src/entity/area_effect_cloud.rs` and all six
sub-rows match the vendored decompiled source under `.vendor/vanilla-decompiled/net/minecraft/`.

- Row 1 (cloud wait time) - VERIFIED. `DragonFireball.onHit` never calls setWaitTime
  (`world/entity/projectile/hurtingprojectile/DragonFireball.java:31-64`), so `AreaEffectCloud.waitTime`
  defaults to 20 (`world/entity/AreaEffectCloud.java:45,53`). `create_dragon_breath` stores
  `wait_time: Mutex::new(20)` (area_effect_cloud.rs:158).
- Row 2 (potion amplifier) - VERIFIED. Vanilla uses `new MobEffectInstance(INSTANT_DAMAGE, 1, 1)`
  = duration 1, amplifier 1 (`DragonFireball.java:46`). `create_dragon_breath` stores amplifier 1
  (area_effect_cloud.rs:144). The doc Potions.HARMING amplifier-0 premise does not apply.
- Row 3 (radius per tick timing) - VERIFIED. Vanilla grows the radius only inside `if (!shouldWait)`
  of `AreaEffectCloud.serverTick` (`AreaEffectCloud.java:204-220`). The radius block sits below the
  `if age < wait_time { return; }` early return (area_effect_cloud.rs:336-365). FIXED in pass 3.
- Row 4 (radius on use) - VERIFIED. DragonFireball never calls setRadiusOnUse/setDurationOnUse, so the
  defaults 0.0 and 0 apply (`AreaEffectCloud.java:55-56`); create_dragon_breath stores both
  (area_effect_cloud.rs:155-156).
- Row 5 (direct-hit damage) - VERIFIED. `Projectile.onHitEntity` is empty and neither
  AbstractHurtingProjectile nor DragonFireball overrides it, so a direct hit deals no damage; the
  cloud INSTANT_DAMAGE is the only damage (`world/entity/projectile/Projectile.java:525`).
- Row 6 (nearest-target filter) - VERIFIED. Vanilla loops `getEntitiesOfClass(LivingEntity.class, box)`
  and picks the first in-range entity with `dist < 16.0`, no spectator/alive filter
  (`DragonFireball.java:47-55`). dragon_fireball.rs breaks on the first in-range living candidate.
- Extra (potion_duration_scale) - VERIFIED. Vanilla calls setPotionDurationScale(0.25F)
  (`DragonFireball.java:45`); create_dragon_breath stores 0.25 (area_effect_cloud.rs:157). FIXED pass 4.
- Extra (sitting flame cloud) - VERIFIED. Vanilla sitting flame uses radius 5, duration 200, default
  radiusPerTick/radiusOnUse/durationOnUse (0.0/0.0/0), potionDurationScale 0.25 and plain
  INSTANT_DAMAGE amplifier 0 (`boss/enderdragon/phases/DragonSittingFlamingPhase.java:65-89`).
  `create_sitting_flame` matches (area_effect_cloud.rs:164-193). FIXED pass 5.

Out of scope (separate gap, not a row here): sitting-flame cloud position uses head+look*2.5 with a
down-to-ground scan in vanilla, while Pumpkin uses a body/yaw offset; it needs a head-position/ground
API, not a cloud parameter or generated data.

No pure unit-test seam: building a cloud or exercising tick needs a live Entity/World, and a bare
constant assertion would be a discouraged constant-mirroring test. No source file changed this pass;
nothing to rustfmt. No unwrap/expect/panic, no em dashes.
