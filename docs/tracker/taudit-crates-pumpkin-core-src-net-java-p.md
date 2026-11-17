# net/java/play/use_item_on.rs (AUDIT)

Audited 2026-11-16. No decompiled sources present (`.vendor/` empty), so vanilla
behaviour is from `ServerGamePacketListenerImpl.handleUseItemOn` and
`ServerPlayerGameMode.useItemOn` (1.21.x) plus the Paper patch that fires
`PlayerInteractEvent` and resyncs the block on cancel.

File: `crates/pumpkin-core/src/net/java/play/use_item_on.rs` (369 lines, 3 TODO
markers before this pass).

## Fixed in this pass

- **TODO: maybe log?** (`use_item_on.rs:26`). Out-of-reach clicks are routine
  (client/`server` latency, 1.0 block interaction range), and vanilla
  `ServerPlayerGameMode.useItemOn` returns `PASS` without logging. The error was
  raised with `Level::WARN` (`net/java/play/mod.rs:105`), so a single lagging
  player could spam the log. `BlockOutOfReach` now logs at `TRACE` like
  `BlockOutOfWorld`/`InvalidGamemode`; no kick, no gameplay change.
- **TODO item cool down** (`use_item_on.rs:154`). Stale: the item path is already
  gated by `player.is_on_cooldown()` in `ItemRegistry::use_on_block`
  (`item/registry.rs:148`), which is exactly where vanilla guards the item's
  `useOn`. Vanilla does not check a cooldown before the block's `useItemOn`
  either.
- **TODO: Config** (`use_item_on.rs:173`). Stale: vanilla has no config option,
  the decrement guard is `!player.getAbilities().instabuild`, which is the
  `GameMode::Creative` check already present at `use_item_on.rs:175`.

## Open gaps

- **No `ServerLevel.mayInteract(player, blockPos)` check.** Vanilla
  `handleUseItemOn` refuses the interaction when the block is outside the world
  border or inside spawn protection. Pumpkin has a world border
  (`command/commands/worldborder.rs`) and `spawn_protection_radius`
  (`net/management/hub.rs:25`) but no combined helper. Suggested fix: add
  `World::may_interact(&Player, &BlockPos) -> bool` (border contains point &&
  not spawn-protected && in build height) and early-return `Ok(())` at
  `use_item_on.rs:25`. Size **M** (needs permission/OP handling for spawn
  protection).
- **No max build height gate.** Vanilla `handleUseItemOn` only proceeds when
  `blockPos.getY() < level.getMaxBuildHeight()`, otherwise it resyncs the block
  to the client. Pumpkin's `World` exposes no `max_build_height` accessor
  (`pumpkin-world` only has per-generator `height()`). Suggested fix: add the
  accessor on `World`, then gate `use_item_on.rs:25`. Size **S/M**.
- **No `itemStack.isItemEnabled(level.enabledFeatures())` check.** Vanilla
  rejects items disabled by datapack features before any interaction.
  `Server::get_enabled_features()` exists (`server/mod.rs:756`), but no
  `Item::is_enabled` equivalent exists. Suggested fix: add the helper and return
  `Ok(())` right after `use_item_on.rs:92`. Size **M**.
- **No `awaitingPositionFromClient` guard.** Vanilla ignores block interactions
  while a teleport is unacknowledged (`handleUseItemOn:
  `this.awaitingPositionFromClient == null`). `JavaClient` keeps no teleport-ack
  state, so this cannot be added locally. Size **M**.
- **`PlayerInteractEvent` fires for spectators** (`use_item_on.rs:57-75`). Paper
  fires the event only in the non-spectator branch of
  `ServerPlayerGameMode.useItemOn`; spectator menu opening is not event-guarded.
  Keeping the current event lets plugins block spectator inventory opening, which
  is a superset of Paper behaviour. Decide with plugin compatibility in mind.
  Size **S** if changed.
- **Item fallback to `ItemRegistry::on_use` has no server-side vanilla
  equivalent** (`use_item_on.rs:190-198`). Vanilla `Item.useOn` returns
  `PASS` and the *client* then sends `ServerboundUseItemPacket`; the server does
  not re-run the item use inside `useItemOn`. This block came from the
  `fix(item): throw usable items when right-clicking a block` port, so it may
  compensate for a client that does not send the fallback. If a vanilla client
  does send `SUseItem` after a `PASS`, this double-fires the item (normally
  masked by the cooldown). Needs a runtime capture to decide. Size **S** to
  verify, no change made.
- **`increment_stat(Used, ...)` unconditional** (`use_item_on.rs:95`). Vanilla
  awards `Stats.ITEM_USED` only for interactions that consume the action. The
  exact call site in 1.21 is in `ServerPlayerGameMode.useItem`/`useItemOn`; could
  not be confirmed without decompiled sources, so statistics may over-count
  clicking a block with an item that does nothing. Size **S**, needs a decompiled
  reference to confirm.

## Redundant but harmless

- `use_item_on.rs:30-36` rejects hit coordinates outside `0.5 +- 1.0000001`
  claiming vanilla validates against the block centre. Vanilla validates nothing
  here; the real guard is `is_valid_hit_position` (`use_item_on.rs:349`), a
  strict subset. Every coordinate rejected by the first check is also rejected by
  the second before any action, so both paths end in `Ok(())`; removing the first
  check changes no behaviour, so it was left alone.
