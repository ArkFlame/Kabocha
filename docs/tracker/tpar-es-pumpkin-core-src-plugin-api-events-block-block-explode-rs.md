# Parity rows for crates/pumpkin-core/src/plugin/api/events/block/block_explode.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-world-expl** [partial/OWN-OPEN] own audit: 5 open, 13 fixed, 15 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-world-expl.md) docs/tracker/taudit-crates-pumpkin-core-src-world-expl.md

## Result

THIRD PASS, cross-file permitted, 2026-11-18. Re-checked the wrapped audit row
(`own/taudit-crates-pumpkin-core-src-world-expl`, 8 numbered rows) against
`.vendor/vanilla-decompiled/net/minecraft/world/level/ServerExplosion.java`
(CraftBukkit/Paper-instrumented target version) and
`.../damagesource/DamageSources.java`. This tpar row is now terminal; no row remains STILL OPEN.

- Row 1 (block list + yield; this file's only concern): VERIFIED MATCHING, no change needed.
  Vanilla `interactWithBlocks` (ServerExplosion.java:610-643) fires the event with the non-air
  block list, default yield `1.0/radius` for `DESTROY_WITH_DECAY` else `1.0` with a finite guard
  (ServerExplosion.java:317-319), then rebuilds `targetBlocks` from `event.blockList()` and re-reads
  `event.getYield()`. `explosion.rs:915-932` mirrors this exactly. `BlockExplodeEvent` exposes mutable
  `blocks: Vec<BlockPos>` and `yield_rate: f32`, its `new()` signature matches the sole call site
  (`explosion.rs:869-875`), and `impl BlockEvent` (`block_explode.rs:41-45`) reports the block at the
  floored centre, matching vanilla `location.getBlock()` (ServerExplosion.java:632) and
  `BlockPos.containing(this.center)`.
- Row 1 sub-gap (Paper exploded-BlockState snapshot): BLOCKED-DATA. Missing subsystem: vanilla reads
  `damageSource.causingBlockSnapshot()` (ServerExplosion.java:633) but Pumpkin's `Explosion` struct
  (`explosion.rs:227-243`) has no causing-block field, has no separate BlockState type (state is
  `Block`), and no Paper `BlockExplodeEvent`/`CraftEventFactory` source exists under `.vendor/` to pin
  the API type; adding a field changes `BlockExplodeEvent::new` arity.
- Row 2 (unloaded-chunk force-load): BLOCKED-DATA. Missing subsystem: async chunk prefetch before the
  sync tick; `load_chunk_for_explosion` already force-loads (`explosion.rs:127-151`), only the
  current-thread-runtime fallback remains, where `block_on`/`block_in_place` would deadlock chunk IO.
- Row 3 (entity blast resistance w/ custom calculator): VERIFIED MATCHING. Vanilla wraps the source
  only when the calculator is null (ServerExplosion.java:316,323-325); Pumpkin applies the hook only
  when `damage_calculator.is_none()` (`explosion.rs:568-572`). Doc premise was inverted.
- Row 4 (seeded level RNG): VERIFIED FIXED. Vanilla draws from `level.getRandom()`
  (ServerExplosion.java:410,612,674); `World::explosion_random` is seeded from the level seed
  (`world/mod.rs:700`) and drives ray power and fire rolls (`explosion.rs:497-501,824-830`).
- Row 5 (`is_small` Keep clause): VERIFIED MATCHING. Vanilla `radius < 2.0 || !interactsWithBlocks()`
  (ServerExplosion.java:770-772) == Pumpkin `power < 2.0 || Keep` (`explosion.rs:395-397`).
- Row 6 (`ClientboundExplodePacket.blockCount`): VERIFIED MATCHING (Mojang vanilla). `explode()`
  returns `toBlow.size()` before the CraftBukkit-only event filter (ServerExplosion.java:694,697,711);
  Pumpkin returns the pre-filter count, locked by `incendiary_block_count_includes_air_positions`.
- Row 7 (damage type): VERIFIED FIXED. `DamageSources.explosion` selects `PLAYER_EXPLOSION` when both
  direct and indirect sources exist (DamageSources.java:251-256); `damage_type()` gates on exactly that.
- Row 8 (`finalizeExplosion` particles/sound): VERIFIED FIXED. Vanilla emits `gameEvent(EXPLODE)`
  (ServerExplosion.java:693); Pumpkin sends particle/sound in the packet (`world/mod.rs:5122-5147`).

No `.rs` source edited: `block_explode.rs` already satisfies Row 1 and the only pure-test seam would
be a getter/Default-style field test excluded by the rules. No rustfmt/cargo/git run per task.
