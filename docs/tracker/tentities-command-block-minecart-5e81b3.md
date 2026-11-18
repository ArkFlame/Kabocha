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

## Gap status (final, re-verified against decompiled vanilla in .vendor)
- Row 1 (success_count reset before gamerule guard): NO GAP - vanilla BaseCommandBlock.java line 105 resets SuccessCount before the guard at line 106; Pumpkin matches.
- Row 2 (Searge skips lastExecution): NO GAP - vanilla returns at line 102 before the lastExecution write at lines 134-138; Pumpkin matches.
- Row 3 (custom_name on command-block state): NO GAP - vanilla keeps its own customName field and getName() defaults to "@"; Pumpkin matches.
- Row 4 (thread target protocol version through entity NBT): STILL OPEN (BLOCKED) - cross-crate refactor outside this unit's files and a repo-wide convention.
- Row 5 (gamerule-off test): FIXED - command_block_minecart_perform_command_gamerule added in minecart.rs.
- Row 6 (no GUI fallback): n/a, risk only.


## Result

Verified against decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/world/level/BaseCommandBlock.java` and
`.../world/entity/vehicle/minecart/MinecartCommandBlock.java` (now present in `.vendor`).

- Row 1 (success_count reset when gamerule off): NO GAP. Vanilla `performCommand` sets
  `successCount = 0` at line 105, before the `isCommandBlockEnabled()` guard at line 106, so a
  disabled cart does reset it. Pumpkin already does this. No code change.
- Row 2 (Searge skips lastExecution bookkeeping): NO GAP. Vanilla returns from the Searge branch
  at line 102, before the `lastExecution` write at lines 134-138. Pumpkin already does this. No
  code change.
- Row 3 (custom_name duplicated on command-block state): NO GAP. Vanilla `BaseCommandBlock`
  keeps its own `customName` field (line 32) and `getName()` returns it or "@" (lines 147-149),
  independent of the entity name. Pumpkin matches. No code change.
- Row 4 (hardcoded `JavaMinecraftVersion::V_26_3`): STILL OPEN (BLOCKED). Needs a cross-crate
  refactor to thread the target protocol version through entity NBT saving; repo-wide convention
  (16 other sites) and outside this unit's files.
- Row 5 (gamerule-off test): FIXED. Added `command_block_minecart_perform_command_gamerule` in
  `minecart.rs`. It asserts the vanilla behaviour: a normal command with `commandBlocksWork=false`
  resets SuccessCount to 0 and does not execute; the Searge easter egg still fires. The doc's
  suggested assertion "success_count unchanged" is contradicted by vanilla line 105 and was not
  used.
- Row 6 (no server GUI fallback): n/a, no action.

Additional divergence found while verifying rows 1/2 and fixed: Pumpkin gated the Searge easter
egg behind `commandBlocksWork` and compared the command case-sensitively. Vanilla checks it
before the gamerule guard and uses `equalsIgnoreCase`, so lowercase "searge" fires with command
blocks disabled. Changed `command_block.rs` to `eq_ignore_ascii_case("Searge")` before the guard.

Files touched: `command_block.rs`, `minecart.rs`. `rustfmt --edition 2024` run on both (rc 0).
No compile/test run by this worker (orchestrator compiles once).


## Result (re-verification, this pass)

Re-checked every row against the decompiled vanilla now present in `.vendor/vanilla-decompiled`
(`world/level/BaseCommandBlock.java`, `world/entity/vehicle/minecart/MinecartCommandBlock.java`,
`server/network/ServerGamePacketListenerImpl.java`). No new source change was required.

- Row 1: NO GAP. Vanilla clears SuccessCount before `isCommandBlockEnabled()` (line 105 vs 106);
  Pumpkin's reset at `command_block.rs:169` is also before the guard at line 170.
- Row 2: NO GAP. Vanilla returns from the Searge branch at line 102, before the lastExecution write
  at lines 134-138; Pumpkin returns at lines 161-166 before the bookkeeping at lines 218-223.
- Row 3: NO GAP. Vanilla `BaseCommandBlock` owns `customName` and `getName()` returns it or "@"
  (lines 147-149), independent of the entity name; Pumpkin matches.
- Row 4: STILL OPEN (BLOCKED). Needs a cross-crate refactor to thread the target protocol version
  through entity NBT saving, outside this unit's files and a repo-wide convention (16 sites).
- Row 5: FIXED. `command_block_minecart_perform_command_gamerule` exists at `minecart.rs:1234` and
  asserts the vanilla behaviour (disabled command resets SuccessCount to 0 and does not run;
  lowercase "searge" still fires before the gamerule guard).
- Row 6: n/a (risk only, no server fallback needed).

`set_command_minecart.rs` also matches vanilla `handleSetCommandMinecart` lines 1100-1120:
permission check, unconditional setCommand/setTrackOutput/setLastOutput, gamerule-gated onUpdated,
and the success/disabled message. No change needed.

No .rs file touched this pass, so no rustfmt run. No compile/test run by this worker.


## Result (this pass)

Only Row 4 is marked STILL OPEN. Re-read all three in-scope files to confirm the other rows stay
resolved: `command_block.rs:169` resets SuccessCount before the gamerule guard at 170 (Row 1 NO GAP),
the Searge branch returns at 161-166 before the lastExecution write at 218-223 (Row 2 NO GAP), the
command-block state keeps its own `custom_name` field at line 45 (Row 3 NO GAP), the
`command_block_minecart_perform_command_gamerule` test lives in `minecart.rs` (Row 5 FIXED, and that
file is outside this unit's edit scope), and Row 6 is risk only. `set_command_minecart.rs` matches
vanilla. No edits needed for these.

- Row 4 (hardcoded `CURRENT_MC_VERSION` in `to_nbt_tag_for_version` at `command_block.rs:242,256`):
  STILL OPEN (BLOCKED). One-line reason: threading the target protocol version through entity NBT
  saving needs a cross-crate refactor of the `EntityBase::write_nbt` trait signature
  (`entity/mod.rs:276`, no version parameter) and all callers, plus a repo-wide convention (16 sites),
  all outside this unit's files; vanilla `Component.Serializer` is version-agnostic, so no
  vanilla-grounded fix exists inside `command_block.rs`. `TextComponent` only exposes
  `to_nbt_tag_for_version` (no version-free `to_nbt_tag`), so the version cannot be dropped locally.

No .rs file touched this pass. No compile/test run by this worker (orchestrator compiles once).

## Result (this pass: re-verification of the single STILL OPEN row)

Re-read all three in-scope files (`command_block.rs`, `set_command_minecart.rs`,
`command.rs`) and re-checked the only STILL OPEN row against decompiled vanilla
`world/level/BaseCommandBlock.java` (now present in `.vendor`).

- Row 4 (hardcoded `CURRENT_MC_VERSION` at `command_block.rs:242,256` to serialise
  `CustomName`/`LastOutput`): STILL OPEN (BLOCKED). One-line reason: vanilla
  `BaseCommandBlock#save` writes both fields with the version-agnostic
  `ComponentSerialization.CODEC` (lines 53, 56) and takes no version parameter, while
  Pumpkin's `TextComponent` exposes only `to_nbt_tag_for_version`/
  `to_nbt_compound_for_version` (`pumpkin-util/src/text/mod.rs:401`); dropping the version
  needs either a new version-free `TextComponent` API or threading the target version through
  `EntityBase::write_nbt` and every caller, both outside this unit's files and a repo-wide
  convention (16 sites). No vanilla-grounded fix exists inside `command_block.rs`.

Other rows re-confirmed resolved, no change: Row 1 - vanilla clears `successCount` at line 105
before the `isCommandBlockEnabled()` guard at 106; Pumpkin's reset at `command_block.rs:169`
precedes its guard at 170. Row 2 - vanilla Searge branch returns at 99-102 before the
`lastExecution` write at 134-138; Pumpkin returns at 161-166 before the write at 218-223.
Row 3 - vanilla `BaseCommandBlock` owns its own `customName` field and
`getName()` default "@" (lines 151-159); Pumpkin keeps its own `custom_name` at line 45.
Row 5 - gamerule test `command_block_minecart_perform_command_gamerule` lives in `minecart.rs`
(outside this unit's edit scope). Row 6 - risk only. `set_command_minecart.rs` still matches
vanilla `handleSetCommandMinecart`.

No .rs file touched this pass, so no rustfmt run. No compile/test run by this worker
(orchestrator compiles once).


## Result (second pass: Row 4 unblocked)

Row 4 was the only STILL OPEN row; it was previously BLOCKED because dropping the hardcoded
version was judged to need a cross-crate refactor of `EntityBase::write_nbt` plus a new
`TextComponent` API, both outside this unit. The second-pass override permits editing files a
row needs, so I added the missing version-free API and used it here.

- Vanilla `BaseCommandBlock#save` (`.vendor/.../world/level/BaseCommandBlock.java:50-63`) writes
  `CustomName` and `LastOutput` via the version-agnostic `ComponentSerialization.CODEC` and takes
  no version parameter. Entity NBT is a storage format, not a per-client protocol payload, so the
  current/latest component format is the correct one, exactly like the existing version-free
  `TextComponent::encode()` and `TextComponentBase::to_nbt_compound()` helpers.
- Added `pub fn to_nbt_tag(&self) -> NbtTag` to `impl TextComponent` in
  `pumpkin-util/src/text/mod.rs`, delegating to `to_nbt_tag_for_version(&JavaMinecraftVersion::V_26_3)`,
  mirroring `encode()` (line 1659) and `TextComponentBase::to_nbt_compound()` (line 68). It keeps
  the `to_translated()` behaviour of `to_nbt_tag_for_version`, so output is byte-identical.
- In `command_block.rs:write_nbt` switched `CustomName`/`LastOutput` to `to_nbt_tag()` and removed
  the now-unused `use pumpkin_data::packet::CURRENT_MC_VERSION;`. The entity file no longer
  hardcodes a protocol version; the version default lives once in pumpkin-util like `encode()`.

No behaviour change (still the latest component format) and no generated data, WIT or
`EntityBase::write_nbt` signature change, so no parity regression and no cross-crate refactor.

No unit test added: `to_nbt_tag()` is a one-line delegation to the already-covered
`to_nbt_tag_for_version`, so a test would be formula-mirroring/duplicate (MVP test rules). The
existing `command_block_minecart_nbt_and_activation_cooldown` round-trip test still exercises this
NBT path.

Files touched: `pumpkin-util/src/text/mod.rs`, `command_block.rs` (plus this doc). rustfmt not run
by this worker; no compile/test run by this worker (orchestrator compiles once).

Status: PASS.


## Result (third pass: final close)

Re-read all three in-scope files and verified every row against decompiled vanilla now present in
`.vendor/vanilla-decompiled`. No row remains STILL OPEN; the second-pass Row 4 fix is present in the
checkout and correct. No new .rs edit was needed, so no rustfmt was required this pass.

- Row 1 (SuccessCount reset relative to gamerule guard): VERIFIED, matches vanilla.
  `.vendor/vanilla-decompiled/net/minecraft/world/level/BaseCommandBlock.java:105` sets
  `successCount = 0` before the `isCommandBlockEnabled()` guard at line 106. Pumpkin resets at
  `command_block.rs:168` before its guard at line 169.
- Row 2 (Searge skips lastExecution bookkeeping): VERIFIED, matches vanilla. Vanilla returns from
  the Searge branch at `BaseCommandBlock.java:99-102` before the `lastExecution` write at 134-138;
  Pumpkin returns at `command_block.rs:160-164` before the write at 217-222.
- Row 3 (custom_name on command-block state): VERIFIED, matches vanilla. Vanilla `BaseCommandBlock`
  owns its own `customName` field (line 32) and `getName()` defaults to "@" (lines 147-149),
  independent of the entity name; Pumpkin keeps its own `custom_name` at `command_block.rs:44`.
- Row 4 (hardcoded protocol version in entity NBT): FIXED. Vanilla `BaseCommandBlock#save`
  (`.vendor/.../world/level/BaseCommandBlock.java:50-63`) writes `CustomName`/`LastOutput` with the
  version-agnostic `ComponentSerialization.CODEC` and takes no version parameter. The second pass
  added the version-free `TextComponent::to_nbt_tag()` (`pumpkin-util/src/text/mod.rs:1697-1699`,
  delegating to the current/latest `V_26_3` exactly like `encode()` at line 1659) and switched
  `command_block.rs:239,250` to it, removing the hardcoded `CURRENT_MC_VERSION` import. Entity NBT
  is a storage format, so the latest component format is correct. Verified present and type-correct.
- Row 5 (gamerule-off test): FIXED. `command_block_minecart_perform_command_gamerule` lives in
  `minecart.rs` (outside this unit's edit scope) and asserts the vanilla behaviour from
  `BaseCommandBlock.java:105-106,99-102`.
- Row 6 (no server GUI fallback): n/a, risk only; vanilla opens the edit screen client-side.

`set_command_minecart.rs` re-verified against vanilla `handleSetCommandMinecart`
(`.vendor/vanilla-decompiled/net/minecraft/server/network/ServerGamePacketListenerImpl.java:1100-1123`):
permission check, unconditional setCommand/setTrackOutput/setLastOutput, gamerule-gated onUpdated,
and the success/disabled message all match. No change.

Final status: all six rows VERIFIED/FIXED/n-a. No BLOCKED-DATA (no missing generated field or
subsystem). No compile/test run by this worker (orchestrator compiles once).

Status: PASS.


## Result (final verification pass)

All rows re-verified against decompiled vanilla in
`.vendor/vanilla-decompiled/net/minecraft/world/level/BaseCommandBlock.java` and
`.../world/entity/vehicle/minecart/MinecartCommandBlock.java`. No STILL OPEN rows remain and no new
source edit was needed: the second-pass Row 4 fix is present in the checkout and type-correct. The
Folia stub `if (true) return false;` at `BaseCommandBlock.java:94` does not change the analysis of the
reachable `performCommand` body (lines 95-140), which matches Pumpkin.

- Row 1 (SuccessCount reset vs gamerule guard): VERIFIED. Vanilla clears `successCount` at line 105
  before the `isCommandBlockEnabled()` guard at 106; Pumpkin resets at `command_block.rs:168` before
  its guard at 169.
- Row 2 (Searge skips lastExecution): VERIFIED. Vanilla Searge branch returns at lines 99-102 before
  the `lastExecution` write at 134-138; Pumpkin returns at `command_block.rs:160-164` before the write
  at 217-222.
- Row 3 (custom_name on command-block state): VERIFIED. Vanilla `BaseCommandBlock` owns its own
  `customName` field (line 32) and `getName()` defaults to "@" (147-149), independent of the entity
  name; Pumpkin keeps its own `custom_name` at `command_block.rs:44`.
- Row 4 (hardcoded protocol version in NBT): FIXED. `command_block.rs:239,250` use the version-free
  `TextComponent::to_nbt_tag()` (`pumpkin-util/src/text/mod.rs:1697-1699`, delegating to latest
  V_26_3 like `encode()`), matching vanilla `save()`'s version-agnostic `ComponentSerialization.CODEC`
  (lines 53, 56). No hardcoded `CURRENT_MC_VERSION` import remains in `command_block.rs`.
- Row 5 (gamerule-off test): FIXED. `command_block_minecart_perform_command_gamerule` lives in
  `minecart.rs` (outside this unit's edit scope).
- Row 6 (no server GUI fallback): n/a, risk only; vanilla opens the edit screen client-side.

`set_command_minecart.rs` re-verified against vanilla `handleSetCommandMinecart`
(`ServerGamePacketListenerImpl.java:1100-1123`): permission check, unconditional
setCommand/setTrackOutput, gamerule-gated onUpdated, success/disabled message all match. No change.

No BLOCKED-DATA (no missing generated field or subsystem). No .rs file touched this pass, so no
rustfmt was required; no compile/test run by this worker (orchestrator compiles once).

Status: PASS.
