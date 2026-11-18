# Final rows from docs/tracker/tentities-command-block-minecart-5e81b3.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- Row 4 (hardcoded `JavaMinecraftVersion::V_26_3`): STILL OPEN (BLOCKED). Needs a cross-crate
- Row 4: STILL OPEN (BLOCKED). Needs a cross-crate refactor to thread the target protocol version
Only Row 4 is marked STILL OPEN. Re-read all three in-scope files to confirm the other rows stay
STILL OPEN (BLOCKED). One-line reason: threading the target protocol version through entity NBT

## Result

All rows independently re-verified against decompiled vanilla present in
`.vendor/vanilla-decompiled/net/minecraft/`. No row remains STILL OPEN; the earlier Row 4 fix is
present and type-correct in the checkout, so no new .rs edit was needed this pass.

- Row 1 (success_count reset relative to gamerule guard): VERIFIED, matches vanilla.
  `.vendor/vanilla-decompiled/net/minecraft/world/level/BaseCommandBlock.java:105` sets
  `successCount = 0` before the `isCommandBlockEnabled()` guard at line 106; Pumpkin resets at
  `command_block.rs:168` before its guard at `command_block.rs:169`.
- Row 2 (Searge skips lastExecution bookkeeping): VERIFIED, matches vanilla. Vanilla returns from
  the Searge branch at `BaseCommandBlock.java:99-102`, before the `lastExecution` write at lines
  134-138; Pumpkin returns at `command_block.rs:160-165` before the bookkeeping at lines 217-222.
- Row 3 (custom_name on command-block state): VERIFIED, matches vanilla. Vanilla `BaseCommandBlock`
  owns its own `customName` field (line 32) and `getName()` defaults to "@" (lines 147-149),
  independent of the entity name; Pumpkin keeps its own `custom_name` at `command_block.rs:44`.
- Row 4 (hardcoded protocol version in entity NBT): FIXED. Vanilla `BaseCommandBlock#save`
  (`.vendor/vanilla-decompiled/net/minecraft/world/level/BaseCommandBlock.java:50-63`) writes
  `CustomName`/`LastOutput` with the version-agnostic `ComponentSerialization.CODEC` (lines 53, 56)
  and takes no version parameter. The version-free `TextComponent::to_nbt_tag()` lives in
  `pumpkin-util/src/text/mod.rs:1695-1699` (delegating to the latest V_26_3 exactly like `encode()`
  at line 1659), and `command_block.rs:239,250` now use it; no hardcoded `CURRENT_MC_VERSION`
  import remains in `command_block.rs`. Entity NBT is a storage format, not a per-client protocol
  payload, so the latest component format is correct. No cross-crate `EntityBase::write_nbt`
  signature change or generated data was needed.
- Row 5 (gamerule-off test): FIXED. `command_block_minecart_perform_command_gamerule`
  (`minecart.rs:1350`) asserts the vanilla behaviour from `BaseCommandBlock.java:99-102,105-106`.
- Row 6 (no server GUI fallback): n/a, risk only; vanilla opens the edit screen client-side.

`set_command_minecart.rs` re-verified against `handleSetCommandMinecart`
(`.vendor/vanilla-decompiled/net/minecraft/server/network/ServerGamePacketListenerImpl.java`):
permission check, unconditional setCommand/setTrackOutput, clear LastOutput when trackOutput false,
gamerule-gated onUpdated, and the success/disabled message all match. No change.

No BLOCKED-DATA (no missing generated field or subsystem). No .rs file touched this pass, so no
rustfmt was required; no compile/test run by this worker (orchestrator compiles once).

Status: PASS.
