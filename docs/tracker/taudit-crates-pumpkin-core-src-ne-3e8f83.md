# Audit: crates/pumpkin-core/src/net/java/play/player_action.rs

Vanilla reference: `net.minecraft.server.level.ServerPlayerGameMode#handleBlockBreakAction`
(+ CraftBukkit/Spigot patch that fires `PlayerInteractEvent` / `BlockDamageEvent` /
`BlockDamageAbortEvent`). No decompiled `.vendor` copy was available in this worktree, so the
comparison uses the 1.21 `ServerPlayerGameMode` structure.

## Fixed in this pass

- `player_action.rs:35` — vanilla `Level.mayInteract` (spawn protection) rejection at
  `START_DESTROY_BLOCK` was missing before the interaction events, so a protected player got the
  crack animation, `BlockBehaviour::attack` hooks and plugin events before `break_block` refused
  the break. Now rejected up front with the vanilla `build.spawn_protection` message and a block
  resync. Replaces `// TODO: do validation`.
- `player_action.rs:270` — `DROP_ITEM`, `DROP_ALL_ITEMS` and `SWAP_ITEM_WITH_OFFHAND` were not
  gated on `!player.isSpectator()` (vanilla guards all three in `handleBlockBreakAction`).
- `player_action.rs:270` — the packet sequence was only acknowledged for block-destroy statuses;
  drops, use-release, swap, spear jab and destroy-direction changes left
  `CAcknowledgeBlockChange` unset for that tick (vanilla acks the sequence for every action).
- `// TODO: Config` — no vanilla or Pumpkin config knob is involved in this code path; the marker
  described a configuration hook that does not exist in vanilla. Marker removed.

## Remaining gaps

| file:line | vanilla class / method | gap | suggested fix | size |
|---|---|---|---|---|
| `player_action.rs:31` | `ServerPlayerGameMode#handleBlockBreakAction` | no `pos.getY() >= maxHeight` guard: vanilla rejects the action with a block resync before any handling, Pumpkin looks the block up at an out-of-range Y | compare `position.0.y` with the world's height limits (build height / dimension) right after the reach check and return after a `CBlockUpdate` | S, FIXED (0ad771977): `position.0.y > world.get_top_y()` resync before spawn protection |
| `player_action.rs:36` | same, plus CraftBukkit ordering | `PlayerInteractEvent` and `BlockDamageEvent` fire before the `cannot_mine_block` check (spectator, adventure `can_break`, creative non-destroying tool), while vanilla runs `blockActionRestricted` / spectator checks first | move the `cannot_mine_block` check above the event block; keep the `CBlockUpdate` resync on rejection | S, FIXED dc1e406ed |
| `player_action.rs:143` | same, "abort destroying since another started" | a `START_DESTROY_BLOCK` on a second block just overwrites `mining_pos`; the previous target keeps its announced break stage, so other players can keep seeing its crack progress | if `player.mining` and the stored `mining_pos` differs from the new position, `world.set_block_breaking(entity, old, Stop)` and resync the old position | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `player_action.rs:190` | `ServerPlayerGameMode` `ABORT_DESTROY_BLOCK` | abort sends `BlockBreakingProgress::Stop` but leaves `current_block_destroy_stage` / `current_block_breaking_speed` and `mining_pos` stale and never resyncs the block; vanilla clears the destroy state and sends a `ClientboundBlockUpdatePacket` | call `player.reset_mining()` (or `stop_mining()`) and add a `CBlockUpdate` for `player_action.position` | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
## Result

Verified against decompiled vanilla `net.minecraft.server.level.ServerPlayerGameMode`
(`.vendor/vanilla-decompiled/net/minecraft/server/level/ServerPlayerGameMode.java`):
`handleBlockBreakAction` lines 192-330 and `abortDestroyBlock` lines 332-350.

| file:line | result | note |
|---|---|---|
| `player_action.rs:143` | FIXED | On a non-instant `START_DESTROY_BLOCK` while `player.mining` is already true and the stored `mining_pos` differs, the old target now gets `set_block_breaking(old, Stop)` (clears the announced crack for other players) plus a `CBlockUpdate` resync via `sync_block_state_to_client` (clears the digging client's crack). Mirrors vanilla sending `ClientboundBlockUpdatePacket(this.destroyPos, ...)` before overwriting `destroyPos`. |
| `player_action.rs:190` | FIXED | `CancelledDigging` now calls `reset_mining()` (clears `mining`, `has_delayed_destroy`, `current_block_destroy_stage` and `current_block_breaking_speed`), resets `mining_pos` to `BlockPos::ZERO`, sends `set_block_breaking(Stop)` for the stored target when it differs from the reported position and is not ZERO (vanilla mismatch branch), keeps the `Stop` for the reported position, and adds a `CBlockUpdate` resync for the reported position (the miner is excluded from `set_block_breaking`, so it needs its own resync). |
| `player_action.rs:31` (height guard) | unchanged | Already marked FIXED in an earlier pass; present at lines 38-45. |
| `player_action.rs:36` (event ordering) | unchanged | Already marked FIXED in an earlier pass; present at lines 63-73. |

No test added: the two fixes live inside the async packet handler and depend on `Arc<Player>`,
`World`, plugin events and network sockets. There is no pure seam in this file without extracting
a trivial predicate helper, which would be a formula-mirroring test discouraged by the MVP test
rules. No generated data, WIT or cross-crate changes were needed.

Files touched: `crates/pumpkin-core/src/net/java/play/player_action.rs`, this doc (`rustfmt --edition 2024` run on the .rs file, exit 0).
