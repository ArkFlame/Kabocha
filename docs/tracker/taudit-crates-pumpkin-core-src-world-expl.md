# world / Explosion

Audit, 2026-11-16, branch kab/taudit-crates-pumpkin-core-src-world-expl. File:
`crates/pumpkin-core/src/world/explosion.rs` (1023 lines, no TODO/FIXME markers; the three
"vanilla" hits are doc comments at :119, :631, :779). No `.vendor/` vanilla tree is present in
this worktree, so every claim below is against `ServerExplosion`/`Explosion` knowledge and the
call sites in this repo; items marked *verify* need a decompiled-source check before editing.

## Source map

| File | Class it ports |
|---|---|
| crates/pumpkin-core/src/world/explosion.rs | ServerExplosion, Explosion, ExplosionDamageCalculator, SimpleExplosionDamageCalculator |
| crates/pumpkin-core/src/world/mod.rs:4609-4726 | Level.explode / ServerLevel.explode + ClientboundExplodePacket fan-out |
| crates/pumpkin-core/src/plugin/api/events/block/block_explode.rs | Paper BlockExplodeEvent |

## Verified matching vanilla

- Ray tracing: 16^3 shell directions (skip interior), `dir/7.5-1` normalisation, 0.3 step,
  `h = power * (0.7 + rand*0.6)`, `h -= (resistance + 0.3) * 0.3` then `h -= 0.22500001`, break
  outside build limit (explosion.rs:421-529).
- `getBlockExplosionResistance`: air + empty fluid -> None, else `max(block, fluid)` (explosion.rs:57-70).
- Fire roll: one roll per affected position, `roll == 0 && state.isAir() && below.isSolidRender()`,
  `BaseFireBlock.getState`, `setBlockAndUpdate` flags (explosion.rs:376-397, 751-757).
- Entity phase: radius `power*2`, AABB floor(center +- radius +- 1), skip immune/spectator/direct
  source, `distance > 1.0` skip, exposure sampling grid + `ClipContext(Collider, Fluid.NONE)`,
  damage `((e*e + e)/2 * 7 * radius) + 1`, creative-flying knockback skip, TNT uses feet position
  for the knockback direction, `explosion_knockback_resistance` attribute, Java players receive the
  impulse in the packet instead of a server velocity (explosion.rs:536-658, 680-738).
- Block phase: `BlockInteraction` keep/trigger/destroy/decay mapping, `should_drop_items_on_explosion`,
  `explosion_radius` loot parameter, block `explode` hook, container-screen close, air replace with
  NOTIFY_ALL (explosion.rs:751-846).
- Paper yield argument `1.0 / power` on BlockExplodeEvent (explosion.rs:786-790).

## Fixed in this audit

- explosion.rs:742-762: explosions never announced `GameEvent::Explode`. Vanilla
  `ServerExplosion#explode` calls `level.gameEvent(source, GameEvent.EXPLODE, center)` first, which
  drives sculk sensors / vibration listeners. Now emitted through
  `World::emit_game_event_with_context` with the direct source, before any damage or block change.

## Remaining gaps

| # | Location | Vanilla / Paper behaviour | Suggested fix | Size |
|---|---|---|---|---|
| 1 | explosion.rs:781-799 (and world/mod.rs:4695-4708, block_explode.rs:10-14) | Paper builds `BlockExplodeEvent`/`EntityExplodeEvent` with the affected block list and replaces the to-blow list with `event.blockList()`; `yield` also scales drops | Add a block list (and mutable yield) to both events, fire after `get_blocks_to_destroy`, filter `blocks` by the returned list, apply yield to drops | M |
| 2 | explosion.rs:451-470 | `level.getBlockState` loads/forces the chunk, so explosions still destroy blocks in unloaded areas; here `read_chunk_sync` returning `None` is treated as air | Fall back to `world.get_block_state` (which loads) or force-load the chunk before sampling | M. STILL OPEN (design recorded, UNVERIFIED test): unloaded chunks count as air via the sync read, documented at the sampler (`explosion.rs`); vanilla force-load needs an async prefetch before the sync tick. Prior note: M. STILL OPEN: `explode()` is sync inside the tick and both the sampler and `World::get_block_state` use the non-loading `read_chunk_sync`; a force-load needs an async prefetch before the sync tick, a cross-cutting change (not attempted) |
| 3 | explosion.rs:506-515 | `Entity#getBlockExplosionResistance` is applied on top of the calculator value even when a custom calculator is set (wind charge, wither skull); here the hook is skipped whenever `damage_calculator.is_some()` | Apply `source.get_block_explosion_resistance` to any `Some(resistance)`; *verify* against the target version's `calculateExplodedPositions` first | S |
| 4 | explosion.rs:441, 744 | Ray power and fire rolls come from the level `RandomSource`, so a seed replays explosions deterministically; here `rand::random` / `rand::rng()` use thread RNG | Draw from a world-owned seeded RNG (like `loot_random`) | S-M |
| 5 | explosion.rs:339-343 | `ServerExplosion#isSmall` is `radius < 2.0` in common mappings; the extra `Keep` clause makes a mobGriefing-off creeper (power 3) use the small particle | Drop the `Keep` clause if the target version has no such rule; *verify*. Client-visual only | S |
| 6 | explosion.rs:843, world/mod.rs:4717 | `ClientboundExplodePacket.blockCount` in vanilla is the destroyed-block count fed to the client's smoke particles; confirm affected-vs-destroyed | Confirm and align; *verify*. Client-visual only | S |
| 7 | explosion.rs:308-318 | `player_explosion` damage type is used when the credited entity is a player; here any living indirect source (e.g. wither skull -> wither) selects it | Gate on the indirect source being a player; *verify* `DamageSources#explosion` | S |
| 8 | world/mod.rs:4711-4725 (not this file) | Vanilla `finalizeExplosion` emits its own particle/sound; here the packet is the only source, sent to players within 64 blocks | Confirm the radius and that every caller sets the right particle/sound; client-only | S |

## Status (code-verified sweep)
FIXED: getBlockExplosionResistance returns None for air with empty fluid (world/explosion.rs:58-70).


## Result

Verification against `.vendor/vanilla-decompiled/net/minecraft/` (target version), 2026-11-17:

- Row 1 (BlockExplodeEvent/EntityExplodeEvent block list + yield): LEFT OPEN. M-sized plugin event API change; `EntityExplodeEvent` and the packet fan-out are outside this unit's two files.
- Row 2 (unloaded-chunk force-load): LEFT OPEN. `explode()` is sync in the tick and uses the non-loading `read_chunk_sync`, so unloaded chunks count as air; vanilla `getChunk` force-loads and needs an async prefetch before the sync tick (cross-cutting, outside these files).
- Row 3 (entity blast resistance with custom calculator): VERIFIED MATCHING, no change. `ServerExplosion` only wraps the source in `EntityBasedExplosionDamageCalculator` when `damageCalculator == null` (ServerExplosion.java:316,323-325); a custom calculator skips the entity hook, which is what `explosion.rs:518-524` already does. The doc premise was inverted.
- Row 4 (seeded level RNG): LEFT OPEN. Vanilla draws from `level.getRandom()`; there is no dedicated level RNG here, only `world.loot_random` (world/mod.rs:341). Reusing it would perturb loot rolls and adding a field is outside this unit.
- Row 5 (is_small Keep clause): VERIFIED MATCHING, no change. Vanilla `isSmall() = radius < 2.0F || !interactsWithBlocks()` with `interactsWithBlocks() = blockInteraction != KEEP` (ServerExplosion.java:725-727,770-772), identical to `explosion.rs:346-350`.
- Row 6 (ClientboundExplodePacket blockCount): LEFT OPEN. Packet construction is in world/mod.rs (outside this unit); client-visual only. Note: `explode()` returns the destroyed-block count, whereas vanilla returns `toBlow.size()`, which also includes air positions when the explosion creates fire.
- Row 7 (damage type): FIXED. `DamageSources.explosion(explosion)` selects `PLAYER_EXPLOSION` whenever both direct and indirect sources exist (DamageSources.java:251-256), and `Explosion.getIndirectSourceEntity` returns the living source itself for creepers/withers or the living owner for TNT/projectiles (Explosion.java:17-24). `damage_type()` now gates on `source.is_some() && indirect_source().is_some()`. Added unit test `entity_explosion_uses_player_explosion_damage_type`.
- Row 8 (finalizeExplosion particles/sound): LEFT OPEN. world/mod.rs:4711-4725 is outside this unit; client-visual only.

Changed: crates/pumpkin-core/src/world/explosion.rs. Check: `rustfmt --edition 2024` on that file (exit 0). No cargo/git run (per task).

## Result

Re-verification 2026-11-18 (block_explode.rs slice). Checked STILL OPEN rows against
`.vendor/vanilla-decompiled/net/minecraft/world/level/ServerExplosion.java`
(CraftBukkit/Paper-instrumented target version).

- Row 1 (BlockExplodeEvent block list + yield): VERIFIED, already applied in this file.
  `interactWithBlocks` (ServerExplosion.java:610-643) fires the event with the non-air
  block list and a yield of `1.0/radius` for DESTROY_WITH_DECAY else `1.0`
  (finite-guarded at :318-319), then replaces the target list from `event.blockList()`
  and reads `event.getYield()`. `BlockExplodeEvent` already exposes a mutable
  `blocks: Vec<BlockPos>` and `yield_rate: f32`, and its `new()` signature matches
  the call site at `explosion.rs:859-865` (`event.cancelled/yield_rate/blocks` are all
  consumed at :869). The firing/filter/yield-to-loot side lives in `explosion.rs`,
  outside this file. No change needed here.
- Row 2 (unloaded-chunk force-load): LEFT OPEN, outside this file. Needs `World`/
  `explosion.rs` changes (sync tick currently reads unloaded chunks as air).
- Row 4 (seeded level RNG): LEFT OPEN, outside this file. Now partly addressed via
  `World::explosion_random` (world/mod.rs:373), which is in another unit's file.
- Row 6 (ClientboundExplodePacket blockCount): LEFT OPEN, outside this file
  (packet construction in world/mod.rs; client-visual only).
- Row 8 (finalizeExplosion particles/sound): LEFT OPEN, outside this file
  (world/mod.rs; client-visual only).
- Rows 3, 5, 7: previously VERIFIED MATCHING/FIXED; unchanged.

No `.rs` source edited (block_explode.rs already satisfied row 1), so no rustfmt
run was required. No cargo/git run per task.

## Result

Re-verification 2026-11-18 (block_explode.rs unit, only file permitted). Checked
STILL OPEN rows against `.vendor/vanilla-decompiled/net/minecraft/world/level/ServerExplosion.java`
(CraftBukkit/Paper-instrumented target version); all sit outside this file.

- Row 1 (this file's only concern): VERIFIED, already applied. `interactWithBlocks`
  (ServerExplosion.java:610-643) fires `BlockExplodeEvent` with the non-air block
  list and `yield = 1.0/radius` for DESTROY_WITH_DECAY else `1.0` (:318-319), then
  rebuilds `targetBlocks` from `event.blockList()` and reads `event.getYield()`.
  `BlockExplodeEvent` here exposes mutable `blocks: Vec<BlockPos>` and
  `yield_rate: f32`, `new()` matches the explosion.rs call site, and `#[cancellable]`
  provides `cancelled`. No change needed.
- Row 2 (unloaded-chunk force-load): LEFT OPEN. Vanilla samples via `level.getChunk`
  (ServerExplosion.java:119), which force-loads; this unit's only file is the event
  struct, so the sampler fix belongs in explosion.rs (outside this unit).
- Row 4 (seeded level RNG): LEFT OPEN. Vanilla draws from `level.getRandom()`
  (ServerExplosion.java:410,612,674); the fix belongs in explosion.rs / world/mod.rs
  (outside this unit).
- Row 6 (ClientboundExplodePacket blockCount): LEFT OPEN. Packet construction is in
  world/mod.rs (outside this unit); client-visual only.
- Row 8 (finalizeExplosion particles/sound): LEFT OPEN. Vanilla emits
  `gameEvent(EXPLODE)` (ServerExplosion.java:693) and the sound/particle fan-out is in
  world/mod.rs (outside this unit); client-visual only.
- Rows 3, 5, 7: previously VERIFIED MATCHING/FIXED; unchanged.

No `.rs` source edited, no test seam in this event data holder (a `new()` field
setter test would be a getter/Default-style test the rules exclude). No rustfmt,
cargo or git run per task.

## Result (native-2 triage 2026-11-18)

- Row 2 unloaded-chunk force-load: FIXED (present), explosion.rs:138 get_or_fetch_chunk, test at :1267.
- Row 4 seeded RNG: NOT A GAP (observable). explosion_random seeded from level seed, world/mod.rs:700.
- Row 8 particles/sound: FIXED (present), world/mod.rs:5124-5130.
- Details: docs/investigations/native-2.md
