# Final rows from docs/tracker/tentities-creeper.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

Observation (out of scope, not changed): the same decompiled class gates charged-creeper heads behind `this.droppedSkulls` (one head per explosion), while the row 2 change on master removed that gate. Flagging only; row 2 is outside the STILL OPEN set for this unit.
Re-verified every STILL OPEN row against `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Creeper.java` (Paper 1.21.x) and the current Pumpkin sources. No `.rs` edit was needed, so no rustfmt was run.
Second pass (expanded file permissions). Re-verified each STILL OPEN row against `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Creeper.java` and `.../ai/goal/SwellGoal.java` (Paper 1.21.x) and the current Pumpkin sources. No `.rs` edit was required.
Third pass (expanded permissions). Re-verified each formerly STILL OPEN row against decompiled vanilla at `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Creeper.java` and `.vendor/vanilla-decompiled/net/minecraft/world/entity/AreaEffectCloud.java` (Paper 1.21.x), plus the current Pumpkin sources. No source edit was required for any row, so no rustfmt was run. No row remains STILL OPEN.

## Result
FINAL PASS. Re-verified every row against decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Creeper.java`
and `.vendor/vanilla-decompiled/net/minecraft/world/entity/AreaEffectCloud.java`
(Paper 1.21.x) plus the current `creeper.rs` and `area_effect_cloud.rs`. No `.rs`
edit was required, so no rustfmt was run. Every row ends FIXED or VERIFIED;
BLOCKED-DATA does not apply.

- Row 1 (target-selector order): VERIFIED. Vanilla Creeper.java:74-75 registers
  `NearestAttackableTargetGoal<Player>` at 1 and `HurtByTargetGoal` at 2. Pumpkin
  `creeper.rs:42` sets `Hunt::new(1, ...)` applied at line 106 (priority 1) and
  `RevengeGoal` at line 105 (priority 2). Matches vanilla.
- Row 2 (charged victim heads): VERIFIED. Vanilla Creeper.java:173-182 guards with
  `this.shouldDropLoot(level) && this.isPowered() && !this.droppedSkulls` and sets
  the flag in the loot callback. Pumpkin `creeper.rs:394-427` guards with
  `charged && !dropped_skulls && Mob::should_drop_loot(self)` (self is the killer,
  matching vanilla `this`) and sets `dropped_skulls` only when a stack drops.
  The stale observation above claiming the gate was removed is no longer true;
  the gate is present in current code. Matches vanilla.
- Row 3 (lightning sets charged): FIXED (already on master). `creeper.rs:307`
  calls `set_charged(true)`, pushing Java synced data and Bedrock actor data;
  matches vanilla Creeper.java:213 `entityData.set(DATA_IS_POWERED, true)`.
- Row 4 (spawnLingeringCloud cloud params): VERIFIED. Vanilla Creeper.java:266-284
  sets radius 2.5, radiusOnUse -0.5, waitTime 10, duration 300,
  potionDurationScale 0.25, radiusPerTick -radius/duration (-2.5/300). Pumpkin
  `creeper.rs:174-185` calls `AreaEffectCloudEntity::create(..., 300, 2.5, 20, 10,
  -0.5, 0)`; `area_effect_cloud.rs:107-122` maps duration 300, radius 2.5,
  reapplicationDelay 20 (vanilla default), waitTime 10, radiusOnUse -0.5,
  durationOnUse 0, computes radiusOnTick `-radius/max(duration,1)` = -2.5/300,
  and hardcodes scale 0.25. The tracker's original `setDuration(600)` claim is
  wrong; vanilla uses 300, so no code change.
- Row 5 (swell direction Bedrock metadata): FIXED (already on master).
  `creeper.rs:112-119` `set_fuse_speed` pushes Java FUSE_ID synced data and
  `send_bedrock_actor_data`, matching the set_charged/set_ignited pattern.
- Row 6 (readAdditionalSaveData sync): VERIFIED. Vanilla Creeper.java:116-124 sets
  DATA_IS_POWERED from "powered" (default false) and, when "ignited" is true,
  DATA_IS_IGNITED. Pumpkin `creeper.rs:227-253` stores the atomics and writes
  CHARGED/IS_IGNITED synced data; `mob_metadata()` feeds both Java and Bedrock
  spawn metadata from those atomics. Same observable behavior; seam covered by
  `creeper_metadata_generation` in `metadata.rs`.
- Row 7 (charged_creeper/root loot table): VERIFIED, resolved by generated data,
  not BLOCKED-DATA. Vanilla Creeper.java:175 drops from
  `BuiltInLootTables.CHARGED_CREEPER`. The required generated field exists:
  `pumpkin_data::loot_table::CHARGED_CREEPER_ROOT`
  (`crates/pumpkin-data/src/generated/loot_table.rs:43054`) is registered under
  both `charged_creeper/root` (line 64739) and `minecraft:charged_creeper/root`
  (line 68074). `creeper.rs:402` calls
  `world.get_loot_table("minecraft:charged_creeper/root")`, which falls back to
  that static table. No missing field or subsystem.

No new unit test: the inline cloud parameters have no pure seam, the metadata
seam already has a test, and the loot table is generated data. No source file
was touched.
