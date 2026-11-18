# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-block-enti.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: living.rs mod.rs sculk_catalyst.rs

| 1 | `entities/sculk_catalyst.rs:125` | `ServerLevel.levelEvent(LevelEvent.SCULK_CATALYST_BLOOM)` also drives the client-side sculk-charge particle burst | Only the sound is sent; clients get no bloom particles. Client-only cosmetic, needs a level-event packet the server does not have. | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 2 | `entities/sculk_catalyst.rs:46,68-77` | `SculkCatalystBlockEntity` overrides no save/load, so `decay_delay` is not an NBT key | Pumpkin writes/reads a `decay_delay` int that is always 0 (dead field: no cursor uses it) and `chunk_data_nbt` can drop it under lock contention. Harmless, but NBT differs from vanilla. | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 3 | `entities/sculk_catalyst.rs:158-160`, caller `entity/living.rs:2983` | Game-event dispatch tests `sourcePosition.closerThan(eventPos, 8)` with the entity's real `Vec3` and a strict `<` | Uses the dying entity's block pos and `<=`, so a death exactly 8 blocks away on an axis blooms when vanilla would not. Boundary-only. | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 4 | `entity/living.rs:2981` (feeds `on_entity_death`) | `getExperienceReward(level, null)` for the catalyst; the killer is passed only for XP orbs | Uses `get_experience_reward(killer)` for both. No-op today because every implementation ignores the killer (`entity/mob/mod.rs:3394`), diverges once killer/enchantment bonuses are implemented. | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
