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


## Result (FINAL PASS, 2026-11-18)

Read the unit's last Result (THIRD PASS) and re-verified every row against
.vendor/vanilla-decompiled/net/minecraft/world/level/ServerExplosion.java,
.vendor/vanilla-decompiled/net/minecraft/world/damagesource/DamageSources.java
and current Pumpkin source (block_explode.rs:21-45, world/explosion.rs, world/mod.rs).
No STILL OPEN row remains; terminal status per row:

- Row 1 (block list + mutable yield): FIXED / VERIFIED MATCHING. Vanilla
  interactWithBlocks (ServerExplosion.java:610-643) builds the non-air block list
  (:616-622), fires the event, reads event.blockList()/getYield() and rebuilds
  targetBlocks (:629-630,636-643); default yield is 1.0/radius only for
  DESTROY_WITH_DECAY with a finite guard (:317-319). Pumpkin mirrors this at
  world/explosion.rs:915-940 and fire_explode_event (:848-881); BlockExplodeEvent
  exposes mutable blocks/yield_rate and new(BlockPos, Vec<BlockPos>, f32, Arc<World>)
  matches the sole call site (explosion.rs:869-875). impl BlockEvent::get_block
  (block_explode.rs:41-45) returns the block at the floored centre, matching vanilla
  location.getBlock() (ServerExplosion.java:632).
- Row 1 sub-gap (Paper exploded-BlockState snapshot): BLOCKED-DATA. Missing field/subsystem:
  vanilla reads damageSource.causingBlockSnapshot() (ServerExplosion.java:633), but Pumpkin's
  Explosion struct (world/explosion.rs:227-243) has no causing-block field and no separate
  BlockState type (state is Block); no Paper BlockExplodeEvent/CraftEventFactory source under
  .vendor/ pins the API type, and adding the field changes BlockExplodeEvent::new arity at the
  call site. Out of scope for this unit's file (block_explode.rs).
- Row 2 (unloaded-chunk force-load fallback): BLOCKED-DATA. Missing subsystem: an async chunk
  prefetch before the sync tick. load_chunk_for_explosion (explosion.rs:127-151) already
  force-loads via get_or_fetch_chunk; only the current-thread-runtime fallback (:146-149)
  keeps the air fallback because Handle::block_on/block_in_place would deadlock chunk IO.
  Requires a Level-wide subsystem change, out of scope for block_explode.rs.
- Row 3 (entity blast resistance w/ custom calculator): VERIFIED MATCHING. Vanilla wraps the
  source only when the calculator is null (ServerExplosion.java:316,323-325); Pumpkin applies
  get_block_explosion_resistance only when damage_calculator.is_none() (explosion.rs:568-572).
  Doc premise was inverted. No change.
- Row 4 (seeded level RNG): VERIFIED FIXED. Vanilla draws from level.getRandom()
  (ServerExplosion.java:410,612,674); World::explosion_random is seeded from the level seed
  (world/mod.rs:700) and drives ray power and fire rolls (explosion.rs:497-501,824-830).
- Row 5 (is_small Keep clause): VERIFIED MATCHING. Vanilla radius < 2.0 || !interactsWithBlocks()
  (ServerExplosion.java:770-772) == Pumpkin power < 2.0 || Keep (explosion.rs:395-397).
- Row 6 (ClientboundExplodePacket blockCount): VERIFIED MATCHING (Mojang vanilla). Mojang
  returns toBlow.size() before the CraftBukkit-only event filter (ServerExplosion.java:694,697,711);
  Pumpkin returns the pre-filter count (explosion.rs:611,910,984), locked by
  incendiary_block_count_includes_air_positions (:1123). Paper's post-filter count is a
  client-visual CraftBukkit divergence.
- Row 7 (damage type): VERIFIED FIXED. DamageSources.explosion selects PLAYER_EXPLOSION when
  both direct and indirect sources are non-null (DamageSources.java:251-256); damage_type()
  gates on exactly source.is_some() && indirect_source().is_some() (explosion.rs:355-359).
- Row 8 (finalizeExplosion particles/sound): VERIFIED FIXED. Vanilla emits gameEvent(EXPLODE)
  (ServerExplosion.java:693); Pumpkin sends particle/sound in the packet (world/mod.rs:5122-5147).

No .rs source edited (no minimal fix exists inside block_explode.rs; the two remaining gaps
need fields/subsystems in explosion.rs/world/mod.rs, outside this unit's file). No pure unit test
added: the only logic in block_explode.rs is a World-backed lookup in get_block, with no pure
seam (getter-style tests are excluded by the rules). No rustfmt/cargo/git run per task.
