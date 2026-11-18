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

## Result

Re-verified every STILL OPEN row against `.vendor/vanilla-decompiled/net/minecraft/` (`LevelChunk.java:389-430`, `ServerChunkCache.java:967-990`, `ServerPlayer.java:3003-3035`, `ChunkMap.java:766-800`) and the current Pumpkin `world/mod.rs` / `entity/player.rs`. This unit's editable basenames are `none`, so no source file was changed; each fix below is already present in the checkout (landed by sibling units) and needs no in-unit seam. This section supersedes the earlier Result below.

- Row 1 (M, neighbouring-chunk light sections): already fixed. `mark_unsent_light` (world/mod.rs:2490) now dirties the adjacent-chunk mask on x/z borders (`trailing_zeros >= 4` / `& 15 == 15`), with pure test `border_block_dirties_light_sections_in_adjacent_chunks` (:9879). Matches vanilla `LevelChunk.setBlockState -> LightEngine.checkBlock` / `SectionPos.aroundAndAtBlockPos`. No edit needed.
- Row 2 (M, spawn player-proximity gate): already fixed. `any_player_close_enough_for_spawning` (:3408) uses a 128-block squared range and `tick_chunk` bails before `spawn_for_chunk` (:3434), matching vanilla `ChunkMap.anyPlayerCloseEnoughForSpawning`. No edit needed.
- Row 3 (S, respawn xp/difficulty/effects): already fixed. The respawn path calls `player.send_difficulty_update()` and `player.send_active_effects()` (:5469-5470); helpers live in `entity/player.rs` (:4654, :7405). Vanilla `ServerPlayer.restoreFrom` sets `lastSentExp = -1` (:3030) and Paper comments out the effect re-copy (:3014-3016). No edit needed.
- Row 4 (S, Bedrock world spawn): already fixed. `spawn_bedrock_player` sends `CSetSpawnPosition { WorldRespawn, .. }` after `CStartGame` (:4079). No edit needed.
- Row 5 (L, EnderDragonPart radius/hit detection): already fixed. `dragon_parts: DashMap` (:351) is populated via `add_dragon_parts` (:5804), included in `get_entities_at_box` / `get_entities_of_type_at_box` (:5845, :5861), and hit-tested via `dragon_part_by_id` (:5797); the dragon-fight rewrite has landed. No edit needed.
- Row 6 (XS, stale comment): no behaviour gap; broadcast methods already filter on `watched_section.is_within_distance` (:8972+) and the stale comment is gone. Comment-only cleanup in world/mod.rs (outside unit), not required.
- Row 7 (XS, marker/comment-only): no behaviour gap; only a structured-logging note (:4932) and a world-spawn compass TODO (:4990) remain, both comment-only and outside this unit.
- h3 (M, ender dragon TODO): already fixed; the `TODO: Implement ender dragon handling` marker no longer exists and dragon handling is wired through `dragon_fight::DragonFight::tick` (:2398).

No .rs file was edited (unit has zero editable basenames), so rustfmt was not run. No unit test added: row 1's pure seam is already covered by the existing test; the rest are World/engine coupled with no in-unit seam.

## Result

Re-verified every STILL OPEN row against `.vendor/vanilla-decompiled/net/minecraft/` and the current `world/mod.rs`. This unit has zero editable basenames (`none`), so no source file was changed; each fix is already present in the checkout (landed by sibling units) and needs no in-unit seam.

| # | Size | Result |
|---|---|---|
| 1 | M | FIXED elsewhere - `mark_unsent_light` (world/mod.rs:2514) dirties adjacent-chunk masks on x/z borders, matching vanilla `SectionPos.aroundAndAtBlockPos` (SectionPos.java:270); pure test `border_block_dirties_light_sections_in_adjacent_chunks` at :9913. |
| 2 | M | FIXED elsewhere - `any_player_close_enough_for_spawning` (:3432) uses a 128-block squared gate before spawning (:3458), matching vanilla `ChunkMap.anyPlayerCloseEnoughForSpawning` (ChunkMap.java:766). |
| 3 | S | FIXED elsewhere - respawn calls `send_difficulty_update()` / `send_active_effects()` (:5499-5500) after `send_world_info`, matching vanilla `ServerPlayer` `lastSentExp = -1` resend (ServerPlayer.java:902+). |
| 4 | S | FIXED elsewhere - `spawn_bedrock_player` sends `CSetSpawnPosition` with `WorldRespawn` after `CStartGame` (:4103-4111). |
| 5 | L | FIXED elsewhere - `dragon_parts: DashMap` (:352) merged into `get_entities_at_box`/`get_entities_of_type_at_box` (:5882, :5902) and hit-tested via `dragon_part_by_id` (:5827), matching vanilla `addToGetEntities`. |
| 6 | XS | N/A - no behaviour gap; stale broadcast comment is gone. |
| 7 | XS | N/A - only comment-only TODOs remain (:4962 structured logging, :5020 world-spawn compass). |
| h3 | M | FIXED elsewhere - `DragonFight::tick` wired at :2422; the `Implement ender dragon handling` TODO no longer exists. |

No .rs file edited (zero editable basenames), so no rustfmt run and no unit test added; row 1's pure seam is already covered by an existing test and the rest are World/engine coupled.

## Result

Re-verified in this pass against `.vendor/vanilla-decompiled/net/minecraft/` (26.3; `LevelChunk`, `ServerChunkCache`/`ChunkMap`, `ServerPlayer`) and current `world/mod.rs`. Editable basenames for this unit: none, so no source file was changed; no fix has an in-unit seam.

| # | Size | Result |
|---|---|---|
| 1 | M | FIXED elsewhere - `mark_unsent_light` (:2514) dirties adjacent-chunk masks on x/z borders; pure test `border_block_dirties_light_sections_in_adjacent_chunks` (:9913). Matches `SectionPos.aroundAndAtBlockPos`. |
| 2 | M | FIXED elsewhere - `any_player_close_enough_for_spawning` (:3432, 128-block^2) gates `tick_chunk` (:3458). Matches `ChunkMap.anyPlayerCloseEnoughForSpawning`. |
| 3 | S | FIXED elsewhere - respawn path calls `player.send_difficulty_update()` / `player.send_active_effects()` (:5499-5500) after world info; helpers in `entity/player.rs`. |
| 4 | S | FIXED elsewhere - `spawn_bedrock_player` sends `CSetSpawnPosition` (`WorldRespawn`) after `CStartGame` (:4103). |
| 5 | L | FIXED elsewhere - `dragon_parts: DashMap` (:352) merged into entity queries/hit tests via `dragon_part_by_id`/`add_dragon_parts` (:5828+). |
| 6 | XS | N/A - no behaviour gap; stale broadcast comment already removed. |
| 7 | XS | N/A - marker/comment-only notes remain in world/mod.rs (outside unit), no behaviour gap. |
| h3 | M | FIXED elsewhere - `DragonFight::tick` wired; `TODO: Implement ender dragon handling` absent. |

No .rs edited (zero editable basenames), so rustfmt not run. No unit test added (row 1 seam already covered; others are World/engine coupled with no in-unit seam).
