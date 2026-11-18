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
- Gap 4 (entity display name): VERIFIED (matches vanilla), no code change.
- Gap 5 (NBT component version): BLOCKED-DATA (whole multi-version NBT serialization subsystem).
- Gap 6 (gamerule-off test): VERIFIED (matches vanilla), no code change.

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


## Result (2026-11-18 third pass, terminal disposition)

All remaining STILL OPEN rows (doc lines 43, 47, 50) re-checked against decompiled vanilla under
`.vendor/vanilla-decompiled/net/minecraft/`. No `.rs` source changed: the code in scope already matches
vanilla, so rustfmt was not needed.

- Gap 4 (entity display name, doc lines 41-45): VERIFIED (matches vanilla), no code change. Premise is
  false. `.vendor/vanilla-decompiled/net/minecraft/world/level/BaseCommandBlock.java:32,147-149,53,68`
  owns its own `customName`, `getName()` returns it or the `Component.literal("@")` default, and NBT
  stores it under `CustomName`. `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/minecart/MinecartCommandBlock.java:32`
  embeds that `BaseCommandBlock`. Pumpkin's `CommandBlockMinecart.custom_name` plus `get_name()`
  defaulting to `@` mirror vanilla; dropping the copy would diverge, so no change was made.
- Gap 5 (NBT component version, doc lines 46-49): BLOCKED-DATA. Missing subsystem: version-aware
  text-component NBT serialization. `crates/pumpkin-util/src/text/mod.rs:1697-1699`
  `TextComponent::to_nbt_tag()` hardcodes `JavaMinecraftVersion::V_26_3`; threading a target version
  through Entity NBT saving is a cross-crate refactor across ~16 sites plus the util, i.e. a whole
  multi-version serialization subsystem, outside a minimal single-unit fix. No generated pumpkin-data
  field is missing. Vanilla serializes components via `ComponentSerialization.CODEC`
  (`BaseCommandBlock.java:53,56`), which needs no version argument server-side.
- Gap 6 (gamerule-off test, doc lines 50-53): VERIFIED (matches vanilla), no new test needed.
  `crates/pumpkin-core/src/entity/vehicle/minecart.rs:1234-1274`
  `command_block_minecart_perform_command_gamerule` sets `command_blocks_work = false`, asserts
  `success_count == 0` and empty output for a normal command, then asserts the case-insensitive Searge
  path still fires. This matches `.vendor/vanilla-decompiled/net/minecraft/world/level/BaseCommandBlock.java:99-105,134-138`:
  `equalsIgnoreCase` Searge check before the gamerule guard, `successCount` reset before the guard, and
  `lastExecution` updated at the end. `command_block.rs::perform_command` and
  `set_command_minecart.rs` already match this ordering.

No STILL OPEN rows remain.


## Result (2026-11-18 third pass, terminal verification)

Re-verified every remaining row against `.vendor/vanilla-decompiled/net/minecraft/`. No `.rs` source
in this unit changed: the code in scope already matches vanilla, so rustfmt was not needed.

- Gap 4 (entity display name, doc lines 41-45): VERIFIED (matches vanilla), no code change. Premise is
  false. `.vendor/vanilla-decompiled/net/minecraft/world/level/BaseCommandBlock.java:32,147-149,53,68`
  owns `customName`, `getName()` returns it or the `Component.literal("@")` default, and NBT stores it
  under `CustomName`. `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/minecart/MinecartCommandBlock.java:32`
  embeds that `BaseCommandBlock`. `command_block.rs:44,112-118,233-240,268-272` mirrors this; dropping
  the copy would diverge from vanilla.
- Gap 5 (NBT component version, doc lines 46-49): BLOCKED-DATA. Missing subsystem: version-aware text
  component NBT serialization. `crates/pumpkin-util/src/text/mod.rs:1697-1699`
  `TextComponent::to_nbt_tag()` hardcodes `JavaMinecraftVersion::V_26_3`; the version-aware variant
  `to_nbt_tag_for_version()` exists but threading a target version through Entity NBT saving is a
  cross-crate refactor across ~16 sites plus the util, i.e. a whole multi-version serialization
  subsystem, outside a minimal single-unit fix. No generated pumpkin-data field is missing. Vanilla
  serializes via `ComponentSerialization.CODEC` (`BaseCommandBlock.java:53,56`), which needs no version
  argument server-side, so current behavior matches vanilla for the single served version.
- Gap 6 (gamerule-off test, doc lines 50-53): VERIFIED (matches vanilla), no new test needed.
  `crates/pumpkin-core/src/entity/vehicle/minecart.rs:1233-1274`
  `command_block_minecart_perform_command_gamerule` sets `command_blocks_work = false`, asserts
  `success_count == 0` and empty output for a normal command, then asserts the case-insensitive Searge
  path still fires. Matches `.vendor/vanilla-decompiled/net/minecraft/world/level/BaseCommandBlock.java:95-105,134-140`:
  game-time early return, `equalsIgnoreCase` Searge check before the gamerule guard with no
  `lastExecution` update on that path, `successCount` reset before the guard, `lastExecution` updated at
  the end. `command_block.rs::perform_command` and `set_command_minecart.rs:34-39` already match this
  ordering (`on_updated` gated behind the gamerule, matching
  `ServerGamePacketListenerImpl.java:1112-1119`).

No STILL OPEN rows remain. All six gaps have terminal dispositions; the three carried-over gaps are
VERIFIED, VERIFIED, and BLOCKED-DATA above.


## Result (2026-11-18 FINAL PASS, terminal disposition)

Re-verified every row against decompiled vanilla under `.vendor/vanilla-decompiled/net/minecraft/`.
No `.rs` source needed editing: both in-scope files already match vanilla, so rustfmt was not required.
No `STILL OPEN` rows and no `STILL OPEN until closed...` line remain.

- Gap 1 (last-execution on early return, doc lines 26-31): VERIFIED (matches vanilla), no code change.
  `world/level/BaseCommandBlock.java:99-103` returns on the Searge path before the `lastExecution` update
  at lines 134-138. Pumpkin `command_block.rs:160-165` likewise returns early, so both leave
  `lastExecution` untouched on that path; `perform_command` updates it at the end (lines 217-222).
- Gap 2 (success_count reset with gamerule off, doc lines 32-35): VERIFIED (matches vanilla), no code
  change. `BaseCommandBlock.java:105` resets `successCount` before the `isCommandBlockEnabled` guard at
  line 106. Pumpkin `command_block.rs:168` resets it before the guard at line 169.
- Gap 3 (interact opens no editor, doc lines 36-40): VERIFIED (matches vanilla), no code change.
  `MinecartCommandBlock#interact` opens no server GUI; the client opens `CommandBlockEditScreen`. Pumpkin
  `command_block.rs:226-228` returns the creative+permission-2 boolean only.
- Gap 4 (entity display name, doc lines 41-45): VERIFIED (matches vanilla), no code change; premise false.
  `world/level/BaseCommandBlock.java:32,147-149,53,68` owns `customName`, `getName()` returns it or the
  `Component.literal("@")` default, and NBT stores it under `CustomName`. `world/entity/vehicle/minecart/MinecartCommandBlock.java:32`
  embeds that `BaseCommandBlock`. Pumpkin `CommandBlockMinecart.custom_name` + `get_name()` defaulting to
  `@` mirror vanilla; dropping the copy would diverge.
- Gap 5 (NBT component version, doc lines 46-49): BLOCKED-DATA. Missing subsystem: version-aware text
  component NBT serialization. `crates/pumpkin-util/src/text/mod.rs:1697-1699` `TextComponent::to_nbt_tag()`
  hardcodes `JavaMinecraftVersion::V_26_3`; threading a target version through Entity NBT saving is a
  cross-crate refactor across ~16 sites plus the util, a whole multi-version serialization subsystem,
  outside a minimal single-unit fix. No generated pumpkin-data field is missing. Vanilla serializes via
  `ComponentSerialization.CODEC` (`BaseCommandBlock.java:53,56`), which needs no version argument
  server-side, so current behavior matches vanilla for the single served version.
- Gap 6 (gamerule-off test, doc lines 50-53): VERIFIED (matches vanilla), no new test needed.
  `crates/pumpkin-core/src/entity/vehicle/minecart.rs:1321-1361`
  `command_block_minecart_perform_command_gamerule` sets `command_blocks_work = false`, asserts
  `success_count == 0` and empty output for a normal command, then asserts the case-insensitive Searge
  path still fires. Matches `BaseCommandBlock.java:95-105,134-138`: game-time early return,
  `equalsIgnoreCase` Searge check before the guard with no `lastExecution` update on that path,
  `successCount` reset before the guard, `lastExecution` updated at the end.

Adjacent packet handler already fixed in source: `net/java/play/set_command_minecart.rs:9-39` keeps
setCommand/setTrackOutput/lastOutput unconditional and gates `on_updated` behind the gamerule, matching
`.vendor/vanilla-decompiled/net/minecraft/server/network/ServerGamePacketListenerImpl.java:1092-1120`.
`block/blocks/command.rs` uses `eq_ignore_ascii_case` for Searge before the guard. No source edit needed.
