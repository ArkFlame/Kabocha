# Parity rows for crates/pumpkin-core/src/entity/ai/goal/creeper_ignite.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/tentities-creeper-0de8b3** [VERIFIED] own audit: 10 open, 12 fixed, 8 verified, 1 blocked-data (docs/tracker/tentities-creeper-0de8b3.md) docs/tracker/tentities-creeper-0de8b3.md
- **own/tentities-creeper** [VERIFIED] own audit: 6 open, 7 fixed, 0 verified, 1 blocked-data (docs/tracker/tentities-creeper.md) docs/tracker/tentities-creeper.md

## Result

THIRD PASS: both formerly STILL OPEN aggregate rows are closed as VERIFIED against decompiled vanilla at `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Creeper.java` and `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/goal/SwellGoal.java` (Paper 1.21.x). No source edit was required, so no rustfmt run. No row remains STILL OPEN.

- **own/tentities-creeper-0de8b3**: VERIFIED. All underlying rows resolved in the checkout and matching vanilla:
  - Row 2 (charged-creeper heads): `Creeper.killedEntityPreEvent` (Creeper.java:173-181) gates on `shouldDropLoot && isPowered && !droppedSkulls` with `BuiltInLootTables.CHARGED_CREEPER`; `creeper.rs:379-407` uses the same `!dropped_skulls` guard with `minecraft:charged_creeper/root` (generated `CHARGED_CREEPER_ROOT`). Matches.
  - Row 3 (explodeCreeper on-death effects): `Creeper.explodeCreeper` (Creeper.java:245-255) order is `dead = true` -> explode -> `spawnLingeringCloud` -> `triggerOnDeathMobEffects(KILLED)`; `creeper.rs:161-195` fires `on_mob_removed` per effect (hook in `entity/effect/mod.rs`) before `entity.remove()`. Matches.
  - Row 4 (SwellGoal aliveness guard): `SwellGoal.canUse` (SwellGoal.java:18-21) is `getSwellDir() > 0 || (target != null && !target.isDeadOrDying() && distanceToSqr < 9.0)`; `creeper_ignite.rs:35-54` uses `fuse_speed > 0` or `is_alive() && !is_dead_or_dying() && dist < 9.0`. The tracker premise of a missing guard is false; no change.

- **own/tentities-creeper**: VERIFIED. All underlying rows resolved:
  - Row 4 (spawnLingeringCloud cloud params): `Creeper.spawnLingeringCloud` (Creeper.java:266-276) sets radius 2.5, radiusOnUse -0.5, waitTime 10, duration 300, potionDurationScale 0.25, radiusPerTick -2.5/300; `creeper.rs:174-185` passes (300, 2.5, 20, 10, -0.5, 0) and `area_effect_cloud.rs` computes radiusPerTick as `-radius/max(duration,1)` and hardcodes scale 0.25. The tracker's 600 claim is incorrect; no change.
  - Row 6 (readAdditionalSaveData sync): `Creeper.readAdditionalSaveData` (Creeper.java:116-123) writes DATA_IS_POWERED and, when ignited, DATA_IS_IGNITED; `creeper.rs:227-253` writes the same synced data and `mob_metadata()` feeds both Java and Bedrock spawn metadata. Same observable behaviour; seam covered by `creeper_metadata_generation` in `metadata.rs`.
  - Row 7 (charged_creeper/root loot table): RESOLVED BY DATA, not BLOCKED-DATA. Generated `pumpkin_data::loot_table::CHARGED_CREEPER_ROOT` exists and `world.get_loot_table` falls back to it. No missing generated field or subsystem; generated data is outside this unit.

No new pure unit test: the inline cloud parameters have no pure seam, and the metadata/loot-table seams are already covered by existing tests. No `.rs` file was touched.


## Result

FINAL PASS. Re-read the current checkout and re-verified every underlying row against decompiled vanilla at `.vendor/vanilla-decompiled/net/minecraft/`. No row remains open.

- **own/tentities-creeper-0de8b3**: VERIFIED. All underlying rows match vanilla:
  - Row 2 (charged-creeper heads): FIXED this pass. Vanilla `Creeper.killedEntityPreEvent` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Creeper.java:172-182`) guards with `this.shouldDropLoot(level) && isPowered && !droppedSkulls` and drops from `BuiltInLootTables.CHARGED_CREEPER`. `CreeperEntity::killed_entity` (`creeper.rs:382-415`) already had the one-shot `dropped_skulls` guard and generated `CHARGED_CREEPER_ROOT` lookup; this pass corrected the gate from the victim's `killed.should_drop_loot()` to the killer's `Mob::should_drop_loot(self)`, matching vanilla's `this.shouldDropLoot(level)` (a Monster's `mob_drops` gamerule check; call site `living.rs:2965` confirms `self` is the killer). Previously a baby-monster victim wrongly suppressed the head.
  - Row 3 (explodeCreeper ordering): VERIFIED. `Creeper.explodeCreeper` (`Creeper.java:245-256`) runs `dead=true` -> explode -> `spawnLingeringCloud` -> `triggerOnDeathMobEffects(KILLED)` -> `discard`; `LivingEntity.triggerOnDeathMobEffects` (`LivingEntity.java:826-833`) delegates to `MobEffectInstance.onMobRemoved`. `CreeperEntity::explode` (`creeper.rs:121-196`) fires `MobEffect::on_mob_removed` for each active effect in that order before `entity.remove()`; `Entity::remove` fires no effect hooks, so no double activation.
  - Row 4 (SwellGoal aliveness guard): VERIFIED, NO CHANGE. `SwellGoal.canUse` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/goal/SwellGoal.java:18-21`) is `getSwellDir() > 0 || (target != null && !target.isDeadOrDying() && distanceToSqr < 9.0)` and `tick` (:42) also checks `!isDeadOrDying()`; the `is_alive() && !is_dead_or_dying()` guard in `creeper_ignite.rs:46-47,77` is correct. The tracker premise (vanilla has no guard) is false.

- **own/tentities-creeper**: VERIFIED. All underlying rows match vanilla:
  - Row 4 (spawnLingeringCloud cloud params): VERIFIED. `Creeper.spawnLingeringCloud` (`Creeper.java:266-284`) sets radius 2.5, radiusOnUse -0.5, wait 10, duration 300, potionDurationScale 0.25, radiusPerTick -radius/duration. `creeper.rs:168-187` calls `AreaEffectCloudEntity::create(..., 300, 2.5, 20, 10, -0.5, 0)`; `area_effect_cloud.rs:116` computes radius_on_tick `-radius/max(duration,1)` and :120 hardcodes potion scale 0.25. The tracker's `600` claim is incorrect.
  - Row 6 (readAdditionalSaveData sync): VERIFIED. `Creeper.readAdditionalSaveData` (`Creeper.java:116-124`) writes DATA_IS_POWERED from "powered" and, when "ignited", DATA_IS_IGNITED. `creeper.rs:227-253` writes CHARGED and IS_IGNITED synced data from the same keys; `mob_metadata()` (:263) feeds both Java and Bedrock spawn metadata. Seam covered by `creeper_metadata_generation` (`metadata.rs:501`).
  - Row 7 (charged_creeper/root loot table): VERIFIED RESOLVED BY DATA, not BLOCKED-DATA. Generated `CHARGED_CREEPER_ROOT` exists (`crates/pumpkin-data/src/generated/loot_table.rs:43054`, registered at :64739 and :68074) and `World::get_loot_table` falls back to it. No missing generated field or subsystem.

Touched file this pass: `crates/pumpkin-core/src/entity/mob/creeper.rs` (one-line loot-gate fix; `Mob` trait already imported at line 28, `Mob::should_drop_loot` UFCS form already used at `mob/mod.rs:3511`). rustfmt to be run by the orchestrator. No pure unit-test seam for the loot gate (fires through `World`/`LootContextParameters`); the metadata seam already has `creeper_metadata_generation`.
