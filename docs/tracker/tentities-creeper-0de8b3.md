# Tracker: entities / Creeper (RE-REVIEW)

Reviewed: `crates/pumpkin-core/src/entity/mob/creeper.rs`, `crates/pumpkin-core/src/entity/ai/goal/creeper_ignite.rs`
after commit `cd3fe8272` (targeting order, per-victim charged heads, lightning charge).
Reference class: `net.minecraft.world.entity.monster.Creeper` / `CreeperSwellGoal` (current protocol target).
No `.vendor/` decompiled tree in this worktree, so vanilla behaviour below is from the class as known
for the target version plus in-repo vanilla ports (lingering potion) as cross-check.

## Fixed in this pass

- **Lingering effect cloud duration** (`creeper.rs:157`, was `300`). Vanilla `Creeper#spawnEffectsCloud`
  builds a plain `new AreaEffectCloud(...)` and never calls `setDuration`, so the entity default of
  `600` ticks applies; `setRadiusPerTick(-radius / duration)` then shrinks 2.5 -> 0 over 30 s.
  Pumpkin passed `300` (15 s). The in-repo lingering potion port passes `600` for the same
  default-duration cloud, confirming the constant. Now `600`; the radius per tick is derived inside
  `AreaEffectCloudEntity::create`, so it follows automatically. Commit `d6fe59a3a`.

## Remaining gaps

1. **Bedrock actor data not refreshed on swell change** — `creeper.rs:99-105` (`set_fuse_speed`).
   `set_charged` / `set_ignited` both push `send_bedrock_actor_data` after the Java tracked-data write;
   `set_fuse_speed` writes only `DATA_SWELL_DIR`, so Bedrock clients never see the fuse animation.
   Vanilla class: `Creeper#setFuseSpeed` (+ Bedrock metadata mapping). Suggested fix: mirror the
   `set_charged` pattern (call `mob_metadata()` and `send_bedrock_actor_data` after the sync). Size **S**;
   client-only effect, no Java parity impact.

2. **NBT load does not re-push synced data** — `creeper.rs:199-213` (`mob_read_nbt`).
   Vanilla `readAdditionalSaveData` writes the field and the entity-data defaults are serialised in the
   spawn packet via `mob_metadata()`. Pumpkin updates only the atomics; confirm the spawn packet is
   built after `mob_read_nbt` (it is for `CHARGED`/`IS_IGNITED`), otherwise call `set_charged`/`set_ignited`
   after load. Size **S**, verify-only unless the packet order changed.

3. **Charged-creeper head loot table is not shipped with the server data** — `creeper.rs:326-350`
   (`killed_entity`). Vanilla resolves `minecraft:charged_creeper/root`; `assets/` in this tree contains no
   `loot_table` JSON, so `world.get_loot_table(...)` returns `None` and charged creepers drop no heads
   without a datapack. Needs the vanilla data pack extracted (Extractor), not a code change. Size **M**.

4. **No potion-duration scale on the spawned cloud** — `creeper.rs:153-163`. Newer vanilla sets
   `setPotionDurationScale(0.25F)` on `spawnEffectsCloud`; `AreaEffectCloudEntity` has no such field.
   Additive feature in the block-entity/entity state, affects effect durations only. Size **L**, verify
   against the target version first.

## Checked and matching vanilla

- `Creeper#tick` ordering: old-swell snapshot, ignited -> fuse speed 1, primed sound/game event only at
  swell 0, negative swell clamp, `swell >= maxSwell` -> `explodeCreeper` (dead before explode, cloud, remove).
- `explodeCreeper` radius `3 * (powered ? 2 : 1)`, `ExplosionInteraction::Mob`.
- `handleFallDamage` swell bonus `fall distance * 1.5`, clamped to `maxSwell - 5`.
- `CreeperSwellGoal`: 3-block start, 7-block stop, line of sight, every-tick update, MOVE control with
  navigator stop on start.
- Interaction: `CREEPER_IGNITERS` tag, fire-charge vs flint-and-steel sound played to the interacting
  player only, durability/stack decrement, `do_hurt_target` no-op.
- `thunderHit` -> powered, NBT keys `powered`/`Fuse`/`ExplosionRadius`/`ignited`, natural spawner entry,
  goal order (swim 1, swell 2, ocelot/cat avoid 3, melee 4, stroll 5, look 6) and target order
  (`HurtByTargetGoal` 1, nearest player 2) as fixed in `cd3fe8272`.
