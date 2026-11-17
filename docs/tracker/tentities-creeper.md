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
| 4 | creeper.rs:153-167 | `Creeper.spawnEffectsCloud`: `setRadius(2.5F)`, `setRadiusOnUse(-0.5F)`, `setWaitTime(10)`, `setDuration(600)`, `setRadiusPerTick(-radius/duration)` | Cloud is created with duration `300`; pass `600` and add the `radius_per_tick = -2.5/600` argument (needs an `AreaEffectCloudEntity::create` parameter; current default is `-3.0/600`). Verify constants against decompiled Creeper first | S |
| 5 | creeper.rs:99-105 | `Creeper` swell direction is also part of the Bedrock actor metadata | `set_fuse_speed` pushes only Java `DATA_SWELL_DIR`; also `send_bedrock_actor_data` like `set_charged`/`set_ignited` do | S |
| 6 | creeper.rs:199-213 | `Creeper.readAdditionalSaveData`; vanilla entity data defaults drive the client | `mob_read_nbt` updates the atomic fields but not synced data; confirm the spawn packet serialises `mob_metadata()` (it does for `CHARGED`/`IS_IGNITED`), otherwise push `set_charged`/`set_ignited` after load | S |
| 7 | creeper.rs:328-352 | `Creeper.killedEntity` uses loot table `minecraft:charged_creeper/root` | No `assets/**/loot_table` JSON is shipped, so `get_loot_table` returns `None` and charged creepers never drop heads without a datapack; needs the vanilla data pack extracted (Extractor), not a code change | M |

## Checked and already matching vanilla

- `Creeper.tick` swell/prime-sound/explode ordering, negative swell clamp, `maxSwell` clamp on fall damage (`cause_fall_damage`).
- Goal list: swim 1, `CreeperSwellGoal` 2, ocelot/cat `AvoidEntityGoal` 3 (6 blocks, 1.0/1.2), `MeleeAttackGoal` 4 with no-op `doHurtTarget`, stroll 5, look 6 / random look 6; `CreeperSwellGoal` 3-block start, 7-block stop, line-of-sight, `requiresUpdateEveryTick`.
- Igniter interaction (tag `CREEPER_IGNITERS`, fire charge vs flint and steel sound, durability/stack decrement), explosion radius ×2 when charged, NBT keys (`powered`, `Fuse`, `ExplosionRadius`, `ignited`).
