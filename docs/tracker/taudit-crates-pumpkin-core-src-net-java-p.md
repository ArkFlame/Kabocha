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

## Gap status
- Rows: STILL OPEN. Note body partly unreadable in this pass; nothing verified.

## Result

Pass 2026-11-17 (no decompiled sources in `.vendor/`; vanilla is `ServerGamePacketListenerImpl.handleUseItemOn`
+ `ServerPlayerGameMode.useItemOn`/`ServerLevel.mayInteract` from memory, per the header notes).

- FIXED **max build height gate**: `handle_use_item_on` now returns early (block resync to the
  client, no action) when `position.y > world.get_top_y()`, i.e. `y >= maxBuildHeight`. No new
  `World` accessor was needed: `get_top_y()` already equals `min_y + height - 1`.
- FIXED **`ServerLevel.mayInteract`**: new local `may_interact(player, world, position)` helper
  (`use_item_on.rs`) checks the world border via `Worldborder::is_within_bounds(x, z, 0.0)` and
  rejects spawn protection via `World::is_in_spawn_protection`. Called from the same early return
  as the build-height gate (vanilla order: build height, then mayInteract). No spawn-protection
  chat message is sent, matching vanilla `mayInteract` (only `break_block`/placement do). The
  duplicated `world` load further down was removed and reused.
- STALE **`increment_stat(Used, ...)` unconditional**: the call is already gated on
  `item_result.consumes_action() || should_try_decrement`; nothing to change.
- OPEN **`isItemEnabled(level.enabledFeatures())`**: needs a new `Item::is_enabled`/feature helper
  in `pumpkin-data` (generated/data layer), outside this unit.
- OPEN **`awaitingPositionFromClient` guard**: `JavaClient` keeps no teleport-ack state; needs
  cross-file net state.
- OPEN **`PlayerInteractEvent` for spectators**: deliberate superset of Paper (lets plugins block
  spectator inventory opening); left as is, no behaviour change.
- OPEN **item `on_use` fallback** and **hit-position epsilon check**: both need a runtime client
  capture to decide; no change.
- `hub.rs`: no open rows (only the management `spawn_protection_radius` atomic, unused by this
  path; `World::is_in_spawn_protection` reads `BasicConfiguration` directly).
- `worldborder.rs`: no open rows; `is_within_bounds`/`contains` already match
  `WorldBorder.isWithinBounds` and are covered by existing tests.
- No new unit test: `may_interact` needs a live `World`/`Player`, so there is no pure seam; the
  border half it relies on is already covered by `world/border.rs` tests.

Gap status: 2 fixed, 2 stale, 4 open (reasons above).


## Result (2026-11-18, decompiled vanilla now present)

Vanilla reference: `ServerGamePacketListenerImpl.handleUseItemOn`
(`server/network/ServerGamePacketListenerImpl.java:2298`), `ServerPlayerGameMode.useItemOn`
(`server/level/ServerPlayerGameMode.java:532`), `ServerLevel.mayInteract`
(`server/level/ServerLevel.java:3058`), `WorldBorder.isWithinBounds`
(`world/level/border/WorldBorder.java:88`).

- VERIFIED FIXED **max build height gate**: vanilla uses `getMaxY()`/`getMinY()`
  (`handleUseItemOn:2322-2327`); existing `position.y > world.get_top_y()` gate matches for the
  max side. No new accessor was needed. (Vanilla also sends a build-limit chat message and resyncs
  the neighbour block; Pumpkin only resyncs the clicked block. Cosmetic, no gameplay effect, and
  not a listed row.)
- VERIFIED FIXED **`ServerLevel.mayInteract`**: vanilla is
  `!(player && (isUnderSpawnProtection || !border.isWithinBounds(pos)))` (`ServerLevel.java:3059`).
  Existing `may_interact` matches via `Worldborder::is_within_bounds(x, z, 0.0)` and
  `World::is_in_spawn_protection`. The latter already handles overworld-only, permission-level
  bypass, radius 0 and no-operators, so the earlier "needs OP handling, size M" caveat is stale.
  Border bounds use the interpolated current diameter, matching `WorldBorder.getMinX/getMaxX`.
- VERIFIED FIXED **`awaitingPositionFromClient` guard**: vanilla guards the interaction with
  `this.awaitingPositionFromClient == null` (`handleUseItemOn:2334`). The existing
  `player.awaiting_teleport` lock check is the equivalent. Deliberately uses the raw lock instead
  of `Player::is_awaiting_teleport()`, which resends the teleport after 20 ticks (a side effect
  this read-only guard must not trigger).
- VERIFIED CORRECT, NO CHANGE **hit-position epsilon check**: earlier passes were wrong to call
  this redundant and to ask for a runtime capture. Vanilla explicitly drops hits where
  `abs(location - centerOf(block)) >= 1.0000001` on any axis (`handleUseItemOn:2317-2319`). The
  inline check already matched; extracted it to the pure `is_within_block_center_limit` helper and
  added the boundary test `block_center_hit_limit_matches_vanilla` in `use_item_on.rs` to lock the
  vanilla constant in place. This was the only code change this pass.
- VERIFIED CORRECT, NO CHANGE **`PlayerInteractEvent` for spectators**: earlier passes claimed
  Paper fires the event only in the non-spectator branch. `ServerPlayerGameMode.java:551` calls
  `CraftEventFactory.callPlayerInteractEvent` unconditionally, *before* the spectator branch at
  line 574. Pumpkin firing the event for spectators therefore matches Paper, not a superset.
- VERIFIED CORRECT, NO CHANGE **item `on_use` fallback removal**: `useItemOn:619/626/639-642`
  calls `state.useItemOn`, `state.useWithoutItem` and `itemStack.useOn`, but never `Item#use` (the
  right-click-in-air path, handled by `handle_use_item`). Pumpkin's `use_with_item`/`on_use`
  plus `item_registry.use_on_block`/`place_block` cover those vanilla calls; the removed
  `ItemRegistry::on_use` fallback would have double-fired `Item#use`. Removal stands.
- STALE **`increment_stat(Used, ...)` unconditional**: still gated on
  `item_result.consumes_action() || should_try_decrement`; nothing to change.
- OPEN **`isItemEnabled(level.enabledFeatures())`**: confirmed real at `handleUseItemOn:2307` and
  `useItemOn:539`. `Server::get_enabled_features`/`DatapackManager::is_feature_enabled` exist, but
  `Item` (`pumpkin-data/src/generated/item.rs`) has no required-features field, so per-item/per-block
  feature data must be added in the generated data layer. Outside this unit, left open.
- `border.rs`, `hub.rs`, `worldborder.rs`: no open rows, no changes. `Worldborder::is_within_bounds`
  matches vanilla `WorldBorder.isWithinBounds` and is covered by existing tests.

Gap status after this pass: 3 resolved by verification (teleport guard, spectator event, item
fallback), 1 verified-and-tested (hit epsilon), 1 stale, 1 open (isItemEnabled, outside unit).


## Result (2026-11-18, post-restart pass)

Only one row remains open from the earlier passes: `isItemEnabled(level.enabledFeatures())`.
No other row was re-verified; the prior Result sections already resolved them against the
decompiled sources and those fixes are present in the working tree.

- VERIFIED OPEN **`isItemEnabled(level.enabledFeatures())`**: vanilla rejects the interaction
  before any action when the held stack's item requires features absent from the level
  (`ServerGamePacketListenerImpl.handleUseItemOn:2307`, `handleUseItem:2397`), and
  `Item.isEnabled` is driven by a per-item `Item.requiredFeatures: FeatureFlagSet` populated
  through `Item.Properties.requiredFeatures(...)` (`Item.java:133/155/384/657`).
  LEFT OPEN: Pumpkin's generated `Item` (`pumpkin-data/src/generated/item.rs:22`) has only
  `id`/`registry_key`/`crafting_remainder`/`components`, with no required-features field, so a
  correct check needs per-item/per-block feature data added in the generated data layer. That is
  generated data, outside this unit's files (`border.rs`, `hub.rs`, `use_item_on.rs`,
  `worldborder.rs`); Pumpkin's known-flag list (`datapack/mod.rs:2293`) only names flags and does
  not map items to them, so no safe conservative check exists here.
- NO CODE CHANGES: `use_item_on.rs`, `world/border.rs`, `net/management/hub.rs` and
  `command/commands/worldborder.rs` needed no edits this pass.
- NO NEW TEST: the open row has no pure seam without generated feature data. The helpers that
  remain in `use_item_on.rs` (`is_within_block_center_limit`, `is_valid_hit_position`) are already
  covered by their unit tests, and `Worldborder::is_within_bounds`/`contains` are covered by the
  `world/border.rs` tests.

Gap status after this pass: 0 fixed, 1 open (isItemEnabled, needs generated per-item feature
data outside this unit).

## Result (r2_014 pass)
- `mayInteract` (world border / spawn protection): VERIFIED - `crates/pumpkin-core/src/net/java/play/use_item_on.rs:305` `may_interact`, vanilla `ServerGamePacketListenerImpl.java:2322-2334`.
- Max build height: FIXED - added `position.0.y < world.get_bottom_y()` to the reject branch in `use_item_on.rs` (top bound already present). Vanilla `ServerGamePacketListenerImpl.java:2322-2326` checks both `maxY` and `minY`. Build-limit message not sent; Pumpkin resyncs the block instead.
- `isItemEnabled` feature check: STILL OPEN - no `Item::is_enabled` helper; not reached.
- `awaitingPositionFromClient` guard: VERIFIED - `use_item_on.rs` `teleport_pending` via `player.awaiting_teleport`; vanilla `ServerGamePacketListenerImpl.java:2334`.
- `PlayerInteractEvent` for spectators: STILL OPEN - policy decision (plugin compat), not reached.
- Item fallback to `ItemRegistry::on_use`: STILL OPEN - not reached in this pass.
- `increment_stat(Used)` unconditional: STILL OPEN - not reached in this pass.
## Result (r3_038 pass)
- VERIFIED **`PlayerInteractEvent` for spectators**: vanilla `ServerPlayerGameMode.useItemOn` calls `CraftEventFactory.callPlayerInteractEvent` unconditionally at `server/level/ServerPlayerGameMode.java:551`, before the `GameType.SPECTATOR` branch. Rust fires it for spectators at `crates/pumpkin-core/src/net/java/play/use_item_on.rs:71-75`. Matches.
- VERIFIED **item `on_use` fallback**: vanilla `useItemOn` (`ServerPlayerGameMode.java:619-652`) calls only `state.useItemOn`, `state.useWithoutItem` and `itemStack.useOn`, never `Item#use`. Rust `use_on_block` path at `use_item_on.rs:157-200` has no `on_use` fallback; the only `on_use` at `use_item_on.rs:253` is the block registry. Matches.
- OPEN **`increment_stat(Used)` gate**: vanilla awards `Stats.ITEM_USED` only on a `Success` result (`world/item/ItemStack.java:439`, `:508`, `:747`). Rust gates on `item_result.consumes_action() || should_try_decrement` (`use_item_on.rs:193-194`), and `consumes_action()` (`block/registry.rs:512`) also counts `Consume`. Left open: the item-level result type was not traced to confirm whether `Consume` can reach here. No change made.
- BLOCKED-DATA **`isItemEnabled(level.enabledFeatures())`** (`ServerGamePacketListenerImpl.java:2307`, `:2397`; `ServerPlayerGameMode.java:537` for the block): needs per-item and per-block `requiredFeatures`. `crates/pumpkin-data/src/generated/item.rs` and `block.rs` have no feature field or `is_enabled` helper. Missing generated `pumpkin-data` field, so no Rust change.
- Files edited: none in this pass.
Gap status: 2 verified, 1 open (stat gate, traced to item result type), 1 blocked-data (isItemEnabled).
