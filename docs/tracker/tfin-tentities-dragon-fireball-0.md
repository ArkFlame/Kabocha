# Final rows from docs/tracker/tentities-dragon-fireball.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- Rows 1-6 (S): STILL OPEN, not attempted this pass. Cloud wait time, potion amplifier, radius-per-tick timing, radius_on_use, direct-hit damage and nearest-target filter all need decompiled DragonFireball/AreaEffectCloud confirmation before a change; numerics from memory are not reliable enough to land.
Re-verified every STILL OPEN row against .vendor/vanilla-decompiled/net/minecraft/ (symlink resolved).
Re-verified all 6 STILL OPEN rows against .vendor/vanilla-decompiled/net/minecraft/. Prior passes'

## Result
FINAL PASS results. Every STILL OPEN row re-verified against decompiled vanilla under
.vendor/vanilla-decompiled/net/minecraft/. All six rows were already brought into parity by earlier
passes; no further source change was needed this pass. The fixes from those passes are present in
the current checkout (verified by re-reading the files): wait_time 20 and potion_duration_scale 0.25
(area_effect_cloud.rs:157-158), radius-growth block below the wait early return
(area_effect_cloud.rs:336-365), and create_sitting_flame (area_effect_cloud.rs:164).

- Row 1 (cloud wait time) - VERIFIED. DragonFireball.onHit never calls setWaitTime
  (hurtingprojectile/DragonFireball.java:31-64), so AreaEffectCloud's 20-tick default applies
  (world/entity/AreaEffectCloud.java:45,53). create_dragon_breath stores wait_time 20
  (area_effect_cloud.rs:158). The doc's "pass 10" premise is wrong.
- Row 2 (potion amplifier) - VERIFIED. Vanilla uses new MobEffectInstance(INSTANT_DAMAGE, 1, 1)
  (DragonFireball.java:46) = amplifier 1; EffectEntry field 3 is the u8 amplifier, stored as 1
  (area_effect_cloud.rs:144). The Potions.HARMING amplifier-0 premise does not apply. Sitting flame
  stores amplifier 0, matching new MobEffectInstance(INSTANT_DAMAGE)
  (DragonSittingFlamingPhase.java:89).
- Row 3 (radius per tick timing) - VERIFIED. Vanilla grows the radius only inside the `if
  (!shouldWait)` block of serverTick (world/entity/AreaEffectCloud.java:210-220). Pumpkin's radius
  block sits below the `if age < wait_time { return; }` early return
  (area_effect_cloud.rs:336-365), so the radius stays at its initial value during the wait.
- Row 4 (radius on use) - VERIFIED. Vanilla DragonFireball never calls setRadiusOnUse, so the 0.0
  default applies (world/entity/AreaEffectCloud.java:36,56); create_dragon_breath stores 0.0
  (area_effect_cloud.rs:155).
- Row 5 (direct-hit damage) - VERIFIED. DragonFireball.onHit contains no direct damage call
  (hurtingprojectile/DragonFireball.java:31-64); the cloud's INSTANT_DAMAGE is the only damage.
  Pumpkin's on_hit does no damage_with_context (dragon_fireball.rs:105-156). The 6.0 fireball damage
  belongs to Fireball/SmallFireball, not DragonFireball.
- Row 6 (nearest-target filter) - VERIFIED. Vanilla loops getEntitiesOfClass(LivingEntity.class,
  box) and picks the first entity with dist < 16.0, no spectator/alive filter
  (DragonFireball.java:35,47-55). Pumpkin filters only on get_living_entity() and breaks on the
  first in-range candidate (dragon_fireball.rs:128-139).

No code changes this pass (all rows already in parity). No BLOCKED-DATA rows: no row needs a
generated pumpkin-data field or a missing subsystem. No pure unit-test seam: the behaviours are
cloud constants read by a live Entity tick and a nearest-entity scan needing a live World; a bare
constant assertion would be a discouraged constant-mirroring test. rustfmt not run by worker (no
shell); touched no source files this pass, so no formatting needed. Worker ran no cargo/git/shell.

## Result (batch x_12, CODER)
- Rows 1-6 re-checked against `.vendor/vanilla-decompiled/net/minecraft/world/entity/projectile/hurtingprojectile/DragonFireball.java:35-46` (`setRadius(3.0F)`, `setDuration(600)`, `setRadiusPerTick((7-3)/600)`, `setPotionDurationScale(0.25F)`, `addEffect(INSTANT_DAMAGE, 1, 1)`, no `setWaitTime`/`setRadiusOnUse`, `getEntitiesOfClass` first-hit loop).
- Rust: `crates/pumpkin-core/src/entity/area_effect_cloud.rs:130-159` (`create_dragon_breath`: radius 3.0, end 7.0, duration 600, per-tick radius, amplifier 1, scale 0.25, wait 20, radius_on_use 0.0), `crates/pumpkin-core/src/entity/projectile/dragon_fireball.rs:105-156` (`on_hit` first living candidate in range, cloud at target).
- All six rows: VERIFIED. No Rust edit this pass. Nearest-target squared-range constant not separately re-read.
