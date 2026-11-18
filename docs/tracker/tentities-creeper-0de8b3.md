# Tracker: entities / Creeper (RE-REVIEW)

Source: `crates/pumpkin-core/src/entity/mob/creeper.rs`, `crates/pumpkin-core/src/entity/ai/goal/creeper_ignite.rs`.
Vanilla reference class: `net.minecraft.world.entity.monster.Creeper` / `CreeperSwellGoal` (1.21.x), cross-checked against `docs/VANILLA_REF.md` (fuse 30, radius 3, power 3/6). No `.vendor/` decompiled tree in this worktree; items marked *verify* need the decompiled class.

## Fixed in this pass

| # | file:line (before) | Gap vs vanilla | Fix |
|---|---|---|---|
| 1 | creeper.rs:99-105, 227-230 | `set_fuse_speed` never pushed Bedrock actor data, and `mob_metadata` omitted the swell direction while defusing, so Bedrock clients kept swelling after the target was lost | Push `mob_metadata().to_bedrock_actor_data()` from `set_fuse_speed`; always include the fuse field (vanilla idle value is -1) |

## Re-review of previously listed gaps

| # | previous claim | Verdict |
|---|---|---|
| 4 | lingering cloud should use duration 600 and `radiusPerTick = -2.5/600` | WRONG. Vanilla `spawnLingeringCloud` halves the cloud duration (`setDuration(getDuration() / 2)`), so 300 is correct; `AreaEffectCloudEntity::create` (area_effect_cloud.rs:113) computes `-radius / duration` = -2.5/300. The call at creeper.rs:153-163 (duration 300, radius 2.5, reapplication 20, wait 10, radiusOnUse -0.5, durationOnUse 0) matches vanilla field for field. No change. |
| 5 | swell direction missing from Bedrock actor metadata | REAL, fixed above. |
| 6 | `mob_read_nbt` updates atomics but not synced data | NOT A GAP. Spawn serialisation uses `mob_metadata()` for both Java and Bedrock (mob/mod.rs:1890-1901), and `mob_metadata` reads the atomics, so a stored `powered`/`ignited`/`Fuse` reaches clients at spawn. No change. |

## Remaining gaps

| # | file:line | Vanilla class / behaviour | Suggested fix | Size |
|---|---|---|---|---|
| 2 | creeper.rs:328-352 | `Creeper.killedEntity` drops heads from loot table `minecraft:charged_creeper/root` | No such loot table ships in `assets/` (no `charged_creeper*` file exists), so `get_loot_table` returns `None` and charged creepers never drop heads without a datapack. Needs the vanilla data pack extracted (Extractor), not a code change | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 3 | creeper.rs:107-168 | `Creeper.explodeCreeper` calls `spawnLingeringCloud()` then `triggerOnDeathMobEffects(RemovalReason.KILLED)` then `discard()` | Pumpkin has no on-death-effect hook (no `triggerOnDeath`/`on_death_effects` anywhere in `crates/pumpkin-core/src`); `explode()` spawns the cloud and removes the entity without firing per-effect removal callbacks. Implement the hook on the effect system first, then call it here | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 4 | creeper_ignite.rs:39 | `CreeperSwellGoal.canUse` has no aliveness check on the target (`getSwellDir() > 0 \|\| target != null && distanceToSqr < 9`); the Rust version adds `target.get_entity().is_alive()` | *verify* against `Mob`/`LivingEntity` target clearing in decompiled vanilla before removing the guard; the guard only suppresses a swell toward an already dead target, so leave it unless vanilla proves the difference observable | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |

## Checked and already matching vanilla

- `Creeper.tick` ordering: `oldSwell = swell`, ignite forces swell dir 1, prime sound + `GameEvent.PRIME_FUSE` only when `dir > 0 && swell == 0`, negative swell clamp, explode at `maxSwell` (creeper.rs:245-289).
- Explosion: `dead = true` before the explosion, radius 3 x2 when charged, `ExplosionInteraction::Mob`, lingering cloud geometry, entity removal (creeper.rs:107-168).
- Fall damage swell `current + 1.5*fall` clamped to `maxSwell - 5` (creeper.rs:172-187).
- Goal list/priorities: swim 1, `CreeperSwellGoal` 2, ocelot/cat avoid 3 (6 blocks, 1.0/1.2), `MeleeAttackGoal(1.0, false)` 4 with no-op `doHurtTarget`, stroll 5 (0.8), look-at 8 blocks 6, random look 6; targets `HurtByTargetGoal` 1 then nearest player 2. `CreeperSwellGoal` 3-block start / 7-block stop / line-of-sight, `should_run_every_tick`.
- Igniter interaction: `CREEPER_IGNITERS` tag, fire-charge vs flint-and-steel sound, random pitch 0.8-1.2, damageable vs stack decrement, leash fallback (creeper.rs:291-325).
- Lightning strike charges the creeper via `set_charged` (creeper.rs:234-243); NBT keys `powered`, `Fuse`, `ExplosionRadius`, `ignited` (creeper.rs:189-213).

## Gap status
- 1 Bedrock actor data on fuse speed: FIXED (creeper.rs:99-106 calls send_bedrock_actor_data)
- 4a lingering cloud duration 600: FIXED (no change needed; creeper.rs:153-163 uses 300, matches the note's correction)
- 5 swell direction Bedrock metadata: FIXED (creeper.rs:99-106)
- 6 mob_read_nbt synced data: FIXED (no gap per note; mob_metadata at mob/mod.rs:1873)
- 2 charged_creeper/root loot table: BLOCKED-DATA (no loot_table JSON in assets/; count 0)
- 3 explodeCreeper ordering (creeper.rs:107-168): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 4b CreeperSwellGoal aliveness guard (creeper_ignite.rs:39): STILL OPEN (unverified: no .vendor decompile in h6; not changed) (note says leave unless vanilla proves it observable)

## Result

Re-review against `.vendor/vanilla-decompiled/net/minecraft/` (Creeper.java, SwellGoal.java, LivingEntity.java, OozingMobEffect/WeavingMobEffect/WindChargedMobEffect).

- Row 2 (charged-creeper heads): FIXED. The tracker's data blocker was wrong. `minecraft:charged_creeper/root` is generated in `crates/pumpkin-data/src/generated/loot_table.rs` (`CHARGED_CREEPER_ROOT` + creeper/piglin/skeleton/wither_skeleton/zombie heads) and `World::get_loot_table` resolves it via the generated-data fallback (`world/mod.rs:9158`, `datapack/mod.rs:636`). The real gap was vanilla's one-shot `!this.droppedSkulls` guard (Creeper.java:174-179): Pumpkin dropped a head for every victim of one explosion. Added `CreeperEntity::dropped_skulls: AtomicBool`, guarded `killed_entity` with it, and set it only when a stack actually drops (vanilla sets the flag in the loot callback, so a non-head victim does not consume the drop). `creeper.rs` formatted with `rustfmt --edition 2024`.
- Row 3 (explodeCreeper on-death effects): STILL OPEN. Needs a cross-file effect refactor outside this unit. Vanilla `Creeper.explodeCreeper` (Creeper.java:245-256) calls `triggerOnDeathMobEffects(KILLED)` -> `MobEffectInstance.onMobRemoved` (LivingEntity.java:826-833); `OozingMobEffect`, `WeavingMobEffect`, `WindChargedMobEffect` override it to spawn slimes/a cobweb/a wind burst. Pumpkin's equivalent is `MobEffect::on_mob_death(&LivingEntity, u8, &DamageType)` in `entity/effect/mod.rs` (outside unit files), and it is only called from the normal damage/death path (`entity/living.rs:3050-3065`), which `explode()` bypasses via `dead=true` + `entity.remove()`. Fix requires adding a damage-type-free removal hook to the trait + impls and calling it from `explode()`; not possible inside creeper.rs/creeper_ignite.rs/area_effect_cloud.rs alone.
- Row 4 (SwellGoal aliveness guard): VERIFIED, NO CHANGE. The tracker premise was wrong. Vanilla `SwellGoal.canUse` (SwellGoal.java:18-21) is `getSwellDir() > 0 || (target != null && !target.isDeadOrDying() && distanceToSqr < 9.0)`, i.e. it DOES carry the aliveness guard, and `tick` (line 42) checks `!target.isDeadOrDying()` too. The Rust `creeper_ignite.rs:39` `is_alive()` guard is therefore correct and must stay. No observable difference to fix.

No unit test added: the changed behaviour (loot drop through `World`/`LootContextParameters`) has no pure seam; the row-4 resolution required no code change.


## Result

Re-verify of the two STILL OPEN rows against decompiled vanilla, now reachable through the `.vendor/vanilla-decompiled` symlink (`rel-mc-backup/net/minecraft/`). No source file in this unit required a change: both rows were already resolved in the checkout.

- Row 3 (explodeCreeper on-death effects): FIXED in checkout, verified. The cross-file hook the previous pass called a blocker is now present: `MobEffect::on_mob_removed`, a damage-type-free removal hook, exists in `entity/effect/mod.rs:45-47` with concrete impls in `oozing.rs`, `weaving.rs`, `wind_charged.rs`; `get_mob_effect` resolves all three. `CreeperEntity::explode` (`creeper.rs:121-196`) now calls them in the vanilla order: `dead=true` -> `explode_from` -> lingering cloud (duration 300, radius 2.5) -> `on_mob_removed` for each active effect -> `entity.remove()`. This matches vanilla `Creeper.explodeCreeper` (`Creeper.java:245-264`) and `LivingEntity.triggerOnDeathMobEffects` (`LivingEntity.java:826-833`). `entity.remove()` only calls `world.remove_entity`, so the normal damage/death path in `living.rs` cannot double-fire the hooks. Code change needed: none.
- Row 4 (SwellGoal aliveness guard): VERIFIED, NO CHANGE. Decompiled vanilla `SwellGoal.canUse` (`SwellGoal.java:18-21`) is `getSwellDir() > 0 || (target != null && !target.isDeadOrDying() && distanceToSqr < 9.0)` and `tick` (line 42) checks `!target.isDeadOrDying()`; `LivingEntity.isDeadOrDying` is `health <= 0 || dead`. The Rust `creeper_ignite.rs` `is_dead_or_dying` guard plus `is_alive()` correctly suppresses a swell toward an already dead/dying target, so the tracker premise (vanilla has no guard) is false. The extra `is_alive()` (`entity.is_alive() && health > 0`) is redundant but not observable, so left to avoid speculative churn.
- Row 2 (charged-creeper heads): re-checked, still FIXED (`dropped_skulls` one-shot guard at `creeper.rs:374-407`, matching `Creeper.java:174-179`).

No pure unit test seam: row 3 fires through `World`/effect impls (entities, blocks, explosions), row 2 through `World::get_loot_table`/`LootContextParameters`, row 4 through live entity targets. No `.rs` file touched this pass, so no `rustfmt` run.

## Result

Re-verified all three STILL OPEN rows against the decompiled vanilla tree; no source file needed a change this pass.

- Row 2 (charged-creeper heads): VERIFIED FIXED. ``Creeper.killedEntityPreEvent`` (Creeper.java:172-182) uses ``BuiltInLootTables.CHARGED_CREEPER`` = ``charged_creeper/root`` (BuiltInLootTables.java:122) with a one-shot ``!droppedSkulls`` guard. ``CreeperEntity::dropped_skulls`` plus the guard in ``killed_entity`` (creeper.rs:374-407) matches. ``World::get_loot_table`` (world/mod.rs:9403) and ``DatapackManager::get_loot_table`` (data/datapack/mod.rs:816-835) fall back to generated ``CHARGED_CREEPER_ROOT``, so the old data blocker is resolved. No change.
- Row 3 (explodeCreeper on-death effects): VERIFIED FIXED. ``Creeper.explodeCreeper`` (Creeper.java:245-264) order is ``dead = true`` -> explode -> ``spawnLingeringCloud`` -> ``triggerOnDeathMobEffects(KILLED)`` -> ``discard``; ``LivingEntity.triggerOnDeathMobEffects`` (LivingEntity.java:826-833) delegates to ``MobEffectInstance.onMobRemoved``. ``CreeperEntity::explode`` (creeper.rs:121-196) fires ``on_mob_removed`` for every active effect after spawning the cloud and before ``entity.remove()``; the trait hook is in entity/effect/mod.rs with impls in oozing.rs/weaving.rs/wind_charged.rs, and Pumpkin's ``entity.remove()`` does not re-fire effect hooks, so there is no double activation. Cloud geometry (300 ticks, radius 2.5, ``-radius/duration``, 0.25 potion scale, wait 10, reapplication 20) matches vanilla ``spawnLingeringCloud``. Effect counts match the vanilla registrations in MobEffects.java:120-123 (OOZING 2 slimes, WEAVING 2-3 cobwebs, WIND_CHARGED 3-5 gust). No change.
- Row 4 (SwellGoal aliveness guard): VERIFIED, NO CHANGE. ``SwellGoal.canUse`` (SwellGoal.java:18-21) is ``getSwellDir() > 0 || (target != null && !target.isDeadOrDying() && distanceToSqr < 9.0)`` and ``tick`` (line 42) also checks ``!isDeadOrDying()``. The Rust ``is_alive() && !is_dead_or_dying()`` guard in creeper_ignite.rs:34-54 is therefore correct; the tracker premise is false. No change.

No pure unit-test seam exists for these rows (all fire through World/entity/loot state) and no ``.rs`` file was touched, so no rustfmt was needed. Out-of-unit parity gaps observed but not changed: oozing ignores MAX_ENTITY_CRAMMING (OozingMobEffect.java:38-39) and weaving uses ``is_air`` instead of ``canBeReplaced``/``isFaceSturdy`` (WeavingMobEffect.java:40-41).


## Result

Resume check after server restart: all three STILL OPEN rows are already resolved in the checkout; no source file needed a change this pass, so no rustfmt run.

- Row 2 (charged-creeper heads): VERIFIED FIXED. ``CreeperEntity::dropped_skulls: AtomicBool`` (creeper.rs:63), the ``!dropped_skulls`` guard plus ``minecraft:charged_creeper/root`` lookup (creeper.rs:383-387), and the one-shot store on an actual drop (creeper.rs:407) are present, matching the vanilla ``!droppedSkulls`` guard.
- Row 3 (explodeCreeper on-death effects): VERIFIED FIXED. ``CreeperEntity::explode`` calls ``mob_effect.on_mob_removed(...)`` (creeper.rs:191); the trait hook is in ``entity/effect/mod.rs:47`` with impls in ``oozing.rs``, ``weaving.rs`` and ``wind_charged.rs``.
- Row 4 (SwellGoal aliveness guard): VERIFIED FIXED. ``creeper_ignite.rs:46-47`` uses ``is_alive() && !is_dead_or_dying()``; the tracker premise is false, no change.

No pure unit-test seam (rows fire through World/entity/loot/effect state). Out-of-unit parity gaps remain observed but unchanged: oozing MAX_ENTITY_CRAMMING and weaving ``canBeReplaced``/``isFaceSturdy``.

## Result

Second-pass re-review against `.vendor/vanilla-decompiled/net/minecraft/`. The three STILL OPEN tracker rows remain verified fixed in the checkout, so no change was needed for them: row 2 (`dropped_skulls` one-shot guard), row 3 (`on_mob_removed` fired in `CreeperEntity::explode`), row 4 (`SwellGoal.canUse` aliveness guard matches vanilla).

Fixed the two parity gaps the previous pass flagged as out-of-unit, now that `oozing.rs`/`weaving.rs` are in this unit's file list:

- `weaving.rs`: vanilla `WeavingMobEffect.spawnCobwebsRandomlyAround` (WeavingMobEffect.java:40-41) needs `canBeReplaced()` on the target and `isFaceSturdy(..., Direction.UP)` on the block below. Replaced `target_state.is_air()` with `target_state.replaceable()` and `!below_state.is_air()` with `below_state.is_side_solid(BlockDirection::Up)`; added the `BlockDirection` import.
- `oozing.rs`: vanilla `OozingMobEffect.onMobRemoved` (OozingMobEffect.java:35-45, clamp formula at line 31) caps spawned slimes at `max_entity_cramming - nearby_slimes`, where nearby counts SLIME entities in the mob bounding box inflated by 2.0 (excluding self, capped at the cramming limit); a rule value < 1 spawns the requested 2 uncapped. Replaced the hardcoded `for _ in 0..2` with that clamp.

Not fixable in this pass: vanilla's `reason == KILLED` guard on both effects would require adding a removal-reason parameter to `MobEffect::on_mob_removed` in `entity/effect/mod.rs`, a cross-crate trait signature change outside this unit. In Pumpkin `on_mob_removed` currently fires only on killed removals, so the guard is not presently observable.

No pure unit-test seam: both effects mutate World/entity state and a clamp test would only mirror the formula. Only `weaving.rs` and `oozing.rs` were touched; rustfmt is left to the orchestrator.
