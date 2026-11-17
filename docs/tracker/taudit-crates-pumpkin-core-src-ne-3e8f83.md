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
| `player_action.rs:31` | `ServerPlayerGameMode#handleBlockBreakAction` | no `pos.getY() >= maxHeight` guard: vanilla rejects the action with a block resync before any handling, Pumpkin looks the block up at an out-of-range Y | compare `position.0.y` with the world's height limits (build height / dimension) right after the reach check and return after a `CBlockUpdate` | S |
| `player_action.rs:36` | same, plus CraftBukkit ordering | `PlayerInteractEvent` and `BlockDamageEvent` fire before the `cannot_mine_block` check (spectator, adventure `can_break`, creative non-destroying tool), while vanilla runs `blockActionRestricted` / spectator checks first | move the `cannot_mine_block` check above the event block; keep the `CBlockUpdate` resync on rejection | S |
| `player_action.rs:143` | same, "abort destroying since another started" | a `START_DESTROY_BLOCK` on a second block just overwrites `mining_pos`; the previous target keeps its announced break stage, so other players can keep seeing its crack progress | if `player.mining` and the stored `mining_pos` differs from the new position, `world.set_block_breaking(entity, old, Stop)` and resync the old position | S |
| `player_action.rs:190` | `ServerPlayerGameMode` `ABORT_DESTROY_BLOCK` | abort sends `BlockBreakingProgress::Stop` but leaves `current_block_destroy_stage` / `current_block_breaking_speed` and `mining_pos` stale and never resyncs the block; vanilla clears the destroy state and sends a `ClientboundBlockUpdatePacket` | call `player.reset_mining()` (or `stop_mining()`) and add a `CBlockUpdate` for `player_action.position` | S |
