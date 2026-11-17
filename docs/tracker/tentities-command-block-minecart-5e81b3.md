# entities / Command Block Minecart

Status: RE-REVIEWED (2026-11-17, branch `kab/tentities-command-block-minecart-5e81b3`).
The tracker said the reviewed source changed; it has not. `git diff e28c42289 --` over
`entity/vehicle/minecart/command_block.rs` and `entity/vehicle/minecart.rs` is empty, so this pass
re-audits the same code against vanilla again and re-states the open items below. No code change was
committed: none of the open items is certain without decompiled `BaseCommandBlock`, and the previous
gamerule fix (bcd8a34e6) is already in the branch.

Rust: `crates/pumpkin-core/src/entity/vehicle/minecart/command_block.rs` (1-311) +
`crates/pumpkin-core/src/entity/vehicle/minecart.rs` (116-213 kind dispatch, 230 tick, 332 activator
rail, 793-804 tracker init, 904-920 interact) + `crates/pumpkin-core/src/net/java/play/set_command_minecart.rs`.
Vanilla: `MinecartCommandBlock` (+ inner `MinecartCommandBase`), `AbstractMinecart#activateMinecart`,
`BaseCommandBlock`, `ServerGamePacketListenerImpl#handleSetCommandMinecart`. No decompiled source under
`.vendor/`, so items marked *inferred* are unconfirmed.

## Confirmed matching (no gap)
- `activate()` cooldown `tickCount - lastActivated >= 4` and the guard `gameTime == lastExecution`
  (`command_block.rs:141-149`, `151-155`).
- Gamerule `commandBlocksWork` gates both the Searge easter egg and command execution
  (`command_block.rs:158-169`); the same early return exists in the block port
  (`block/blocks/command.rs:146-150`).
- `success_count` is taken from a `CommandResult` callback, not from the dispatcher return
  (`command_block.rs:23-34`, `183-207`).
- Tracked data `DATA_ID_COMMAND_NAME` / `DATA_ID_LAST_OUTPUT` are re-synced after every execution and on
  load, and `init_data_tracker` is reached from `minecart.rs:793-804`, so initial spawn sync is correct.
- NBT keys and defaults: `Command`, `SuccessCount`, `CustomName`, `TrackOutput`, `LastOutput`,
  `UpdateLastExecution`, `LastExecution` (`command_block.rs:239-310`); `LastOutput` is dropped when
  `TrackOutput=false` and `LastExecution` is only written when `UpdateLastExecution` is true.
- `set_command` clears the success count and the packet handler applies command/trackOutput regardless of
  the gamerule and re-syncs (`command_block.rs:75-81`, `set_command_minecart.rs:28-36`).
- Drop item and entity name default `"@"` (`minecart.rs:886-895`, `command_block.rs:114-120`).
- `interact` opens no server-side GUI; vanilla's `MinecartCommandBlockEditScreen` is opened by the client,
  so only the creative+permission-2 check is server-observable (`command_block.rs:235-237`).

Existing automated coverage: `minecart.rs:975-1030` (`command_block_minecart_nbt_and_activation_cooldown`)
covers NBT round trip, `TrackOutput=false`, cooldown and the Searge output.

## Remaining gaps
1. `command_block.rs:168` resets `success_count` even when `commandBlocksWork` is false. The block port
   (`block/blocks/command.rs:146-150`) returns before any state change, so a disabled cart clears the
   persisted `SuccessCount` (visible through `data get`) while a disabled block does not.
   Vanilla class: `BaseCommandBlock#performCommand`. *Inferred* (no decompile). Fix: move the reset inside
   the `command_blocks_work` guard, keeping it before the empty-command check. Size: S.
2. `command_block.rs:161-166` the Searge branch returns before the `lastExecution` bookkeeping at
   `226-231`. If vanilla assigns `lastExecution` after the command body (as the block port's structure
   implies), a Searge cart keeps an old `lastExecution` in NBT and skips the `UpdateLastExecution` write.
   Vanilla class: `BaseCommandBlock#performCommand`. *Inferred*. Fix: compute `lastExecution` once before
   the return points (or fall through to it). Size: S.
3. `command_block.rs:46`, `114-120`, `242-252`, `283-287` keep `custom_name` on the command-block state,
   duplicating `Entity.custom_name` (`entity/mod.rs:1343-1365`). Renaming via name tag vs `CustomName` NBT
   can make the command-source name and the entity display name diverge. Vanilla keeps the name on the
   entity only. Fix: drop the field and read the entity name in `get_name`, defaulting to `"@"`. Size: S.
4. `command_block.rs:250,264` hardcodes `JavaMinecraftVersion::V_26_3` when serialising
   `CustomName`/`LastOutput`. Repo-wide convention (16 other sites), so not an entity bug; it matters only
   when several protocol versions are served. Vanilla class: `Component.Serializer`. Fix: thread the
   target version through entity NBT saving. Size: M.
5. No test covers the gamerule-off path (`commandBlocksWork=false`): neither the easter egg nor the
   `success_count` reset. Suggest extending `command_block_minecart_nbt_and_activation_cooldown`
   (`minecart.rs:975`) by flipping `world.level_info.game_rules.command_blocks_work` and asserting the
   command does not run and `success_count` is unchanged. Vanilla class: `BaseCommandBlock#performCommand`.
   Size: S.
6. Remaining risk, not a gap: a client that does not open `MinecartCommandBlockEditScreen` locally gets no
   server fallback for `interact` (`command_block.rs:235-237`). Size: S.

## Gap status
- Remaining-gap items 1-5 (CustomName/LastOutput persistence, performCommand gating, etc.): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 6 (risk, not a gap): n/a, no action
