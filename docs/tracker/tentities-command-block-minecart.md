# entities / Command Block Minecart

Status: AUDITED (2026-11-16, re-review after `6f7885ca7 fix(command): minecart command block counter returns Result`).
Rust: `crates/pumpkin-core/src/entity/vehicle/minecart/command_block.rs` + `minecart.rs`.
Vanilla: `MinecartCommandBlock` (+ `AbstractMinecart`), `BaseCommandBlock`, `ServerGamePacketListenerImpl#handleSetCommandMinecart`.

## Fixed in this pass
- `minecart.rs:167` `drop_item()` returned `Item::MINECART` for the command block kind. Vanilla
  `MinecartCommandBlock#getDropItem()` returns `Items.COMMAND_BLOCK_MINECART`. Fixed + test
  `minecart_drop_item_matches_vanilla`.

## Remaining gaps
1. `command_block.rs:158` Searge branch uses `command.eq_ignore_ascii_case("Searge")`; `BaseCommandBlock#performCommand`
   compares with a plain `equals("Searge")` constant, so `searge`/`SEARGE` should not trigger. Vanilla class:
   `BaseCommandBlock`. Fix: exact `==`. Size: S.
2. `command_block.rs:205-215` last output is only written/synced when the captured output string is non-empty.
   `BaseCommandBlock#performCommand` runs its callback on every execution and calls `setLastOutput(...)` when
   `trackOutput`, so an execution with empty output must clear the client's `DATA_ID_LAST_OUTPUT` instead of
   leaving the previous text. Vanilla class: `BaseCommandBlock`. Fix: always `set_last_output(...)` + `on_updated`
   when `track_output`, even when captured is empty. Size: S.
3. `command_block.rs:151,219-226` empty-command path: Rust still resets `success_count` and stores
   `last_execution` when `command.is_empty()` or the gamerule is off. Verify against `BaseCommandBlock#performCommand`
   (gameTime guard -> gamerule/empty guard; `lastExecution` update ordering, and whether the method returns false
   without updating it). Vanilla class: `BaseCommandBlock`. Size: S (behavioural, needs decompile to confirm).
4. `minecart.rs:912` `CommandBlockMinecart::interact` returns `false` when the player is not creative or below
   permission 2; vanilla `MinecartCommandBlock#interact` returns `InteractionResult.PASS` for non-game-master
   players (rideable check happens in `AbstractMinecart`), so the client should not be told the click was handled.
   Vanilla class: `MinecartCommandBlock`. Fix: return true only for `canUseGameMasterBlocks`
   (creative + perm 2), which matches; otherwise keep PASS semantics. Size: S (needs decompile confirmation).
5. `command_block.rs:119` `get_name()` falls back to `"@"`; that matches `BaseCommandBlock.DEFAULT_NAME` for the
   command source only. Entity display/death message name should stay the entity type name (no code change expected;
   noted so a future refactor does not push `"@"` into the entity name). Vanilla class: `BaseCommandBlock`. Size: S.
6. `set_command_minecart.rs:34-37` only re-syncs entity data when the `commandBlocksWork` gamerule is on. Vanilla
   `ServerGamePacketListenerImpl#handleSetCommandMinecart` calls `setCommand`/`setTrackOutput` unconditionally;
   `on_updated` fires from `setCommand`. Verify that a disabled gamerule must not suppress the client sync.
   Vanilla class: `ServerGamePacketListenerImpl`. Size: S.
