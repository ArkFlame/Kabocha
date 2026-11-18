# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-world-expl.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

Allowed to touch any cross-file row this pass; re-verified every PRIOR-OPEN row against

## Result
FINAL PASS (2026-11-18). Re-verified all 8 rows against
`.vendor/vanilla-decompiled/net/minecraft/world/level/ServerExplosion.java`
(CraftBukkit/Paper-instrumented target version), `.../damagesource/DamageSources.java` and the
current Pumpkin source. No `.rs` change was required: every fixable row landed in earlier passes;
the two terminal gaps need a subsystem/data set not present in this repo.

| # | Status | Evidence |
|---|--------|----------|
| 1 | VERIFIED | Block list + yield, event filter and yield-to-loot applied. Vanilla `interactWithBlocks` fires the event with the non-air list, rebuilds it from `event.blockList()` and reads `getYield()` (`ServerExplosion.java:610-643`), default yield `1/radius` only for DESTROY_WITH_DECAY (`:317-319`). Pumpkin: `explosion.rs:915-933`, `fire_explode_event` at :848, `BlockExplodeEvent` mutable `blocks`/`yield_rate` and `BlockEvent` impl in `block_explode.rs:41-45`. |
| 1a | BLOCKED-DATA | Paper's exploded-`BlockState` snapshot: vanilla sources it from `damageSource.causingBlockSnapshot()` (`ServerExplosion.java:633`). Missing subsystem: Pumpkin's `Explosion` struct (`explosion.rs:227-243`) has no causing-block field, and no Paper `BlockExplodeEvent`/`CraftEventFactory` source exists under `.vendor/` to pin the API type; adding a field changes `BlockExplodeEvent::new` arity at the `explosion.rs:870` call site. |
| 2 | BLOCKED-DATA | Unloaded-chunk force-load: vanilla `level.getChunk` force-loads the sampled chunk (`ServerExplosion.java:119`). Missing subsystem: async chunk prefetch before the sync tick. `load_chunk_for_explosion` (`explosion.rs:127-151`) already force-loads via `get_or_fetch_chunk`; only the current-thread-runtime fallback (`:146-149`) keeps the air fallback, because `Handle::block_on`/`block_in_place` would deadlock the chunk IO tasks. |
| 3 | VERIFIED MATCHING | `ServerExplosion` wraps the source in `EntityBasedExplosionDamageCalculator` only when the calculator is null (`:316,323-325`); Pumpkin applies `source.get_block_explosion_resistance` only when `damage_calculator.is_none()` (`explosion.rs:578-582`). Doc premise was inverted; no change. |
| 4 | VERIFIED FIXED | Vanilla draws from `level.getRandom()` (`ServerExplosion.java:410,612,674`); `World::explosion_random` seeded from `level.seed.0` (`world/mod.rs:700`) drives ray power (`explosion.rs:508`) and fire rolls (`:835`). |
| 5 | VERIFIED MATCHING | `isSmall() = radius < 2.0 \|\| !interactsWithBlocks()` (`ServerExplosion.java:770-772`) == Pumpkin `power < 2.0 \|\| Keep` (`explosion.rs:403-405`). |
| 6 | VERIFIED MATCHING | Mojang `explode()` returns `toBlow.size()` computed before the CraftBukkit-only event filter (`ServerExplosion.java:694,697,711`); Pumpkin returns the pre-filter `to_blow_count` (`explosion.rs:911,985`), sent at `world/mod.rs:5156`, locked by `incendiary_block_count_includes_air_positions` (`explosion.rs:1124`). Paper's post-filter count is a client-visual divergence that would regress that test. |
| 7 | VERIFIED FIXED | `DamageSources.explosion` selects `PLAYER_EXPLOSION` when both direct and indirect sources exist (`DamageSources.java:251-256`); `damage_type()` gates on exactly that (`explosion.rs:355-362`), test at :1229. |
| 8 | VERIFIED FIXED | Vanilla emits `gameEvent(EXPLODE)` (`ServerExplosion.java:693`); Pumpkin sends particle/sound in the packet and honours `!source.is_silent()` for `play_sound` (`world/mod.rs:5142-5161`). |

No pure unit test added this pass: the only logic in `block_explode.rs` is a world lookup inside
`get_block`, with no seam that is not a World-backed integration test (getter-style tests are
excluded by the rules). No rustfmt/cargo/git run (per task brief).
