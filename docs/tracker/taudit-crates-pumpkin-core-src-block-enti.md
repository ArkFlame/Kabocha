# block entities / SculkCatalyst — audit
Sources: `crates/pumpkin-core/src/block/entities/sculk_catalyst.rs`, `block/blocks/sculk/sculk_catalyst.rs`, `entity/living.rs:2978-2989`, `world/mod.rs:2099-2129,2497-2510`.
Compared against vanilla `SculkCatalystBlockEntity` (+ `CatalystListener`), `SculkCatalystBlock`, `SculkSpreader` (no decompiled reference in `.vendor/`).
The tracker note's "5 TODO/FIXME/vanilla markers" are five prose comments mentioning "vanilla"; the file contains no `TODO`/`FIXME`/`todo!`.
## Verified matching (no change made)
- Hearing radius 8, soul only for `charge > 0`, gate `should_drop_experience()` == vanilla `shouldDropExperience()` (`!is_baby`, `entity/living.rs:422`) and non-player deaths.
- Bloom: `BLOOM` blockstate set, `SCULK_CATALYST_BLOOM` sound, scheduled block tick 20 ticks later resets the state and the `bloomed` latch (`blocks/sculk/sculk_catalyst.rs:21-37`).
- The catalyst is ticked once per game tick by the generic block-entity loop; both that loop and the scheduled-tick dispatcher skip an entity/block that no longer owns the position (`world/mod.rs:2118-2126`, `:2501`), so a replaced block cannot be corrupted.
- `get_block_state_id` never loads a chunk (falls back to air, `world/mod.rs:7278-7281`), so the `SculkLevel` reads cannot block or generate from the tick path.
- Spreader state is intentionally not persisted (as vanilla): `write_nbt` stores only `decay_delay`.
## Remaining gaps
| # | Location | Vanilla | Gap | Size |
|---|---|---|---|---|
| 1 | `entities/sculk_catalyst.rs:125` | `ServerLevel.levelEvent(LevelEvent.SCULK_CATALYST_BLOOM)` also drives the client-side sculk-charge particle burst | Only the sound is sent; clients get no bloom particles. Client-only cosmetic, needs a level-event packet the server does not have. | S |
| 2 | `entities/sculk_catalyst.rs:46,68-77` | `SculkCatalystBlockEntity` overrides no save/load, so `decay_delay` is not an NBT key | Pumpkin writes/reads a `decay_delay` int that is always 0 (dead field: no cursor uses it) and `chunk_data_nbt` can drop it under lock contention. Harmless, but NBT differs from vanilla. | S |
| 3 | `entities/sculk_catalyst.rs:158-160`, caller `entity/living.rs:2983` | Game-event dispatch tests `sourcePosition.closerThan(eventPos, 8)` with the entity's real `Vec3` and a strict `<` | Uses the dying entity's block pos and `<=`, so a death exactly 8 blocks away on an axis blooms when vanilla would not. Boundary-only. | S |
| 4 | `entity/living.rs:2981` (feeds `on_entity_death`) | `getExperienceReward(level, null)` for the catalyst; the killer is passed only for XP orbs | Uses `get_experience_reward(killer)` for both. No-op today because every implementation ignores the killer (`entity/mob/mod.rs:3394`), diverges once killer/enchantment bonuses are implemented. | S |
| 5 | `entities/sculk_catalyst.rs:62` | `level.getRandom()` | Reseeds a Xoroshiro from OS entropy every tick, so spreading is not reproducible from the world seed. Same pattern in `fire`, `stem`, `jigsaw_block`, `grass_block`; not catalyst-specific, fix house-wide. | S |
| 6 | `entities/sculk_catalyst.rs:132-172` | Souls arrive through the `GameEvent.ENTITY_DIE` listener (BY_DISTANCE, radius 8) | Manual dispatch from `LivingEntity::on_death` only, so any entity removed without that path (despawn/`discard`, direct `remove`) never feeds a catalyst. Equivalent for normal deaths; revisit when a game-event system lands. | M |
| 7 | `pumpkin-world/.../sculk/spreader.rs:81-83` | `SculkSpreader.createLevelSpreader()` constants | Level spreader uses 10/4/10/5 (growth spawn cost, no-growth radius, charge decay, additional decay); not verifiable without a decompile, and shares the file's charge-merge logic. | M |
