# Tracker: entities / Creeper (RE-REVIEW)

Source: `crates/pumpkin-core/src/entity/mob/creeper.rs`, `crates/pumpkin-core/src/entity/ai/goal/creeper_ignite.rs`.
Vanilla reference class: `net.minecraft.world.entity.monster.Creeper` / `CreeperSwellGoal` (1.21.x).
No `.vendor/` decompiled tree in this worktree, so items marked *verify* need the decompiled class before editing.

## Fixed in this pass

| # | file:line (before) | Gap vs vanilla | Fix |
|---|---|---|---|
| 1 | creeper.rs:94-95 | Target-selector order was nearest-player (1) then `HurtByTargetGoal` (2); vanilla is `HurtByTargetGoal` 1, `NearestAttackableTargetGoal<Player>` 2 | `CREEPER_HUNTS` moved to 2, `RevengeGoal` to 1 |
| 2 | creeper.rs:334-359 | Charged-creeper victim heads gated by a one-shot `dropped_skulls` flag, so only the first victim of one explosion got a head; vanilla drops one per victim and uses the victim's `shouldDropLoot()` | Removed the flag, gate on `killed.should_drop_loot()` |
| 3 | creeper.rs:236-249 | `mob_on_lightning_strike` set the charged bit and Java synced data by hand, so Bedrock actor metadata stayed uncharged (vanilla routes through `setPowered`) | Call `set_charged(true)` |

## Remaining gaps

| # | file:line | Vanilla class / behaviour | Suggested fix | Size |
|---|---|---|---|---|
| 4 | creeper.rs:153-167 | `Creeper.spawnEffectsCloud`: `setRadius(2.5F)`, `setRadiusOnUse(-0.5F)`, `setWaitTime(10)`, `setDuration(600)`, `setRadiusPerTick(-radius/duration)` | Cloud is created with duration `300`; pass `600` and add the `radius_per_tick = -2.5/600` argument (needs an `AreaEffectCloudEntity::create` parameter; current default is `-3.0/600`). Verify constants against decompiled Creeper first | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 5 | creeper.rs:99-105 | `Creeper` swell direction is also part of the Bedrock actor metadata | `set_fuse_speed` pushes only Java `DATA_SWELL_DIR`; also `send_bedrock_actor_data` like `set_charged`/`set_ignited` do | S, FIXED f5ae2bc87 (already on master) |
| 6 | creeper.rs:199-213 | `Creeper.readAdditionalSaveData`; vanilla entity data defaults drive the client | `mob_read_nbt` updates the atomic fields but not synced data; confirm the spawn packet serialises `mob_metadata()` (it does for `CHARGED`/`IS_IGNITED`), otherwise push `set_charged`/`set_ignited` after load | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 7 | creeper.rs:328-352 | `Creeper.killedEntity` uses loot table `minecraft:charged_creeper/root` | No `assets/**/loot_table` JSON is shipped, so `get_loot_table` returns `None` and charged creepers never drop heads without a datapack; needs the vanilla data pack extracted (Extractor), not a code change | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |

## Checked and already matching vanilla

- `Creeper.tick` swell/prime-sound/explode ordering, negative swell clamp, `maxSwell` clamp on fall damage (`cause_fall_damage`).
- Goal list: swim 1, `CreeperSwellGoal` 2, ocelot/cat `AvoidEntityGoal` 3 (6 blocks, 1.0/1.2), `MeleeAttackGoal` 4 with no-op `doHurtTarget`, stroll 5, look 6 / random look 6; `CreeperSwellGoal` 3-block start, 7-block stop, line-of-sight, `requiresUpdateEveryTick`.
- Igniter interaction (tag `CREEPER_IGNITERS`, fire charge vs flint and steel sound, durability/stack decrement), explosion radius ×2 when charged, NBT keys (`powered`, `Fuse`, `ExplosionRadius`, `ignited`).

## Gap status
- 1 target-selector order: FIXED (creeper.rs:92-93, RevengeGoal at 1, CREEPER_HUNTS at 2)
- 2 charged victim heads: FIXED (creeper.rs:328; dropped_skulls flag removed)
- 3 lightning sets charged via set_charged: FIXED (creeper.rs:237)
- 4 spawnEffectsCloud parameters: FIXED (no change needed; see creeper-0de8b3 row 4a)
- 5 swell direction Bedrock metadata: FIXED (creeper.rs:99-106)
- 6 readAdditionalSaveData sync: FIXED (no gap per creeper-0de8b3 row 6)
- 7 charged_creeper/root loot table: BLOCKED-DATA (no loot_table JSON in assets/)

## Result

Re-review against the decompiled tree now present at `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Creeper.java` (Paper 1.21.x). No source edits were required to `creeper.rs` or `creeper_ignite.rs`.

- Row 4 (spawnLingeringCloud cloud params): NO GAP. Vanilla calls `setRadius(2.5F)`, `setRadiusOnUse(-0.5F)`, `setWaitTime(10)`, `setDuration(300)`, `setPotionDurationScale(0.25F)`, `setRadiusPerTick(-radius/duration)`. The tracker's `setDuration(600)` claim is incorrect. `creeper.rs:154-165` already passes duration 300, radius 2.5, reapplication delay 20 (vanilla default), wait 10, radius-on-use -0.5, duration-on-use 0 (vanilla default); `AreaEffectCloudEntity::create` computes radius-on-tick as -2.5/300 and hardcodes potion scale 0.25. Matches vanilla.
- Row 6 (readAdditionalSaveData sync): NO GAP. Vanilla writes `DATA_IS_POWERED`/`DATA_IS_IGNITED` straight into synced data on load. Pumpkin stores atomics in `mob_read_nbt`; the spawn path `java_spawn_metadata` -> `mob_java_spawn_metadata` -> `mob_metadata()` serialises charged/ignited/fuse_speed from those atomics (`MobMetadata::charged/ignited/fuse_speed` push Java entries CHARGED/IS_IGNITED/FUSE_ID plus Bedrock flags). Same observable behavior; the seam is already unit-tested in `entity/mob/metadata.rs`.
- Row 7 (charged_creeper/root loot table): RESOLVED BY DATA. `assets/datapack/data/minecraft/loot_table/charged_creeper/root.json` is now shipped and registered in generated `pumpkin-data/src/generated/loot_table.rs` as `charged_creeper/root`. `killed_entity` calls `world.get_loot_table("minecraft:charged_creeper/root")`, which resolves via the static table. Assets are outside this unit, so no edit was possible or needed.
- No new unit test: the inline cloud parameters have no pure seam, and the creeper metadata seam is already covered by an existing test. No `.rs` file was touched, so no rustfmt run.

Observation (out of scope, not changed): the same decompiled class gates charged-creeper heads behind `this.droppedSkulls` (one head per explosion), while the row 2 change on master removed that gate. Flagging only; row 2 is outside the STILL OPEN set for this unit.

## Result

Re-verified every STILL OPEN row against `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Creeper.java` (Paper 1.21.x) and the current Pumpkin sources. No `.rs` edit was needed, so no rustfmt was run.

- Row 4 (spawnLingeringCloud cloud params): NO GAP. Vanilla calls setRadius(2.5F), setRadiusOnUse(-0.5F), setWaitTime(10), setDuration(300), setPotionDurationScale(0.25F), setRadiusPerTick(-radius/duration). `creeper.rs:168-185` passes duration 300, radius 2.5, reapplication 20, wait 10, radius-on-use -0.5, duration-on-use 0; `AreaEffectCloudEntity::create` computes radius-on-tick as `-radius/max(duration,1)` and hardcodes potion scale 0.25. The tracker's `600` claim is incorrect.
- Row 6 (readAdditionalSaveData sync): NO GAP. Vanilla sets DATA_IS_POWERED and, when ignited, DATA_IS_IGNITED into synced data on load. Pumpkin's `mob_read_nbt` stores the atomics and sets IS_IGNITED synced data; at spawn `java_spawn_metadata` -> `mob_java_spawn_metadata` -> `mob_metadata()` serialises charged/ignited/fuse_speed from those atomics for Java clients, and `mob_bedrock_spawn_metadata` does the same for Bedrock. Same observable behaviour; this seam is already covered by the `creeper_metadata_generation` test in `metadata.rs`.
- Row 7 (charged_creeper/root loot table): RESOLVED BY DATA. `pumpkin_data::loot_table::get_loot_table("minecraft:charged_creeper/root")` resolves through the generated static `CHARGED_CREEPER_ROOT` table, and `datapack_manager.get_loot_table` falls back to that static table. `killed_entity` therefore drops charged-creeper heads without a datapack. Generated data is outside this unit; no edit possible or needed.
- No new unit test: the inline cloud parameters have no pure seam, and the creeper metadata seam already has a test. No `.rs` file was touched, so no rustfmt run.


## Result

Second pass (expanded file permissions). Re-verified each STILL OPEN row against `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Creeper.java` and `.../ai/goal/SwellGoal.java` (Paper 1.21.x) and the current Pumpkin sources. No `.rs` edit was required.

- Row 4 (spawnLingeringCloud cloud params): NO GAP. Vanilla sets radius 2.5, radiusOnUse -0.5, waitTime 10, duration 300, potionDurationScale 0.25, radiusPerTick -radius/duration (-2.5/300). `creeper.rs:174-185` passes duration 300, radius 2.5, reapplicationDelay 20 (vanilla default), waitTime 10, radiusOnUse -0.5, durationOnUse 0 (vanilla default); `area_effect_cloud.rs:116-120` computes radiusPerTick as `-radius/max(duration,1)` and hardcodes potion scale 0.25. The tracker's `600` claim is incorrect.
- Row 6 (readAdditionalSaveData sync): NO GAP. Vanilla always sets DATA_IS_POWERED and, when true, DATA_IS_IGNITED on load. `creeper.rs:227-253` stores the atomics and pushes CHARGED/IS_IGNITED synced data; `mob_metadata()` (line 263) feeds both Java and Bedrock spawn metadata from those same atomics. The seam is already covered by `creeper_metadata_generation` in `metadata.rs`.
- Row 7 (charged_creeper/root loot table): RESOLVED BY DATA. Assets exist at `assets/datapack/data/minecraft/loot_table/charged_creeper/*.json`; generated `pumpkin-data/src/generated/loot_table.rs` registers `CHARGED_CREEPER_ROOT` under `minecraft:charged_creeper/root`; `world.get_loot_table` falls back to the static table. No code change possible or needed.

No new pure unit test: the inline cloud parameters have no pure seam, and the creeper metadata seam already has a test. No source file was touched, so no rustfmt run was needed.
