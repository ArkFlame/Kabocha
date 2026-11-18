# Final rows from docs/tracker/tentities-creeper-0de8b3.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: area_effect_cloud.rs creeper.rs creeper_ignite.rs living.rs mod.rs

- Row 3 (explodeCreeper on-death effects): STILL OPEN. Needs a cross-file effect refactor outside this unit. Vanilla `Creeper.explodeCreeper` (Creeper.java:245-256) calls `triggerOnDeathMobEffects(KILLED)` -> `MobEffectInstance.onMobRemoved` (LivingEntity.java:826-833); `OozingMobEffect`, `WeavingMobEffect`, `WindChargedMobEffect` override it to spawn slimes/a cobweb/a wind burst. Pumpkin's equivalent is `MobEffect::on_mob_death(&LivingEntity, u8, &DamageType)` in `entity/effect/mod.rs` (outside unit files), and it is only called from the normal damage/death path (`entity/living.rs:3050-3065`), which `explode()` bypasses via `dead=true` + `entity.remove()`. Fix requires adding a damage-type-free removal hook to the trait + impls and calling it from `explode()`; not possible inside creeper.rs/creeper_ignite.rs/area_effect_cloud.rs alone.
Re-verify of the two STILL OPEN rows against decompiled vanilla, now reachable through the `.vendor/vanilla-decompiled` symlink (`rel-mc-backup/net/minecraft/`). No source file in this unit required a change: both rows were already resolved in the checkout.
Re-verified all three STILL OPEN rows against the decompiled vanilla tree; no source file needed a change this pass.
Resume check after server restart: all three STILL OPEN rows are already resolved in the checkout; no source file needed a change this pass, so no rustfmt run.

## Result

- Row 3 (explodeCreeper on-death effects): VERIFIED FIXED. `Creeper.explodeCreeper` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Creeper.java:245-264`) runs `dead=true` -> explode -> `spawnLingeringCloud` (Creeper.java:266-284, radius 2.5, radiusOnUse -0.5, wait 10, duration 300, potionDurationScale 0.25, radiusPerTick -radius/duration) -> `triggerOnDeathMobEffects(KILLED)` -> `discard`; `LivingEntity.triggerOnDeathMobEffects` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/LivingEntity.java:826-833`) delegates to `MobEffectInstance.onMobRemoved`. `CreeperEntity::explode` (`crates/pumpkin-core/src/entity/mob/creeper.rs:121-196`) fires `MobEffect::on_mob_removed` (trait hook in `entity/effect/mod.rs:47`, impls in `oozing.rs`/`weaving.rs`/`wind_charged.rs`) for each active effect after spawning the cloud and before `entity.remove()`, which only calls `world.remove_entity`, so no double activation. Cloud built by `AreaEffectCloudEntity::create` (`area_effect_cloud.rs:93-125`) with matching geometry. Effect counts match `MobEffects.java:120-123` (OOZING 2 slimes, WEAVING 2-3 cobwebs, WIND_CHARGED 3-5 gust). No code change this pass; `reason == KILLED` guard left unobservable (fires only on KILLED-equivalent paths).

No pure unit-test seam exists (all rows fire through World/entity/loot/effect state) and no `.rs` file was touched, so no rustfmt run.

Status: PASS fixed=1 open=0 files=tfin-tentities-creeper-0de8b3-0.md
