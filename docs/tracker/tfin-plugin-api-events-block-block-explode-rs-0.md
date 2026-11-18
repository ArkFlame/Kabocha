# Final rows from docs/tracker/tpar-es-pumpkin-core-src-plugin-api-events-block-block-explode-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

`.../damagesource/DamageSources.java`. This tpar row is now terminal; no row remains STILL OPEN.
No STILL OPEN row remains; terminal status per row:

## Result

FINAL PASS, 2026-11-18. Re-verified every row against
.vendor/vanilla-decompiled/net/minecraft/world/level/ServerExplosion.java and
.vendor/vanilla-decompiled/net/minecraft/world/damagesource/DamageSources.java
and current Pumpkin source (plugin/api/events/block/block_explode.rs:21-45,
world/explosion.rs, world/mod.rs). No source fix was needed: the two gaps
need missing subsystems (named below), not a change inside this unit file.
Terminal status per row (no STILL OPEN remains):

- Row 1 (block list + mutable yield): VERIFIED MATCHING. Vanilla
  interactWithBlocks (ServerExplosion.java:610-643) builds the non-air list
  (:616-622), fires the event, reads event.blockList()/getYield() and rebuilds
  targetBlocks (:636-643); default yield is 1/radius only for DESTROY_WITH_DECAY
  with a finite guard (:317-319). Pumpkin mirrors this at
  world/explosion.rs:915-940 and fire_explode_event (:848-881);
  BlockExplodeEvent exposes mutable blocks/yield_rate and new(BlockPos,
  Vec<BlockPos>, f32, Arc<World>) matches the sole call site (:869-875).
  impl BlockEvent::get_block (block_explode.rs:41-45) returns the block at the
  floored centre, matching vanilla location.getBlock() (:632).
- Row 1 sub-gap (Paper exploded-BlockState snapshot): BLOCKED-DATA. Missing
  subsystem: vanilla reads damageSource.causingBlockSnapshot()
  (ServerExplosion.java:633), but Pumpkin Explosion struct
  (world/explosion.rs:227-243) has no causing-block field and no separate
  BlockState type (state is Block); no Paper BlockExplodeEvent/CraftEventFactory
  source exists under .vendor/ to pin the API type, and adding the field changes
  BlockExplodeEvent::new arity at the call site. Grep confirmed no causing_block
  concept anywhere in crates/.
- Row 2 (unloaded-chunk force-load fallback): BLOCKED-DATA. Missing subsystem:
  an async chunk prefetch before the sync tick. load_chunk_for_explosion
  (world/explosion.rs:127-151) already force-loads via get_or_fetch_chunk;
  only the current-thread-runtime fallback (:140-148) keeps the air fallback
  because Handle::block_on/block_in_place would deadlock chunk IO. Requires a
  Level-wide subsystem change, out of scope for block_explode.rs.
- Row 3 (entity blast resistance w/ custom calculator): VERIFIED MATCHING.
  Vanilla wraps the source only when the calculator is null
  (ServerExplosion.java:316,323-325); Pumpkin applies
  get_block_explosion_resistance then the source adjustment only when
  damage_calculator.is_none() (world/explosion.rs:567-579). Doc premise inverted.
- Row 4 (seeded level RNG): VERIFIED FIXED. Vanilla draws from level.getRandom()
  (ServerExplosion.java:410,612,674); World::explosion_random is seeded from
  the level seed (world/mod.rs:700) and drives ray power and fire rolls
  (world/explosion.rs:507-509,824-830).
- Row 5 (is_small Keep clause): VERIFIED MATCHING. Vanilla
  radius < 2.0 || !interactsWithBlocks() (ServerExplosion.java:770-772), where
  interactsWithBlocks() is blockInteraction != KEEP (:725-727), equals
  Pumpkin power < 2.0 || matches!(block_interaction, Keep)
  (world/explosion.rs:403-404).
- Row 6 (ClientboundExplodePacket blockCount): VERIFIED MATCHING (Mojang vanilla).
  Vanilla returns toBlow.size() before the CraftBukkit-only event filter
  (ServerExplosion.java:694,697,711); Pumpkin returns the pre-filter
  to_blow_count (world/explosion.rs:611,911,985), locked by
  incendiary_block_count_includes_air_positions (:1124). Paper post-filter count
  is a client-visual CraftBukkit divergence.
- Row 7 (damage type): VERIFIED FIXED. DamageSources.explosion selects
  PLAYER_EXPLOSION when both direct and indirect sources are non-null
  (DamageSources.java:251-256); damage_type() gates on exactly
  source.is_some() and indirect_source().is_some() (world/explosion.rs:355-362).
- Row 8 (finalizeExplosion particles/sound): VERIFIED FIXED. Vanilla emits
  gameEvent(EXPLODE) (ServerExplosion.java:693); Pumpkin sends particle/sound
  in the packet (world/mod.rs:5122-5147).

No .rs source edited: no minimal fix exists for the two gaps inside this
unit file, and they require fields/subsystems outside it. No pure unit test
added: block_explode.rs only has a World-backed lookup in get_block, with no
pure seam (getter-style tests are excluded by the rules). No rustfmt/cargo/git
run per task.
