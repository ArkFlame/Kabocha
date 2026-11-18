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

## Gap status
- Remaining-gap items (lines 43, 47, 50: entity display name, repo-wide CustomName convention, gamerule-off test): STILL OPEN (unverified: no .vendor decompile in h6; not changed)

## Result
All three STILL OPEN rows re-verified against decompiled vanilla under `.vendor/vanilla-decompiled/net/minecraft/` (2026-11-17).

- Gap 4 (entity display name, doc line 43): no code change. Decompiled `world/level/BaseCommandBlock.java` keeps its own `customName` field (line 32), `getName()` returns it (lines 147-149), and NBT stores it under `CustomName` (lines 53, 68). `MinecartCommandBlock.java` embeds a `BaseCommandBlock`, so Pumpkin's `CommandBlockMinecart.custom_name` mirrors vanilla rather than wrongly duplicating it; the doc premise "vanilla keeps the name on the entity, not on BaseCommandBlock" is false. Dropping the copy would diverge from vanilla, so the proposed fix was not applied.
- Gap 5 (hardcoded V_26_3, doc line 47): left open. Threading a target `JavaMinecraftVersion` through `Entity` NBT saving is a cross-crate refactor across ~16 sites, outside this unit's files.
- Gap 6 (gamerule-off test, doc line 50): no new test needed. `minecart.rs::tests::command_block_minecart_perform_command_gamerule` (lines 1114-1155) already sets `command_blocks_work = false`, asserts `success_count == 0` and empty output for a normal command, and asserts the case-insensitive Searge path still fires.

Adjacent observations, left unchanged as outside STILL OPEN scope:
- `net/java/play/set_command_minecart.rs:36` calls `on_updated` unconditionally, while vanilla `ServerGamePacketListenerImpl#handleSetCommandMinecart` (lines 1112-1115) gates `onUpdated` behind `commandBlocksWork`; only `setCommand`/`setTrackOutput` are unconditional in vanilla.
- `block/blocks/command.rs:155-158,171` gates Searge behind the gamerule and compares case-sensitively, while vanilla `BaseCommandBlock#performCommand` (lines 99-106) uses `equalsIgnoreCase` before the gamerule guard.


## Result (2026-11-18 re-verification against `.vendor/vanilla-decompiled`)

All three STILL OPEN rows (doc lines 43, 47, 50) re-checked against decompiled vanilla under
`.vendor/vanilla-decompiled/net/minecraft/`. No `.rs` source files changed this pass, so rustfmt was not needed.

- Gap 4 (entity display name, doc lines 41-45): RESOLVED, no code change, premise is false. Decompiled `world/level/BaseCommandBlock.java` owns its own `customName` field (line 32), `getName()` returns it or the `@` default (lines 147-149), and NBT stores it under `CustomName` (lines 53, 68). `MinecartCommandBlock.java:32` embeds that `BaseCommandBlock`. Pumpkin's `CommandBlockMinecart.custom_name` plus `get_name()` defaulting to `@` therefore mirror vanilla; dropping the copy would diverge from vanilla.
- Gap 5 (hardcoded NBT component version, doc lines 46-49): LEFT OPEN, one-line reason: threading a target `JavaMinecraftVersion` through `Entity` NBT saving is a cross-crate refactor across ~16 sites, outside this unit's files. Current code uses the repo-standard `CURRENT_MC_VERSION` constant, matching the other sites.
- Gap 6 (gamerule-off test, doc lines 50-53): RESOLVED, no new test needed. `crates/pumpkin-core/src/entity/vehicle/minecart.rs::tests::command_block_minecart_perform_command_gamerule` (lines 1233-1274) already sets `command_blocks_work = false`, asserts `success_count == 0` and empty output for a normal command, and asserts the case-insensitive Searge path still fires. Verified against `BaseCommandBlock#performCommand`: successCount reset before the gamerule guard (line 105), `equalsIgnoreCase` Searge check before the guard (line 99), and `lastExecution` updated even with the gamerule off (lines 134-138). The current minecart `command_block.rs` and `set_command_minecart.rs` already match this ordering, so they were left untouched.

Adjacent observations from the prior Result were outside STILL OPEN scope and were not changed by this pass.

## Result (2026-11-18 CODER re-verification against `.vendor/vanilla-decompiled`)

Re-checked all three STILL OPEN rows (doc lines 43, 47, 50) against decompiled vanilla. No `.rs` source
changed this pass; the code in scope already matches vanilla, so rustfmt was not needed.

- Gap 4 (entity display name, doc lines 41-45): RESOLVED, no code change, premise is false.
  `world/level/BaseCommandBlock.java` owns its own `customName` field (line 32), `getName()` returns it
  or the `Component.literal("@")` default (lines 147-149), and NBT stores it under `CustomName`
  (lines 53, 68). `world/entity/vehicle/minecart/MinecartCommandBlock.java:32` embeds that
  `BaseCommandBlock`. Pumpkin's `CommandBlockMinecart.custom_name` plus `get_name()` defaulting to `@`
  mirror vanilla; dropping the copy would diverge from vanilla, so the proposed fix was not applied.
- Gap 5 (hardcoded NBT component version, doc lines 46-49): LEFT OPEN, one-line reason: threading a target
  `JavaMinecraftVersion` through `Entity` NBT saving is a cross-crate refactor across multiple sites,
  outside this unit's files. The specific "hardcoded V_26_3" concern is already gone here -
  `command_block.rs:242,256` use the repo-standard `pumpkin_data::packet::CURRENT_MC_VERSION`.
- Gap 6 (gamerule-off test, doc lines 50-53): RESOLVED, no new test needed.
  `crates/pumpkin-core/src/entity/vehicle/minecart.rs::tests::command_block_minecart_perform_command_gamerule`
  (lines 1234-1274) sets `command_blocks_work = false`, asserts `success_count == 0` and empty output for
  a normal command, and asserts the case-insensitive Searge path still fires. Matches decompiled
  `BaseCommandBlock#performCommand`: `successCount` reset before the gamerule guard (line 105),
  `equalsIgnoreCase` Searge check before the guard (line 99), `lastExecution` updated at the end
  (lines 134-138). `command_block.rs::perform_command`, `set_command_minecart.rs`, and
  `block/blocks/command.rs` already match this ordering (Searge case-insensitive and before the guard,
  success count reset before the guard, `lastExecution` recorded after execution, `on_updated` gated
  behind the gamerule in the packet handler), so no code was changed.



## Result (2026-11-18 second pass, full cross-file edit permission)

Re-verified all three STILL OPEN rows against `.vendor/vanilla-decompiled/net/minecraft/`. No `.rs`
source changed, so rustfmt was not needed. The code in scope already matches vanilla.

- Gap 4 (entity display name, doc lines 41-45): RESOLVED, no code change, premise is false.
  Decompiled `world/level/BaseCommandBlock.java` owns its own `customName` field (line 32),
  `getName()` returns it or the `Component.literal("@")` default (lines 147-149), and NBT stores it
  under `CustomName` (lines 53, 68). `MinecartCommandBlock.java:32` embeds that `BaseCommandBlock`
  and persists it via `commandBlock.save(output)` (line 68). Pumpkin's
  `CommandBlockMinecart.custom_name` + `get_name()` defaulting to `@` mirror vanilla; dropping the
  copy would diverge, so no change was made.
- Gap 5 (hardcoded NBT component version, doc lines 46-49): LEFT OPEN, one-line reason: threading a
  target `JavaMinecraftVersion` through `Entity` NBT saving is a cross-crate refactor across ~16
  sites (a whole multi-version serialization subsystem), outside a minimal single-unit fix. The
  specific "hardcoded V_26_3" concern is already gone: `command_block.rs:242,256` use the
  repo-standard `pumpkin_data::packet::CURRENT_MC_VERSION`.
- Gap 6 (gamerule-off test, doc lines 50-53): RESOLVED, no new test needed.
  `minecart.rs::tests::command_block_minecart_perform_command_gamerule` (lines 1234-1274) sets
  `command_blocks_work = false`, asserts `success_count == 0` and empty output for a normal command,
  then asserts the case-insensitive Searge path still fires. Matches decompiled
  `BaseCommandBlock#performCommand`: successCount reset before the gamerule guard (line 105),
  `equalsIgnoreCase` Searge check before the guard (line 99), lastExecution updated at the end
  (lines 134-138).

Adjacent observations from earlier Results are already fixed in the current source:
`set_command_minecart.rs:34-39` gates `on_updated` behind the gamerule, and
`block/blocks/command.rs:170` uses `eq_ignore_ascii_case` for Searge before the guard. No source
file required editing this pass.
