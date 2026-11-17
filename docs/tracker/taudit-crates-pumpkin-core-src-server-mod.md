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
| 4 | `server/mod.rs:1614` (`update_tick_times`) | vanilla `MinecraftServer` debug sample sends `SampleType.TICK_TIME` with the raw ring entry (worst/current tick), not `tick_duration.max(target)` | mirror the vanilla 4-value sample layout field by field | S - STILL OPEN: vanilla debug sample source not decompiled here |
| 5 | `server/mod.rs:1649` (`get_tps`) | no vanilla equivalent; Paper clamps `Bukkit.getTPS()` to `[0, tps]` | clamp to `basic_config.tps` here so plugin API and `/tps` agree without the caller clamping | XS |
| 6 | `server/mod.rs:1032` (`remove_player`) | n/a (listing is a Pumpkin feature); join adds only when `PlayerConfig::server_listing` is set | remember whether the player was added to the listing and only then remove, so a gated-off add cannot remove another ticket | S - FIXED (UNVERIFIED): `Player::listed_in_sample` records the ticket per connection; `remove_player` drops the sample entry only when set |
| 7 | `server/mod.rs:894` (`add_player`) | vanilla does not derive "played before" from `Pos` | keep the Bedrock fallback but prefer the stored `HasPlayedBefore` tag when present | S |

## Checked, no gap

- `join_rejection` order (full server then duplicate login) matches `PlayerList.canPlayerLogin`; bans/whitelist are checked in the login handlers.
- `set_difficulty` / `set_difficulty_locked` gating on `difficulty_locked` and hardcore forcing `HARD` match `MinecraftServer.setDifficulty`.
- `on_game_rule_changed` mirrors only `IMMEDIATE_RESPAWN`, which is the only rule vanilla mirrors through `ClientboundGameEventPacket`.
- `tick_worlds` skipping `#minecraft:tick`, `/schedule` and autosave while `/tick freeze` is active matches `runsNormally()` gating.
- `saved_login_world` clearing placement for a disabled dimension is a Pumpkin config feature, not a vanilla path.
## Gapwave2-2 status

- row 6 (`remove_player` sample ticket): FIXED (UNVERIFIED). `ConnectionCache::add_player` returns whether it pushed the ticket; the join stores it on the player and leave only removes a ticket that connection holds. No unit test: `Player` construction is too heavy for a pure test; needs a gametest or live duplicate-UUID join check.
- rows 1-4, 7: STILL OPEN.

## Result

Re-checked against `ServerPlayer.load` / `WorldBorder` / `MinecraftServer` behaviour described
above. No decompiled vanilla copy and no Paper/Spigot source are present under `.vendor/`, so
rows that needed exact vanilla layouts or cross-file refactors were left open.

| # | result | detail |
|---|---|---|
| 1 | FIXED (UNVERIFIED) | `Player::read_nbt` already restores `playerGameType`, so the final saved mode was correct; the join now also constructs the player with the saved mode (or the forced/level default) to avoid a transient default gamemode, and `force_gamemode` overrides the saved value via the NBT copy before `read_nbt` applies abilities. Pure unit tests added in `server/mod.rs`. |
| 2 | OPEN | Needs a per-`World` map-id counter and per-dimension persistence in `world/` (outside this unit: `server/mod.rs` only). |
| 3 | OPEN | `send_game_time_sync` in `world/time.rs` already broadcasts to every world's players using the caller's world age; sending a per-level `gameTime` needs that helper changed (outside this unit) and the exact 1.21 packet source is not vendored here. |
| 4 | OPEN | Vanilla `DebugSampleType.TICK_TIME` layout is not decompiled here; the current 4-value sample cannot be verified or mirrored safely. |
| 5 | no change | `get_tps` already clamps to the tick-rate target, matching Paper's `[0, tps]` clamp. |
| 6 | no change | Already fixed in a prior pass (sample ticket recorded per connection). |
| 7 | FIXED (UNVERIFIED) | `HasPlayedBefore` is now read from the saved tag (via `read_nbt`) and the Pos-based fallback applies only when the tag is absent, so a stored `false` is no longer overwritten. Pure unit tests added. |

Files touched: `crates/pumpkin-core/src/server/mod.rs` (imports, `add_player`, two pure helpers,
`#[cfg(test)] mod tests`). `rustfmt --edition 2024` ran on that file with exit 0. No build or test
run: shell/cargo access is blocked in this session; orchestrator verification required.

