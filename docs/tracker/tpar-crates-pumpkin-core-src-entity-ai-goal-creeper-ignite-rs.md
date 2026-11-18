# Parity rows for crates/pumpkin-core/src/entity/ai/goal/creeper_ignite.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/tentities-creeper-0de8b3** [partial/OWN-OPEN] own audit: 10 open, 12 fixed, 8 verified, 1 blocked-data (docs/tracker/tentities-creeper-0de8b3.md) docs/tracker/tentities-creeper-0de8b3.md
- **own/tentities-creeper** [partial/OWN-OPEN] own audit: 6 open, 7 fixed, 0 verified, 1 blocked-data (docs/tracker/tentities-creeper.md) docs/tracker/tentities-creeper.md

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
