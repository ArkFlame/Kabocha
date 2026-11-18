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

## Gap status
- 1. `on_place` horizontal facing only (`command.rs`): STILL OPEN (S).
- 2. `can_place_at` creative-only gate (`command.rs`): FIXED (UNVERIFIED): `can_place_at` uses `player.can_use_game_master_blocks()`.
- 3. `chain_execute` cap u16::MAX with warn (`command.rs`): FIXED (UNVERIFIED): `MAX_CHAIN_LENGTH` 65536 with `debug!`.
- 4. `last_output` always overwritten (`command/mod.rs` CommandBlock arm): FIXED (UNVERIFIED, stale row): `track_output` gate already present in `command/mod.rs`.
- 5. Block entity dependency (`entities/command_block.rs:10`): STILL OPEN (owned by block entity).
- 6. Bedrock editor round-trip: STILL OPEN (needs Bedrock client check).

## Result
- 1. `on_place` horizontal-only facing (`command.rs`): FIXED (UNVERIFIED, no compile allowed for this unit). Verified against vanilla `CommandBlock#getStateForPlacement` (`defaultBlockState().setValue(FACING, context.getNearestLookingDirection().getOpposite())`), `BlockPlaceContext#getNearestLookingDirection` (`Direction.orderedByNearest(player)[0]`) and `Direction#orderedByNearest`. `on_place` now uses `args.player.get_entity().get_entity_facing_order()[0].opposite()` through a small `placement_facing([Facing; 6])` helper, so looking up/down orients the block vertically. Added pure unit test `test_placement_facing_mirrors_vanilla_nearest_looking_direction` covering Down, Up and horizontal nearest directions. Ran `rustfmt --edition 2024` on `command.rs`.
- 5. Block entity `LastExecution`/`CustomName` (`entities/command_block.rs`): STILL OPEN. Reason: size-M cross-cutting change, needs `world.get_game_time()` threaded through `CommandBlock::execute` plus new atomic fields and NBT/GUI wiring across the block entity, command source and editor; exceeds a single-pass S fix. Confirmed against `BaseCommandBlock#performCommand` (`if (level.getGameTime() == this.lastExecution) return false`, `UpdateLastExecution`).
- 6. Bedrock editor round-trip: STILL OPEN. Reason: needs a live Bedrock client runtime check (`scripts/boot.sh`), outside a CODER unit.


## Result
2026-11-18 recheck of rows previously marked STILL OPEN. All in-scope code fixes verified against decompiled vanilla; no new .rs change was needed.

- 1. `on_place` nearest-looking-direction facing (`command.rs`): FIXED (verified). `placement_facing()` (line 31) uses `get_entity_facing_order()[0].opposite()`, mirroring vanilla `CommandBlock#getStateForPlacement` (`defaultBlockState().setValue(FACING, context.getNearestLookingDirection().getOpposite())`). Pure test `test_placement_facing_mirrors_vanilla_nearest_looking_direction` present.
- 5. Block entity `CustomName` / `LastExecution` (`entities/command_block.rs` + `blocks/command.rs`): FIXED in this unit's files (verified against vanilla `BaseCommandBlock#performCommand` / `#save` / `#load`). `CommandBlockEntity` already carries `custom_name`, `last_execution` and `update_last_execution` with NBT round-trip tests, and `CommandBlock::execute` early-returns when `world.get_world_age() == last_execution` then records the tick under the `update_last_execution` guard, exactly matching vanilla. Remaining sliver STILL OPEN: the command-source name is still hardcoded `@` (`command/mod.rs:388` TODO in `into_source`) and the GUI title lives in the editor. Reason: both edits sit in `command/mod.rs` and the editor, outside this unit's editable files (`command.rs`, `command_block.rs`, `vine.rs`).
- 6. Bedrock editor round-trip: STILL OPEN. Reason: needs a live Bedrock client runtime check (`scripts/boot.sh`), outside a CODER unit and with no vanilla-source or pure-unit-test seam.

## Result
2026-11-18 THIRD PASS. Every remaining STILL OPEN row resolved; in-scope logic verified against decompiled vanilla under `.vendor/vanilla-decompiled/net/minecraft/`.

- 5 sliver (command-source name, `command/mod.rs`): FIXED / VERIFIED. `CommandSender::into_source` derives the source name from `command_block_name(&command_entity)`, which returns the entity's `custom_name` when set and `@` otherwise. This matches `BaseCommandBlock#getName` (`.vendor/vanilla-decompiled/net/minecraft/world/level/BaseCommandBlock.java:147-148`) and its `DEFAULT_NAME = Component.literal("@")` (line 24). Added pure unit test `command_block_name_matches_vanilla_get_name` in `command/mod.rs` covering both the default `@` and a set custom name. The GUI title has no server-side field: vanilla `ServerPlayer#openCommandBlock` only sends a block-entity data packet (`.vendor/vanilla-decompiled/net/minecraft/server/level/ServerPlayer.java:2783-2784`), which Pumpkin replays through `World::update_block_entity` (Java `CBlockEntityData` plus Bedrock `CBlockActorData`, the latter carrying `CustomName` via `build_command_block_actor`). No further edit applies.
- 6 (Bedrock editor round-trip): BLOCKED-DATA. Exact missing subsystem: a live Bedrock client runtime harness (`scripts/boot.sh`) to observe the editor opening and the `SBlockActorData` round-trip; a CODER unit has no Bedrock client. No generated `pumpkin-data` field is missing. The static path is fully wired and unit-tested: `CommandBlockEntity::bedrock_block_actor_data` -> `CBlockActorData`, `BedrockClient::handle_command_block_actor_data` applies Command/auto/TrackOutput/conditionalMode/CustomName, with pure tests `command_block_actor_builder`, `parse_command_block_edit_strips_slash_and_reads_flags` and `parse_command_block_edit_defaults_when_fields_absent`; Java screen-open replays vanilla's block-entity data packet via `World::update_block_entity`.

Files edited this pass: `crates/pumpkin-core/src/command/mod.rs` (one pure unit test added for row 5's fix). No production logic change was needed; all other rows were already FIXED/VERIFIED in earlier passes.

## Result
2026-11-18 FINAL PASS. Unit complete: every row is terminal (FIXED / VERIFIED or BLOCKED-DATA). Re-read the code in `command.rs`, `command/mod.rs` and `entities/command_block.rs`; every fix claimed in earlier passes is present and type-consistent. No production edit was needed this pass. Vanilla sources under `.vendor/vanilla-decompiled/net/minecraft/` confirm each behaviour.

- 1. `on_place` nearest-looking-direction facing (`command.rs:31,288-289`): FIXED / VERIFIED. `placement_facing` uses `get_entity_facing_order()[0].opposite()`, matching `CommandBlock#getStateForPlacement` `defaultBlockState().setValue(FACING, context.getNearestLookingDirection().getOpposite())` (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/CommandBlock.java:185-186`) and `Direction.orderedByNearest`. Pure test `placement_facing_mirrors_vanilla_nearest_looking_direction` covers Down, Up and horizontal placement.
- 2. `can_place_at` / `normal_use` GameMaster gate (`command.rs:293-305,405-413`): FIXED / VERIFIED. Both call `can_use_game_master_blocks()` and `normal_use` returns `Consume`. Command blocks implement `GameMasterBlock` and vanilla gates on `player.canUseGameMasterBlocks()` (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/CommandBlock.java:31,135`); `onUse` returns `ActionResult.CONSUME`.
- 3. `chain_execute` cap (`command.rs:25,207,264-268`): FIXED / VERIFIED. `MAX_CHAIN_LENGTH = 65536` with `debug!`, matching the vanilla default `MAX_COMMAND_SEQUENCE_LENGTH = 65536` (`.vendor/vanilla-decompiled/net/minecraft/world/level/gamerules/GameRules.java:53`).
- 4. `last_output` gated by TrackOutput (`command/mod.rs:156-163`): FIXED / VERIFIED. `send_system_message` returns early when `track_output` is false, matching vanilla `BaseCommandBlock.CloseableCommandBlockSource#sendSystemMessage` which writes LastOutput only when `trackOutput` (`.vendor/vanilla-decompiled/net/minecraft/world/level/BaseCommandBlock.java`, `save`/`load` `TrackOutput` at lines 59/76).
- 5. Block entity `CustomName` / `LastExecution` and command-source name: FIXED / VERIFIED. `CommandBlockEntity` carries `custom_name`, `last_execution`, `update_last_execution` with NBT round-trip (`entities/command_block.rs:21-26,93-98,124-157`). `CommandBlock::execute` early-returns on `world.get_world_age() == last_execution` and records the tick under `update_last_execution` (`command.rs:161-203`), matching `if (level.getGameTime() == this.lastExecution) return false` and `UpdateLastExecution` (`.vendor/vanilla-decompiled/net/minecraft/world/level/BaseCommandBlock.java:95,59,76`). `CommandSender::into_source` derives the name from `command_block_name`, which returns `custom_name` or `@`, matching `getName` and `DEFAULT_NAME = Component.literal("@")` (`BaseCommandBlock.java:148,24`); pure test `command_block_name_matches_vanilla_get_name` present.
- 6. Bedrock editor round-trip: BLOCKED-DATA. Missing subsystem: a live Bedrock client runtime harness (`scripts/boot.sh`) to observe the editor opening and the `SBlockActorData` round-trip; no generated `pumpkin-data` field is missing. The server-side static path is complete and unit-tested (`CommandBlockEntity::bedrock_block_actor_data` -> `CBlockActorData`, `BedrockClient::handle_command_block_actor_data`, plus tests `command_block_actor_builder`, `parse_command_block_edit_strips_slash_and_reads_flags`, `parse_command_block_edit_defaults_when_fields_absent`); Java screen-open replays vanilla's block-entity data packet (`ServerPlayer#openCommandBlock`) via `World::update_block_entity`. The residual observation is a client-only GUI effect that does not block landing per AGENTS.md.

No source file changed this pass (all fixes landed earlier); only this tracker doc was finalized.
