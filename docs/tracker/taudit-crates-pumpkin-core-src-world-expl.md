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


## Result (native-4 cross-file pass)
- Row 6 (ClientboundExplodePacket blockCount): FIXED (present). `world/mod.rs:5140` sends `result.block_count`; `world/explosion.rs:822-833` counts air positions when fire is created, covered by test `incendiary_block_count_includes_air_positions` (explosion.rs:1114).
- Row 8 (finalizeExplosion particles/sound): FIXED (present), see `world/mod.rs:5124-5130`.

## Result (block_explode.rs pass 2026-11-18)

Checked the STILL OPEN rows of this unit against
`.vendor/vanilla-decompiled/net/minecraft/world/level/ServerExplosion.java` (Paper/CraftBukkit
instrumented) and `.../block/state/BlockBehaviour.java`.

- Row 1 (block list + yield): VERIFIED applied upstream of this file. Vanilla
  `interactWithBlocks` (:610-643) fires the event with the non-air list and
  `yield = 1.0/radius` only for `DESTROY_WITH_DECAY` (:317-319), then rebuilds `targetBlocks`
  from `event.blockList()` and re-reads `event.getYield()`; `explosion.rs:905-923` already does
  exactly that. FIXED in this pass: `BlockExplodeEvent` was missing the `BlockEvent` trait impl
  that Paper's `org.bukkit.event.block.BlockExplodeEvent` inherits (`getBlock()` = block at the
  explosion centre, `ServerExplosion.java:632`), so plugins resolving the event through the
  generic block-event path got nothing. Added `impl BlockEvent for BlockExplodeEvent`
  (`block_explode.rs:39-44`); every other block event in this crate already has it.
- Row 2 (unloaded-chunk force-load): LEFT OPEN, not this file. `load_chunk_for_explosion`
  (`explosion.rs:127-151`) already force-loads via `get_or_fetch_chunk`; the remaining
  current-thread-runtime fallback is a `Level` change outside this unit.
- Row 4 (seeded level RNG): LEFT OPEN, not this file. `world.explosion_random` exists
  (`world/mod.rs:376,700`) and is already used at `explosion.rs:497-502,822-831`.
- Row 6 (`ClientboundExplodePacket.blockCount`): LEFT OPEN, not this file (packet built in
  `world/mod.rs`). Vanilla `explode()` returns `toBlow.size()` after the event filter, while
  Pumpkin returns the pre-filter count; the fix belongs to `explosion.rs`/`world/mod.rs`.
- Row 8 (`finalizeExplosion` particles/sound): LEFT OPEN, not this file (`world/mod.rs`).
- New sub-gap recorded, not fixed: Paper's `BlockExplodeEvent` also carries the exploded
  `BlockState` snapshot (`ServerExplosion.java:632-634`); adding a field changes
  `BlockExplodeEvent::new` arity, and the only call site is `explosion.rs`, outside this unit.
- Rows 3, 5, 7: unchanged from earlier passes (VERIFIED MATCHING / FIXED).

Changed: `crates/pumpkin-core/src/plugin/api/events/block/block_explode.rs`.
No pure unit test added: the only new behaviour is a world lookup inside a data holder, with no
seam that is not a World-backed integration test. No rustfmt/cargo/git run (per task brief).


## Result (SECOND PASS, cross-file permitted, 2026-11-18)

Allowed to touch any cross-file row this pass; re-verified every STILL OPEN row against
`.vendor/vanilla-decompiled/net/minecraft/world/level/ServerExplosion.java`,
`.../damagesource/DamageSources.java` and the current Pumpkin source. `explosion.rs` is being
edited concurrently by another worker, so no shared-file edit was attempted.

- Row 1 (block list + yield, block_explode.rs): VERIFIED applied and correct. Vanilla
  `interactWithBlocks` (:610-643) fires the event with the non-air block list, reads
  `event.blockList()`/`event.getYield()` and rebuilds the target list; `explosion.rs:915-923`
  mirrors this, and `BlockExplodeEvent` exposes mutable `blocks`/`yield_rate`. `BlockEvent`
  impl present (:41-44). New sub-gap (Paper's `BlockState` snapshot at :632-634) LEFT OPEN:
  no `getExplodedBlockState` API seam in this crate and adding a field changes `new()` arity
  at the `explosion.rs` call site (concurrently edited); a spec-only change without the
  CraftEventFactory source would be a guess.
- Row 2 (unloaded-chunk force-load): force-load is present and tested
  (`load_chunk_for_explosion` at explosion.rs:127-151, test at :1267). Only the
  current-thread-runtime fallback remains (:140-149), which keeps the air fallback to avoid a
  `block_on`/`block_in_place` deadlock. Fixing it needs an async chunk prefetch before the sync
  tick (a Level-wide subsystem change), so it is BLOCKED here, not a minimal edit.
- Row 3 (entity blast resistance with custom calculator): VERIFIED MATCHING. Pumpkin applies
  `source.get_block_explosion_resistance` only when `damage_calculator.is_none()`
  (explosion.rs:568-572), matching `ServerExplosion` wrapping the source only when the
  calculator is null. Doc premise was inverted; no change.
- Row 4 (seeded level RNG): VERIFIED FIXED. `World::explosion_random` is seeded from
  `level.seed.0` (world/mod.rs:700) and drives ray power (explosion.rs:497-501) and fire rolls
  (:824-830), matching vanilla `level.getRandom()` (:410,674).
- Row 5 (`is_small` Keep clause): VERIFIED MATCHING. Vanilla `radius < 2.0 || !interactsWithBlocks()`
  (:770-772) == Pumpkin `power < 2.0 || Keep` (explosion.rs:395-397). No change.
- Row 6 (`ClientboundExplodePacket.blockCount`): LEFT OPEN as a deliberate client-visual
  divergence. Vanilla Mojang `explode()` returns `toBlow.size()` from
  `calculateExplodedPositions()` BEFORE the (CraftBukkit-only) event filter, including air
  positions when fire (:371-468, :694, :711); Pumpkin returns the same pre-filter count
  (explosion.rs:611,984), locked by `incendiary_block_count_includes_air_positions` (:1123).
  Paper's post-event-filter count (:640-643) would regress that test and is client-visual only.
- Row 7 (damage type): VERIFIED FIXED. `DamageSources.explosion` selects `PLAYER_EXPLOSION`
  when both direct and indirect sources exist (DamageSources.java:251-256); `damage_type()`
  gates on exactly that (explosion.rs:346-354). No change.
- Row 8 (`finalizeExplosion` particles/sound): VERIFIED FIXED in world/mod.rs:5122-5147
  (packet carries particle/sound and `play_sound = !source.is_silent()`). No change.

No `.rs` source edited this pass (no safe minimal fix found in the designated file;
`explosion.rs` is concurrently owned), so no rustfmt/cargo/git run.
