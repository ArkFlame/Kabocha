# entities / Command Block Minecart

Status: RE-REVIEWED (2026-11-17, source unchanged since the previous audit commit d155c012c; re-checked against
`block/blocks/command.rs` and `block/entities/command_block.rs`, which are the repo's port of the same
vanilla `BaseCommandBlock` path).
Rust: `crates/pumpkin-core/src/entity/vehicle/minecart/command_block.rs` + `minecart.rs` +
`crates/pumpkin-core/src/net/java/play/set_command_minecart.rs`.
Vanilla: `MinecartCommandBlock` (+ `AbstractMinecart`), `BaseCommandBlock`,
`ServerGamePacketListenerImpl#handleSetCommandMinecart`. No decompiled source under `.vendor/`, so the
remaining items below are marked when they are inferred rather than confirmed.

## Previously fixed (unchanged, still correct)
- Exact `"Searge".equals(command)` comparison (b67f2cb41).
- Tracked output re-synced when a command prints nothing (a606e1ffa).
- `handleSetCommandMinecart` always applies command/trackOutput regardless of `commandBlocksWork` (4de2458f8).
- Activation cooldown `tickCount - lastActivated >= 4`, `lastExecution` guard, NBT keys, drop item,
  tracked data ids: verified matching.

## Fixed in this pass
- `command_block.rs:157-164` the Searge easter egg ran even with the `commandBlocksWork` gamerule off.
  `block/blocks/command.rs:148-168` (the audited port of the same `BaseCommandBlock#performCommand`) gates it
  behind the gamerule and the empty-command check. Now gated on `command_blocks_work`. Vanilla class:
  `BaseCommandBlock`. Commit in this branch (feat(command): gate minecart command block easter egg on the gamerule).

## Remaining gaps
1. `command_block.rs:151-153` last-execution bookkeeping happens after execution, so the early-return path
   (Searge, now also the gamerule-off path) never stores `last_execution`. If vanilla assigns
   `lastExecution = gameTime` before dispatching (as the block path implies), a Searge cart re-executes every
   activation instead of once per tick/activation. Vanilla class: `BaseCommandBlock#performCommand`.
   Fix: compute and store `lastExecution` (respecting `UpdateLastExecution`) before the command body, then
   return early. Size: S (needs decompile to confirm ordering).
2. `command_block.rs:168` `success_count` is zeroed even when `commandBlocksWork` is false; the block port
   resets it only inside the allowed path. Success count feeds conditional chain blocks, so a disabled cart
   could clear another block's condition state. Vanilla class: `BaseCommandBlock#performCommand`.
   Fix: move the reset inside the gamerule guard. Size: S (confirm exact ordering).
3. `command_block.rs:233-235` `interact` returns a bool and never opens the editor. This matches vanilla
   server behaviour: the client opens `CommandBlockEditScreen` itself in `Minecraft#startUseItem` and sends
   no packet, so the server only needs PASS (false) for non-gamemasters and SUCCESS (true) for
   creative+permission 2. Remaining risk only: a client that skips the local screen has no server fallback.
   Vanilla class: `MinecartCommandBlock#interact`. Fix: none unless a client shows no screen. Size: S.
4. `command_block.rs:114-120, 240-250, 282-286` `custom_name` is stored and (de)serialized as `CustomName`
   inside the command-block state, while `Entity.custom_name` (`entity/mod.rs:1343-1365`) already owns the
   entity display name. Vanilla keeps the name on the entity, not on `BaseCommandBlock`. Duplicate sources can
   diverge (rename via name tag vs data command). Vanilla class: `MinecartCommandBlock` + `Entity`.
   Fix: drop the command-block copy and read the entity name in `get_name`, defaulting to "@". Size: S.
5. `command_block.rs:245-263` `write_nbt` hardcodes `JavaMinecraftVersion::V_26_3` when serializing
   `CustomName`/`LastOutput`. 16 other sites do the same, so this is repo convention, not an entity bug:
   only matters once multiple protocol versions are served. Vanilla class: `Component.Serializer`.
   Fix: thread the target version through entity NBT saving when multi-version lands. Size: M.
6. No automated test covers the gamerule-off easter-egg path added here (tests were not run in this pass,
   compile was forbidden by the brief). Suggest a unit test in `minecart.rs` toggling
   `game_rules.command_blocks_work` and asserting `success_count == 0` / unchanged output.
   Vanilla class: `BaseCommandBlock#performCommand`. Size: S.
