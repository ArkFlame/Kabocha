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
| 2 | creeper.rs:328-352 | `Creeper.killedEntity` drops heads from loot table `minecraft:charged_creeper/root` | No such loot table ships in `assets/` (no `charged_creeper*` file exists), so `get_loot_table` returns `None` and charged creepers never drop heads without a datapack. Needs the vanilla data pack extracted (Extractor), not a code change | M |
| 3 | creeper.rs:107-168 | `Creeper.explodeCreeper` calls `spawnLingeringCloud()` then `triggerOnDeathMobEffects(RemovalReason.KILLED)` then `discard()` | Pumpkin has no on-death-effect hook (no `triggerOnDeath`/`on_death_effects` anywhere in `crates/pumpkin-core/src`); `explode()` spawns the cloud and removes the entity without firing per-effect removal callbacks. Implement the hook on the effect system first, then call it here | M |
| 4 | creeper_ignite.rs:39 | `CreeperSwellGoal.canUse` has no aliveness check on the target (`getSwellDir() > 0 \|\| target != null && distanceToSqr < 9`); the Rust version adds `target.get_entity().is_alive()` | *verify* against `Mob`/`LivingEntity` target clearing in decompiled vanilla before removing the guard; the guard only suppresses a swell toward an already dead target, so leave it unless vanilla proves the difference observable | S |

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
