# blocks / Command Block (`crates/pumpkin-core/src/block/blocks/command.rs`)

Status: AUDITED (2026-11-17). Coverage row: `audit` / `crates/pumpkin-core/src/block/blocks/command.rs`.
Vanilla classes: `CommandBlock`, `CommandBlockEntity` (+ `BaseCommandBlock`), `ComparatorBlock`,
`ServerPlayer#openCommandBlock`, `ServerGamePacketListenerImpl#handleSetCommandBlock`.
Companion audit: `docs/tracker/tentities-command-block-minecart.md` (same `BaseCommandBlock` logic).

Tracker note "2 TODO/FIXME/vanilla markers" is stale: the file has no TODO/FIXME. The two `Vanilla ...`
comments (`chain_execute` facing follow, `on_scheduled_tick` condition capture) are explanatory, not gaps.

## Fixed in this pass
- `command.rs` `CommandBlock::execute` had no Searge easter egg. Vanilla `BaseCommandBlock#performCommand`
  answers the exact command `Searge` with `#itzlipofutzli` and `successCount = 1` without dispatching.
  Mirrored (also inside the `command_blocks_work` guard, matching the logic already fixed for the minecart).
  Commit `0f1566884`.
- `command.rs` `normal_use` gated on `permission_lvl >= Two` only and returned `SuccessServer`.
  Vanilla `CommandBlock#onUse` gates on `player.canUseGameMasterBlocks()` (creative **and** op level 2) and
  returns `ActionResult.CONSUME`. Now uses the existing `Player::can_use_game_master_blocks()` and `Consume`.
  Commit `9621174fb`.
- `command.rs` `get_comparator_output` did `success_count as u8`, which wraps (`256 -> 0`) and can exceed 15.
  Vanilla `ComparatorBlock#getInputSignal` clamps the analog output at 15. Now `min(15)`. Commit `d6cfd2d12`.

## Remaining gaps
1. `command.rs` `CommandBlock::on_place` uses `player.get_entity().get_facing().opposite()` (horizontal only).
   Vanilla `CommandBlock#getStateForPlacement` uses `context.getNearestLookingDirection().getOpposite()`
   (all six facings), so placing while looking up/down must orient the block vertically. No shared Rust helper
   exists yet (`vine.rs::get_nearest_looking_directions` is vine-specific). Vanilla class: `CommandBlock`.
   Fix: add a shared nearest-looking-direction helper and use it here. Size: S (verify against decompile).
2. `command.rs` `can_place_at` allows placement for any creative player. Vanilla treats command blocks as
   `GameMasterBlock`, i.e. placement requires `canUseGameMasterBlocks()` (creative + op 2); ops in survival
   must be refused. Fix: `args.player.can_use_game_master_blocks()`. Size: S (confirm the vanilla check site).
3. `command.rs` `chain_execute` caps the loop at `u16::MAX` (65535) iterations and logs a `warn!`; vanilla caps
   at 65536 and only debug-logs. Fix: `65536` constant and `debug!`. Size: S, cosmetic.
4. `command/mod.rs` `CommandSender::send_system_message` (CommandBlock arm) always overwrites `last_output`.
   Vanilla `BaseCommandBlock#performCommand` only records output when `trackOutput` is true, so a block with
   `TrackOutput=false` keeps its stored `LastOutput` untouched. Fix: check `track_output` before writing.
   Vanilla class: `BaseCommandBlock`. Size: S.
5. Block entity gaps that this file depends on (`crates/pumpkin-core/src/block/entities/command_block.rs:10`
   has `todo: CustomName, LastExecution, UpdateLastExecution`): no `CustomName` (GUI title / `/data` / NBT),
   and no `LastExecution` game-time guard, so a block can execute twice in one tick when the redstone edge and
   the scheduled tick coincide (vanilla `BaseCommandBlock#performCommand` early-returns on `gameTime == lastExecution`
   unless `UpdateLastExecution`). Size: M.
6. Bedrock: the editor round-trip relies on `world.update_block_entity` replaying the vanilla
   `ClientboundBlockEntityDataPacket`-opens-the-screen behaviour for Java clients. Bedrock command-block UI needs
   the actor data path (`bedrock_block_actor_data`, which exists) plus verification that opening works; unverified.
   Size: M (runtime check, `scripts/boot.sh`).

## Verified matching (no change)
- `update()` mirrors `setPoweredAndUpdate`: edge-only, REDSTONE-mode rising edge marks the condition and schedules
  one tick later; chain/auto blocks never self-schedule from the edge; chain blocks run with `auto` set on place.
- `chain_execute` follows each chain block's own `FACING` (chains may turn), requires `powered || auto`, resets
  `success_count` when a conditional block's condition fails, and honours the `command_blocks_work` gamerule.
- `on_scheduled_tick`: repeating blocks compute the *next* condition while acting on the captured one, reschedule
  only while `powered || auto`, and pulse the output signal afterwards; impulse blocks only execute a captured
  condition; chain blocks never execute from their own tick.
- `get_comparator_output` returns `None` without a block entity (comparator then reads 0, as vanilla does).
- `place`d: `track_output` from the `sendCommandFeedback` gamerule, `auto` forced for chain blocks, creation of the
  block entity when absent, and neighbour update gated on the gamerule.
- No automated test added: both changed paths need a live `World`/`Server` (comparator args, command dispatch);
  the minecart twin has the same constraint.
