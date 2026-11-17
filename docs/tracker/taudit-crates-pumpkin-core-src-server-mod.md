# Audit: `crates/pumpkin-core/src/server/mod.rs`

Date: 2026-11-16. Vanilla reference: `net.minecraft.server.MinecraftServer`,
`net.minecraft.server.level.ServerLevel`, `net.minecraft.world.level.border.WorldBorder`.
No decompiled copy is vendored here (`.vendor/` absent); comparisons are against those
classes by name and behaviour. Paper/Spigot behaviour noted where it differs.

## Fixed in this pass

| file:line (before) | vanilla | fix |
|---|---|---|
| `server/mod.rs:683` `sync_level_border` wrote `border_size = new_diameter` and `border_size_lerp_time = 0` | `WorldBorder.createSettings()` persists current size, lerp target and **remaining** lerp time; `WorldBorder.applySettings` resumes the transition | persist `current_diameter()` / `new_diameter` / `remaining_ms()`, and `Worldborder::apply_level_data` resumes it in `World::load` (commit 9161633e3) |
| `server/mod.rs:1078,1093` shutdown logged "Starting worlds" / "Completed worlds" | n/a (log wording) | "Stopping worlds" / "Stopped worlds" (commit c34097b31) |
| `server/mod.rs:1188` stale TODO claiming mob spawn options need an update | `MinecraftServer.updateMobSpawningFlags` sets per-level `spawnEnemies = difficulty != PEACEFUL` | flags are already derived per query in `World::should_spawn_monsters`; comment corrected (commit 0bc5e3adb) |

## Remaining gaps

| # | file:line | vanilla class / behaviour | suggested fix | size |
|---|---|---|---|---|
| 1 | `server/mod.rs:876` (`add_player`) | `ServerPlayer.loadAndSpawn` / `ServerPlayer.load`: gamemode comes from saved `playerGameType`, else `server.getForcedGameType()`, else `LevelData.getGameType()` | pass the saved NBT gamemode (or `defaultgamemode` when absent) into `Player::new` instead of always `defaultgamemode`; verify `Player::read_nbt` does not already restore it | S |
| 2 | `server/mod.rs:1402` (`next_map_id`) | map ids are allocated per `ServerLevel` and the id counter is not global across dimensions | move the counter into `World` (or key it per world) so a map id is unique per dimension, and persist per dimension | M |
| 3 | `server/mod.rs:1578` (`sync_game_time`) | in 1.21 the time packet is sent per level, from the level tick, for every 20th tick (also while frozen) | send `CUpdateTime` per world (each world has its own `level_time`) instead of overworld-only; confirm which side owns the packet before moving the call | M |
| 4 | `server/mod.rs:1614` (`update_tick_times`) | vanilla `MinecraftServer` debug sample sends `SampleType.TICK_TIME` with the raw ring entry (worst/current tick), not `tick_duration.max(target)` | mirror the vanilla 4-value sample layout field by field | S |
| 5 | `server/mod.rs:1649` (`get_tps`) | no vanilla equivalent; Paper clamps `Bukkit.getTPS()` to `[0, tps]` | clamp to `basic_config.tps` here so plugin API and `/tps` agree without the caller clamping | XS |
| 6 | `server/mod.rs:1032` (`remove_player`) | n/a (listing is a Pumpkin feature); join adds only when `PlayerConfig::server_listing` is set | remember whether the player was added to the listing and only then remove, so a gated-off add cannot remove another ticket | S |
| 7 | `server/mod.rs:894` (`add_player`) | vanilla does not derive "played before" from `Pos` | keep the Bedrock fallback but prefer the stored `HasPlayedBefore` tag when present | S |

## Checked, no gap

- `join_rejection` order (full server then duplicate login) matches `PlayerList.canPlayerLogin`; bans/whitelist are checked in the login handlers.
- `set_difficulty` / `set_difficulty_locked` gating on `difficulty_locked` and hardcore forcing `HARD` match `MinecraftServer.setDifficulty`.
- `on_game_rule_changed` mirrors only `IMMEDIATE_RESPAWN`, which is the only rule vanilla mirrors through `ClientboundGameEventPacket`.
- `tick_worlds` skipping `#minecraft:tick`, `/schedule` and autosave while `/tick freeze` is active matches `runsNormally()` gating.
- `saved_login_world` clearing placement for a disabled dimension is a Pumpkin config feature, not a vanilla path.