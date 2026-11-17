# Tracker: audit / `crates/pumpkin-core/src/world/mod.rs`

Reviewed at branch `kab/taudit-crates-pumpkin-core-src-world-mod-` (after
`fix(world): draw scattered item drops from the level random`,
`fix(world): send a unique Bedrock multiplayer correlation id`,
`chore(world): drop stale audit markers`).
Vanilla reference: `net.minecraft.server.level.ServerLevel`, `Level`, `ServerChunkCache`,
`NaturalSpawner`, `ServerPlayerGameMode`, `ClientboundSetDefaultSpawnPositionPacket` (1.21.x)
plus Spigot/Paper `World` behaviour where it differs.

## Fixed in this review

| Gap | Change |
|---|---|
| `scatter_stack` re-seeded `Xoroshiro::from_seed(get_seed())` on every call, so every drop from the same block position produced the identical offset, split sizes and velocity (items landed stacked on one spot). Vanilla `ItemEntity`/`Containers.dropItemStack` draw from the shared level `RandomSource`. | Locks `loot_random` (the world-seeded level random, same source `next_loot_seed` uses) at `mod.rs:7004`. |
| Bedrock `StartGame` sent `multiplayer_correlation_id: Uuid::default()` (nil) to every client. Vanilla/Geyser send a session-unique id. | `Uuid::new_v4()` per login at `mod.rs:3639`. |
| Stale markers: bare `// TODO` in `World::load`, already-implemented `// TODO: view distance` / `// TODO: sim view dinstance`, redundant `// TODO Make this unique`. | Removed. |

## Remaining gaps

### 1. Block updates are broadcast to every player in the world - size M

`mod.rs:2249` (`flush_block_updates` / `send_block_updates`). Vanilla routes block changes
through `ChunkHolder`/`ChunkMap` tracking: `ServerLevel.blockUpdated` only reaches players that
track the affected chunk section, and `ClientboundSectionBlocksUpdatePacket` is per watching
player. Pumpkin groups by chunk section but then sends to every player in the world, so distant
players receive (and decode) thousands of block updates per tick. Fix: send through
`entity_tracker`/chunk-tracking player list instead of `broadcast_to_all`; reuse
`broadcast_to_chunk_editioned` where the packet type allows.

### 2. No light update packet after a block change - size M

`mod.rs:2250`. Vanilla `LevelChunk.setBlockState` marks the section dirty and
`ChunkHolder.broadcastChanges` sends `ClientboundLightUpdatePacket` for every section whose
light changed (broken block next to a light source/opaque face). Pumpkin only queues block
state changes, so clients keep stale lighting until the chunk is reloaded. Fix: have the
lighting pass record changed sections and flush them with the block updates.

### 3. `canSpawnEntitiesInChunk` check missing - size M

`mod.rs:3024` (`tick_spawning_chunk`). Vanilla/Paper refuse natural spawns in a chunk that has
no player within the spawn range (`ServerLevel.canSpawnEntitiesInChunk` / `NaturalSpawner`
chunk checks) and skip chunks whose `noActionTime` is high. Pumpkin calls
`spawn_for_chunk` unconditionally for every spawning chunk. Fix: gate on the player-tracking
distance of the chunk (or the existing `natural_spawn_chunk_cache`) before spawning.

### 4. Bedrock permission level / player permissions hardcoded - size S

`mod.rs:3579`. Bedrock uses its own 0..3 permission levels (`playerPermissions` in
`PlayerListPacket`, `permissionLevel` in `StartGame`), and op level mapping differs from Java.
Pumpkin sends `permission_level: 2` for everyone. Fix: derive from the player's op level via a
Java->Bedrock mapping (0/1/2/3) and keep `PlayerListPacket` in sync.

### 5. World spawn / compass position not sent - size M

`mod.rs:4588` (`send_world_info`). Vanilla sends
`ClientboundSetDefaultSpawnPositionPacket(level.getSharedSpawnPos(), level.getSharedSpawnAngle())`
on login and on `/setworldspawn`; the packet type does not exist in `pumpkin-protocol` yet, so
compass and respawn-compass targets stay at 0,0. Fix: add the packet (id, `BlockPos`, angle),
send it in `send_world_info`, and rebroadcast in the world-spawn command/API.

### 6. Respawn does not resync difficulty, experience and effects - size S

`mod.rs:5048` (`respawn_player`). Vanilla `ServerPlayer.restoreFrom` plus
`ServerGamePacketListenerImpl` resend `ClientboundSetExperiencePacket`,
`ClientboundSetHealthPacket` (already covered by `reset_state`), the difficulty
(`ClientboundChangeDifficultyPacket`) and every active `ClientboundUpdateMobEffectPacket`.
Pumpkin sends permission level and hunger only, so after death the client shows a stale exp bar
and stale difficulty. Fix: resend exp + difficulty + mob effects after `reset_state`.

### 7. Ender dragon handling in entity queries - size M

`mod.rs:5577` (`extend_entities_where`). Vanilla `EnderDragon`/`EnderDragonPart` are filtered
out of most entity queries (parts are not selectable targets and are excluded from
`getEntities` results unless explicitly requested); Pumpkin returns entity parts as ordinary
entities, so `extend_entities_where` hands dragon/end-crystal parts to goals and commands.
Fix: skip `EntityBase` implementations flagged as multipart sub-entities unless the caller asks
for them.

### 8. `playerDestroy` ordering vs block removal - size S

`mod.rs:6414` (`break_block`). Vanilla `ServerPlayerGameMode.destroyBlock` calls
`blockState.onDestroyedByPlayer(...)` (which removes the block) and only then
`block.playerDestroy(...)` for the loot, so loot context sees the block already gone and
`playerDestroy` cannot be cancelled by a re-set. Pumpkin drops loot first and then sets the
block, relying on the `unchanged()` re-check. Fix: set the block (and broadcast) before
`drop_loot`, keeping the `unchanged()` guard for plugin re-sets.

### 9. Structured logging for join messages - size XS

`mod.rs:4530`. `info!("{}", event.join_message.to_pretty_console())` prints the formatted
component. Vanilla has no equivalent; this is a housekeeping item (use
`info!(player = %name, "connected")`).
