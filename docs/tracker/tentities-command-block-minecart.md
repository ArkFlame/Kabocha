# entities / Command Block Minecart

Status: AUDITED (2026-11-17, re-review after the reviewed source changed; tracking row coverage RE-REVIEW-UNTRACKED).
Rust: `crates/pumpkin-core/src/entity/vehicle/minecart/command_block.rs` + `minecart.rs` +
`crates/pumpkin-core/src/net/java/play/set_command_minecart.rs`.
Vanilla: `MinecartCommandBlock` (+ `AbstractMinecart`), `BaseCommandBlock`,
`ServerGamePacketListenerImpl#handleSetCommandMinecart`.

## Fixed in this pass
- `command_block.rs:159` Searge easter egg used `eq_ignore_ascii_case`, vanilla `BaseCommandBlock#performCommand`
  uses `"Searge".equals(command)` -> exact comparison. Commit `4b0d4ab25`.
- `command_block.rs:206-220` tracked output was only written/synced when the captured string was non-empty, so a
  command that printed nothing kept the previous `DATA_ID_LAST_OUTPUT` on clients. Vanilla writes `lastOutput`
  back on every execution; now an empty capture clears and re-syncs. Commit `ca39bd55a`.
- `set_command_minecart.rs:34-37` re-synced entity data only when the `commandBlocksWork` gamerule was on.
  Vanilla `handleSetCommandMinecart` always applies `setCommand`/`setTrackOutput` (and `setCommand` fires
  `onUpdated`); the gamerule only gates execution. Now always re-synced, gamerule kept only for the chat message.
  Commit `1ca0ad406`.

## Verified matching (no change)
- Activation cooldown `tickCount - lastActivated >= 4` (`command_block.rs:141-149`), `lastExecution`
  game-time guard, `UpdateLastExecution` / `-1` sentinel handling (`151-234`).
- NBT keys `Command`, `SuccessCount`, `TrackOutput`, `LastOutput` (only when tracking), `UpdateLastExecution`,
  `LastExecution` (only when updating) in `237-302`.
- `drop_item()` returns `COMMAND_BLOCK_MINECART` (`minecart.rs:173`); tracked data ids
  `DATA_ID_COMMAND_NAME` / `DATA_ID_LAST_OUTPUT`.

## Remaining gaps
1. `command_block.rs:157-164` the Searge branch runs before the `command_blocks_work` / empty-command guard.
   Vanilla keeps the easter egg inside the gamerule branch of `BaseCommandBlock#performCommand`, so with command
   blocks disabled it must not answer. Vanilla class: `BaseCommandBlock`. Fix: move the branch inside the guard.
   Size: S (behavioural, confirm against decompile).
2. `command_block.rs:75-81` `set_command` zeroes `success_count`; vanilla `BaseCommandBlock#setCommand` only
   stores the string and calls `onUpdated` (no `successCount` reset documented). Vanilla class:
   `BaseCommandBlock`. Fix: drop the reset and only re-sync. Size: S (confirm against decompile).
3. `command_block.rs:233-235` / `minecart.rs:912` `interact` returns a bool and never opens the editor, so a
   creative gamemaster clicking a command block minecart gets no GUI. Vanilla `MinecartCommandBlock#interact`
   returns `InteractionResult.PASS` for non-gamemasters (no click is swallowed) and the client opens the command
   block editor. Vanilla class: `MinecartCommandBlock`. Fix: tri-state result (PASS/SUCCESS) and wire the
   editor-open path. Size: M (GUI/container path).
4. `command_block.rs:114-120, 273-302` `BaseCommandBlock.customName` lives only in the command-block struct,
   while entity display name (`entity/mod.rs:1334`) reads `entity.custom_name`. Vanilla
   `MinecartCommandBlock#getName` prefers the command block custom name over `super.getName()`. Fix: mirror
   `CustomName` from `read_nbt` into the entity name (or override the entity name lookup). Size: S (verify).
5. `command_block.rs:237-266` `write_nbt` hardcodes `JavaMinecraftVersion::V_26_3` when serializing
   `CustomName` / `LastOutput`; it should use the entity/protocol version so older clients keep their text
   format. Vanilla class: `Component.Serializer` via `BaseCommandBlock#save`. Size: S.
