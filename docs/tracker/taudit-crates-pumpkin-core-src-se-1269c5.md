# DEEP AUDIT: crates/pumpkin-core/src/server/mod.rs

Status: audited, all TODO/FIXME markers resolved.
Commits: d5ff69ec7 (listing online count), 503ea7a9c (shutdown kick message),
83a08e165 (difficulty docs).

## Fixed
- mod.rs:1254-1263: CachedStatus::add_player only counted players that allow server listing, so
  the status players.online ignored everyone who disabled listing. Vanilla
  MinecraftServer.buildServerStatus uses playerList.getPlayerCount() for online and
  ServerPlayer.allowsListing() only for sample. Now online counts every join and the sample stays
  opt-in (connection_cache.rs).
- mod.rs:1243 and lib.rs:623: shutdown kick used literal 'Server stopped'; vanilla
  MinecraftServer.stopServer disconnects with multiplayer.disconnect.server_shutdown
  ('Server closed'). Now uses the translation key, which also resolves for Bedrock via get_text.
- mod.rs:1503: stale set_difficulty note about mob spawn options. Spawning reads the level
  difficulty per attempt (ServerLevel.getCurrentDifficultyAt), vanilla has no extra refresh.
  Docs corrected (also the Option<bool> vs bool mismatch).

## Remaining gaps (not fixed)

1. mod.rs:1141 load_player_world falls back to worlds.first(), not the overworld.
   Vanilla places a joining player in Level.OVERWORLD when the saved dimension is gone. With
   plugin-created worlds this can join a non-overworld world. Size S.
   Fix: get_world_from_dimension(&Dimension::OVERWORLD).

2. mod.rs:1769 next_map_id: fetch_add returns the id but stores self.map_id.load() (the
   incremented value) into level data. Verify against vanilla MinecraftServer.getNextMapId,
   which persists the id handed out; if so one map id is skipped per restart. Size XS.

3. mod.rs:1673 get_random_player uses rand::rng() (thread RNG); vanilla
   MinecraftServer.getRandomPlayer draws from overworld.getRandom(). No visible parity difference,
   but seeded tests differ. Size XS.

4. mod.rs:1982 update_tick_times builds the debug sample as [max(tick,target), tick, 0, idle].
   Verify field order/units against vanilla ClientboundDebugSamplePacket TICK_TIME sample.
   Client-only display. Size S.

5. mod.rs:1171 add_player rechecks only max players and duplicate login under the join lock;
   bans and whitelist are enforced earlier in the connection handlers (net::can_not_join).
   Vanilla PlayerList.canPlayerLogin does every check atomically inside the same lock, so a
   plugin path calling add_player directly skips ban/whitelist. Size S.

## Checked, no gap
- 926/952/1915 comments match behaviour (level.dat time/border sync, 20-tick time sync).
- sync_level_time / sync_level_border match the vanilla level.dat fields.
- suspend_player_flushes / resume_player_flushes match
  ServerCommonPacketListenerImpl.suspendFlushing / resumeFlushing.
- save_all order (world info, players, advancements, persistent data, worlds) is crash-safe.
