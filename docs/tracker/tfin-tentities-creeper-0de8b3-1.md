# Final rows from docs/tracker/tentities-creeper-0de8b3.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

Second-pass re-review against `.vendor/vanilla-decompiled/net/minecraft/`. The three STILL OPEN tracker rows remain verified fixed in the checkout, so no change was needed for them: row 2 (`dropped_skulls` one-shot guard), row 3 (`on_mob_removed` fired in `CreeperEntity::explode`), row 4 (`SwellGoal.canUse` aliveness guard matches vanilla).
Third-pass verification against `.vendor/vanilla-decompiled/net/minecraft/`. All three STILL OPEN rows are already resolved in the checkout; no source file needed a change and no `rustfmt` run was required.
THIRD PASS verification against `.vendor/vanilla-decompiled/net/minecraft/`. All three original STILL OPEN rows are already resolved in the checkout; no source file needed a change, so no rustfmt run was required. The stale `Remaining gaps` table above has been updated to final statuses.

## Result

FINAL PASS verification against `.vendor/vanilla-decompiled/net/minecraft/`. Every STILL OPEN row ends resolved; no source file needed a change this pass, so no rustfmt run.

- Row 2 (charged-creeper heads): VERIFIED FIXED. `Creeper.killedEntityPreEvent` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Creeper.java:172-182`) guards with `shouldDropLoot && isPowered && !droppedSkulls` and drops from `BuiltInLootTables.CHARGED_CREEPER` (`charged_creeper/root`). `CreeperEntity::killed_entity` (`creeper.rs:393-427`) matches: one-shot `dropped_skulls` guard, `minecraft:charged_creeper/root` lookup, and the flag stored only when a stack actually drops (vanilla sets it inside the loot callback). Old data blocker is stale: `CHARGED_CREEPER_ROOT` is generated and resolved by `World::get_loot_table`.
- Row 3 (explodeCreeper ordering): VERIFIED FIXED. `Creeper.explodeCreeper` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Creeper.java:245-264`) runs `dead=true` -> explode -> `spawnLingeringCloud` -> `triggerOnDeathMobEffects(KILLED)` -> `discard`; `spawnLingeringCloud` (Creeper.java:266-284) uses radius 2.5, radiusOnUse -0.5, wait 10, duration 300, 0.25 potion scale, radiusPerTick -radius/duration; `LivingEntity.triggerOnDeathMobEffects` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/LivingEntity.java:826-833`) delegates to `MobEffectInstance.onMobRemoved`. `CreeperEntity::explode` (`creeper.rs:121-196`) calls `MobEffect::on_mob_removed` for each active effect (`creeper.rs:191`) after the cloud and before `entity.remove()`; trait hook in `entity/effect/mod.rs` with impls in `oozing.rs`/`weaving.rs`/`wind_charged.rs`. `Entity::remove` fires no effect hooks, so no double activation.
- Row 4 (SwellGoal aliveness guard): VERIFIED, NO CHANGE. `SwellGoal.canUse` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/goal/SwellGoal.java:18-21`) is `getSwellDir() > 0 || (target != null && !target.isDeadOrDying() && distanceToSqr < 9.0)` and `tick` (SwellGoal.java:42) checks `!isDeadOrDying()`. The `is_alive() && !is_dead_or_dying()` guard in `creeper_ignite.rs:46-47,77` is correct; the tracker premise (vanilla has no guard) is false.

No pure unit-test seam (all rows fire through World/entity/loot/effect state) and no `.rs` file was touched, so no rustfmt was needed.

Status: PASS fixed=0 open=0 files=tfin-tentities-creeper-0de8b3-1.md
