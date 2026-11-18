# Final rows from docs/tracker/tblocks-creaking-heart.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: creaking_heart.rs

FIXED: block entity tick re-syncs state every 20 ticks (block/entities/creaking_heart.rs tick); awake heart with a player in 32 blocks spawns a bound creaking (try_spawn_creaking); is_protector requires a bound UUID (no fallback). UNVERIFIED: spawn distance 32 and free-spot search are from memory. STILL OPEN: idle heartbeat, teardown of bound creaking when heart goes uprooted, break sound routing.
out-of-unit files were touched. All prior STILL OPEN rows are resolved; open = 0.

## Result
FINAL RESULTS. All rows closed; open = 0. Vanilla sources:
.vendor/vanilla-decompiled/net/minecraft/world/level/block/CreakingHeartBlock.java and
.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/CreakingHeartBlockEntity.java.

- block entity tick re-syncs state every 20 ticks: FIXED / VERIFIED. serverTick resets ticker to
  20-24 (CreakingHeartBlockEntity.java:101-103) and calls updateCreakingState; CreakingHeartBlockEntity::tick
  calls CreakingHeartBlock::update_state every 20 world-age ticks.
- awake heart + player in 32 blocks spawns a bound creaking: FIXED / VERIFIED. serverTick checks
  isSpawningMonsters, non-peaceful, AWAKE, player within 32.0, spawnProtector, setCreakingInfo then plays
  CREAKING_SPAWN + CREAKING_HEART_SPAWN (lines 111-121); try_spawn_creaking matches and binds the UUID.
- is_protector requires a bound UUID (no fallback): FIXED / VERIFIED. is_protector uses
  is_some_and(|uuid| uuid == creaking_uuid).
- spawn distance 32 and free-spot search (previously UNVERIFIED): VERIFIED. PLAYER_DETECTION_RANGE = 32
  (line 46), player check 32.0 (line 114); SPAWN_RANGE_XZ = 16, SPAWN_RANGE_Y = 8,
  ATTEMPTS_PER_SPAWN = 5 (lines 49-51), try_spawn_mob OnTopOfColliderNoLeaves with check_collisions true.
  No code change.
- idle heartbeat (Sound::BlockCreakingHeartIdle): VERIFIED, no code change. Only path is client-side
  animateTick: 1-in-16, CREAKING_ACTIVE, state != UPROOTED, isSurroundedByLogs
  (CreakingHeartBlock.java:64-73); no server tick plays it. Per client-only sound rule, skipped.
- teardown of bound creaking when heart goes uprooted: FIXED / VERIFIED. updateCreakingState uproots only
  when no required logs AND creakingInfo == null (lines 140-149). update_state should_uproot(!has_logs &&
  !bound) matches; a bound heart never reverts to UPROOTED. Pure test bound_heart_never_uproots_without_logs.
- break sound routing: FIXED / VERIFIED. CreakingHeartBlock has no break-sound override (playerWillDestroy
  only removeProtector + XP, lines 170-178). Pumpkin plays the break sound from the block SoundType; the old
  broken() play_sound double-played and was removed; XP-drop logic retained.
- spawn sound timing (additional): FIXED / VERIFIED. Both sounds play in serverTick only after a protector
  spawns (lines 118-120); removed BlockCreakingHeartSpawn from update_state, added both to try_spawn_creaking.
- bound protector teardown on heart removal (additional): FIXED / VERIFIED. preRemoveSideEffects calls
  removeProtector(null) (lines 313-315); on_block_replaced override + remove_protector helper tear down and
  unbind the creaking. Despawn conditions also match lines 129-133 (not active && !persistent, >34 blocks,
  stuck), DISTANCE_CREAKING_TOO_FAR = 34 (line 48).

Touched files (prior passes): crates/pumpkin-core/src/block/entities/creaking_heart.rs and
crates/pumpkin-core/src/block/blocks/creaking_heart.rs. This pass changed only this doc. No generated data,
WIT, cross-crate or new source files. Cargo/git/rustfmt not run (worker contract); compile/test UNVERIFIED.
