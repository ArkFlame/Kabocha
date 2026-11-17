# server/mod.rs - deep audit

Source: `crates/pumpkin-core/src/server/mod.rs` (2058 lines). Vanilla reference:
`net/minecraft/server/MinecraftServer.java`, `net/minecraft/server/players/PlayerList.java`,
`net/minecraft/world/level/border/WorldBorder.java`, `net/minecraft/world/level/storage/PrimaryLevelData.java`.
No `TODO`/`FIXME` markers in the file; the 8 greps are `vanilla` mentions in doc comments.

## Fixed in this audit
- `spawn_world` fired `WorldInitEvent` only. Startup worlds fire `WorldInitEvent` and
  `WorldLoadEvent` (`mod.rs:516-530`), so a world created at runtime never sent the load event.
  Both are fired now (`mod.rs:898-901`).
- `close_admission` did not hold `player_join_lock`, contradicting its own doc: a join could pass
  the `admit_player` check, get admitted after the shutdown kick pass read `get_all_players`, and
  stay connected. It now takes the join lock before setting the flag (`mod.rs:1366-1377`).
- `get_tps` returned `1000 / mspt` where `mspt` is work time only (`ticker.rs:125` excludes the
  idle wait), so a healthy 2 ms tick reported 500 TPS. Capped at the configured tick rate
  (`mod.rs:2015-2025`); `/tps` already capped externally.

## Remaining gaps
| # | Location | Vanilla | Gap | Size |
|---|---|---|---|---|
| 1 | `mod.rs:980-992` (`sync_level_border`), `world/border.rs:92` (`apply_level_data`), `world_info/anvil.rs:181,321` | `WorldBorder.Settings#sizeLerpTime` is a game-time deadline; `applySettings` subtracts the current game time and resumes the lerp over the remaining ticks | Pumpkin writes `remaining_ms()` into `BorderSizeLerpTime` and reads it back as a duration. Self-consistent for Pumpkin saves, but a vanilla/Paper world saved mid-transition has an epoch-scale deadline (about 1.7e12) that Pumpkin treats as a ~55 year remaining lerp, so the border never settles. Store the absolute game-time deadline (and convert on load) to match the NBT contract. | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 2 | `mod.rs:590` (`load_persistent_data`), called only from `new()` at `mod.rs:533` | Level data is loaded when a level is opened (`ServerLevel` constructor / `MinecraftServer.createLevels`) | A world created at runtime via `create_world`/`spawn_world` never restores its own `data/minecraft/chunks.dat` forced chunks, and its scoreboard starts empty even though `save_persistent_data` will write both. Move the per-world forced-chunk restore into `spawn_world` (scoreboard and command storage stay server-wide). | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 3 | `mod.rs:1824` (`tick`, frozen branch) vs `mod.rs:1871-1885` | In vanilla the datapack `#minecraft:tick`/`#minecraft:load` functions and `TimerQueue` are ticked from `tickChildren` around the `runsNormally()` world-tick gate (function manager ticks, then levels tick) | While `/tick freeze` is active Pumpkin skips `tick_worlds` entirely, so `#minecraft:tick`, plugin loader ticks and `/schedule` callbacks pause. Confirm against 1.21 `MinecraftServer.tickChildren` ordering; if functions are outside the freeze gate, split the function/scheduler part out of `tick_worlds`. | S/M |
| 4 | `mod.rs:1019-1027` (`reload_datapacks`), `mod.rs:1871-1876` | `/reload` runs the `#minecraft:load` tag as part of the reload (`FunctionManager` reload listener), so its effects exist when the command returns | Pumpkin marks `load` pending and runs it on the next world tick; a command block or plugin that reads `load` output immediately after `/reload` sees stale state. If the deferred run is intentional (comment says `/reload` does not tick), document it in `docs/LEARNINGS.md` instead of changing. | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 5 | `mod.rs:780` (`get_world_from_dimension`) | Never terminates the process from a lookup | `std::process::exit(1)` on an empty world list hides a caller bug behind a hard exit and skips the shutdown save. Return an error or fall back explicitly. | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 6 | `mod.rs:1072-1106` (`save_all`) | `MinecraftServer.saveEverything` logs per-world failures and keeps saving the rest | Pumpkin returns on the first failed step (level.dat, player data, advancements), so a single failure skips the world saves and reports one error. Collect errors and return the first at the end. | S, FIXED 1eed0f9a7 |
| 7 | `mod.rs:1831-1840` | one flush point per tick | The frozen path flushes block updates in `tick_players_and_network` and again in `flush_pending_block_updates`; harmless (second pass sees an empty set) but redundant work on a per-tick path. | XS |
| 8 | `mod.rs:1946-1985` (`update_tick_times`) | `TickTimeMonitor`-style debug sample gating differs from a per-tick send | Pumpkin sends `CDebugSample` for every subscriber every tick with a 4-value sample; vanilla caps debug-sample collection/sending. Check against 1.21.4 `ServerDebugSubscribers`. Behavioural effect is the client graph only. | S (verify) |

## Checked, no change
- `set_difficulty` / `set_difficulty_locked`: matches `MinecraftServer.setDifficulty` (hardcore forces HARD, locked without force is a no-op, all players get `ClientChangeDifficulty`).
- `on_game_rule_changed`: `doImmediateRespawn` maps to game event `ENABLE_RESPAWN_SCREEN` with 1.0/0.0, sent only to Java players past login, as vanilla.
- `sync_game_time`: tick-count based 20-tick sync in both the normal and frozen paths, before world ticks.
- `save_persistent_data` forced-chunk packing: `x` low 32 bits, `z` high 32 bits of `ChunkPos.toLong`.
- `tick_players_and_network`: players/network tick while frozen, matching `PlayerList.tick` being outside the world-tick gate; suspend/resume flushing brackets the whole tick like `ServerCommonPacketListenerImpl`.
- `next_map_id`, `new_container_id` ordering, listing add/remove accounting, telemetry join/leave pairing.

## Status (code-verified sweep)
FIXED: close_admission holds player_join_lock (server/mod.rs:1375-1381). STILL OPEN: spawn_world WorldInitEvent and get_tps rows not re-read.

## Result

Unit files: `crates/pumpkin-core/src/world/border.rs`, `crates/pumpkin-world/src/world_info/anvil.rs`,
`crates/pumpkin-core/src/server/ticker.rs` (`server/mod.rs` itself is out of scope for this unit).

| # | Outcome |
|---|---|
| 1 | FIXED (NBT contract). Re-checked vanilla `WorldBorder.Settings`: `lerp_time` is `extent.getLerpTime()` = `MovingBorderExtent#lerpProgress`, a **remaining tick count that decrements every tick**, not an absolute game-time deadline, so `applySettings` treats it as a duration. Pumpkin stored milliseconds in `BorderSizeLerpTime`; `anvil.rs` now converts ticks to the internal milliseconds on read (`saturating_mul(50)`) and back with `div_ceil(50)` on write, and `border.rs` sends the protocol/command lerp value in ticks (`remaining_ticks()`, `speed / 50`), matching `ClientboundInitializeBorderPacket` / `ClientboundSetBorderLerpSizePacket` and `/worldborder ... time`. Bounds helpers (`contains`, `within_bounds`, `distance_to_border`, `clamp_*`, collision shape) now use `current_diameter()` rather than the lerp target, as vanilla clamps against `extent` min/max. Tests: `border_lerp_time_is_stored_in_game_ticks`, `bounds_follow_the_running_transition_not_the_target`, `remaining_ticks` assertion in the apply test. |
| 2 | OPEN: restore of per-world forced chunks belongs in `crates/pumpkin-core/src/server/mod.rs` `spawn_world`, outside this unit. |
| 3 | OPEN: freeze gate split lives in `server/mod.rs` `tick` / `tick_worlds`, outside this unit. Verified against Paper 1.21 `MinecraftServer.tickChildren`: `getFunctions().tick()` is called before the `runsNormally()` gate, and that method itself gates `#minecraft:tick` / `#minecraft:load` on `runsNormally()`, so Pumpkin's placement inside `tick_worlds` matches function execution; only plugin loader ticks and `/schedule` differ and need the `mod.rs` split. |
| 4 | OPEN: `/reload` load-tag timing is `server/mod.rs` + datapack manager, outside this unit. |
| 5 | OPEN: `get_world_from_dimension` `process::exit` is `server/mod.rs`, outside this unit. |
| 7 | OPEN: double flush is `server/mod.rs`, outside this unit (behaviourally harmless). |
| 8 | OPEN: debug sample gating is `server/mod.rs` `update_tick_times`, outside this unit. |

`ticker.rs` re-read: overload skip, warning rate limit, sprint deadline reset and the park loop match
Paper `runServer` / `waitUntilNextTick`; no change needed.
