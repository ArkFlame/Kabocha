# crates/pumpkin-core/src/world/mod.rs — deep audit

Source: crates/pumpkin-core/src/world/mod.rs (9400+ lines, `World`: join/respawn, tick, spawners, block break/place, broadcasts, entity tracker glue).
Compared against vanilla `ServerLevel`, `ServerChunkCache`, `ServerPlayerGameMode`, `PlayerList`, `ItemScatterer`, `EquipmentDrops`; Paper adds no extra behaviour for these paths. 37 TODO/FIXME/vanilla markers at start, 26 substantive gaps reviewed, 4 fixed.

## Fixed in this audit
- :3797 `LevelSettings.permission_level` was a hardcoded `2` (Operator) for every Bedrock player, so non-ops got operator client permissions. Now per-player, matching BDS `StartGamePacket`: `PermissionLvl::Four` -> `Operator`, else `Member` (same mapping already used by `Player::send_abilities_update`, player.rs:4203).
- :3857 `multiplayer_correlation_id` was `Uuid::default()`, identical for every session. BDS generates a per-session UUID; now `Uuid::new_v4()`.
- break_block (:6829): loot was dropped before the block was removed, unlike vanilla `ServerPlayerGameMode.destroyBlock` which calls `level.destroyBlock(pos, false)` and then `playerDestroy` (drops). Loot conditions and neighbour updates used to see the block still present. Now the block is removed first; the now-meaningless post-drop `unchanged()` bail was removed.
- scatter_stack (:7410): re-seeded a fresh `Xoroshiro::from_seed(get_seed())` on every call, so every broken container in a tick produced the same offsets, the same split sizes and stacked identical item entities. Vanilla `ItemScatterer` draws from `level.random`; now uses `World::loot_random` (the level random analogue), taking the lock only around the draws so entity spawn plugin events cannot deadlock on it.

## Remaining gaps
| # | Location | Vanilla | Gap | Size |
|---|---|---|---|---|
| 1 | world/mod.rs:2423 | `LevelChunk.setBlockState` marks the light sections of the changed chunk *and* of every neighbouring chunk the block touches; `ServerChunkCache.blockChanged` / Paper's light engine then pushes light packets for each dirty section | `mark_unsent_light` (:2380) only marks same-chunk sections `y-1..=y+1`. A block on an x/z chunk border changes the neighbour chunk's light, but that chunk never gets a light packet, so clients keep stale light until the section is resent. Fix: in `set_block_state` (:6700) or `mark_unsent_light`, also set the mask for the adjacent chunk_pos when `x & 15` is 0/15 (resp. `z`). | M |
| 2 | world/mod.rs:3274 | `ServerChunkCache.tickChunks` gates `NaturalSpawner.spawnForChunk` on `level.canSpawnEntitiesInChunk(pos)` = `chunkMap.anyPlayerCloseEnoughForSpawning(pos)`, i.e. a player within 128 blocks (no per-player spawn radius set) | Pumpkin ticks every chunk inside the simulation distance and has no player-proximity gate. With `simulation_distance` > 8 a chunk can spawn mobs with no player nearby, which vanilla never does. Fix: filter `spawn_list`/`chunk` by `self.players` distance <= 128 in `tick_chunk` before `spawn_for_chunk`. | M |
| 3 | world/mod.rs:5254 | `ServerPlayer.restoreFrom` re-sends `ClientboundSetExperiencePacket` and one `ClientboundUpdateMobEffectPacket` per surviving effect; the client drops both when it recreates the entity on respawn | Respawn sends neither. `set_experience(0, 0, 0)` in the `!keep_inventory` branch resets `last_sent_xp`, but the `keep_inventory` path and the effect list are never pushed, so a kept-inventory respawn can show a stale xp bar and lose effect icons client-side. `Player::send_difficulty_update` (player.rs:4497) is likewise only called on join (:4461). Fix: after `send_world_info` in the respawn path call `player.send_difficulty_update()` and `player.send_active_effects()`. | S |
| 4 | world/mod.rs:4789 | Bedrock sends `SetSpawnPositionPacket` (`WorldSpawn`) after StartGame so the compass points at the shared spawn | The Java path sends `CPlayerSpawnPosition` at :4659 and respawn sends both editions at :5227, but `spawn_bedrock_player` never sends `CSetSpawnPosition`, so a Bedrock client has no world spawn until its first respawn. Fix: send `CSetSpawnPosition { spawn_position_type: WorldSpawn, .. }` after `CStartGame` in `spawn_bedrock_player`. | S |
| 5 | world/mod.rs:5958 | `Level.getEntities` / `getEntitiesOfClass` return `EnderDragonPart` sub-entities; damage routed through the parent dragon | `add_dragon_parts`/radius collection skips dragon parts (the TODO sits in the entity-radius collector), so hit detection and area effects miss a live dragon's body parts until the dragon fight wrapper exists. Needs the dragon fight rewrite, not a local patch. | L |
| 6 | world/mod.rs:2422 | — | Stale: `broadcast_to_chunk*` already filters on `player.watched_section.is_within_distance`. Only the comment should be deleted. | XS |
| 7 | world/mod.rs:563, :3751, :4240, :4247, :4731 | — | Marker/comment only. The view-distance and simulation-distance values at :4240/:4247 already come from config and match vanilla `ClientboundLogin`; :563 is an empty TODO; :4731 is a logging style note. No behaviour gap. | XS |

## Status (h3 sweep)
- Ender dragon handling (world/mod.rs:5978 still `// TODO: Implement ender dragon handling`): STILL OPEN (M, not changed in this pass).


## Result

Re-verified the remaining/STILL OPEN rows against `.vendor/vanilla-decompiled/net/minecraft/` (26.3 + Paper). No source file was changed: this unit may edit `player.rs` only, and every open row's fix lives in `world/mod.rs` (a file outside this unit). The two player.rs helpers row 3 needs already exist (`Player::send_difficulty_update` :4563 and `Player::send_active_effects` :7281), so there is no in-unit seam to fix.

| # | Size | Result |
|---|---|---|
| 1 | M | STILL OPEN - neighbouring-chunk light-section masks belong in `set_block_state`/`mark_unsent_light` (world/mod.rs), outside this unit. |
| 2 | M | STILL OPEN - player-proximity spawn gate belongs in `tick_chunk` (world/mod.rs), outside this unit. |
| 3 | S | STILL OPEN - verified vanilla `ServerPlayer.restoreFrom` (:3003) sets `lastSentExp = -1` for the next-tick XP resend; Paper comments out the effect re-copy (:3014-3016). Pumpkin's `player.rs` helpers already exist; the missing two calls go after `send_world_info` in the respawn path (world/mod.rs:5309), outside this unit. |
| 4 | S | STILL OPEN - Bedrock `CSetSpawnPosition` belongs in `spawn_bedrock_player` (world/mod.rs), outside this unit. |
| 5 | L | STILL OPEN - EnderDragonPart collection needs the dragon-fight rewrite (world/mod.rs), outside this unit and not a local patch. |
| 6 | XS | STILL OPEN - stale-comment deletion in world/mod.rs, outside this unit. |
| 7 | XS | STILL OPEN - marker/comment-only cleanup in world/mod.rs, outside this unit. |
| h3 | M | STILL OPEN - Ender dragon TODO (world/mod.rs:5978), outside this unit; needs dragon-fight rewrite. |

No .rs file was edited, so no rustfmt was run. No unit test added: there is no behaviour change in this unit, and a helper-method mirror test would be a forbidden constant/derive test.
