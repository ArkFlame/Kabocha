# Final rows from docs/tracker/tentities-command-block-minecart.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files:

All three STILL OPEN rows re-verified against decompiled vanilla under `.vendor/vanilla-decompiled/net/minecraft/` (2026-11-18).

## Result

Terminal dispositions (FINAL PASS, 2026-11-18). Every carried-over STILL OPEN row (source doc lines 43, 47, 50 = Gaps 4, 5, 6) is now closed. No `.rs` source needed editing: the in-scope code already matches decompiled vanilla, so rustfmt was not required. Gaps 1-3 of the source doc were already fixed/closed in earlier passes and remain so.

- Gap 4 (entity display name, source doc lines 41-45): VERIFIED (matches vanilla), no code change; premise false.
  `.vendor/vanilla-decompiled/net/minecraft/world/level/BaseCommandBlock.java:32,147-149,50-63` owns its own
  `customName`, `getName()` returns it or the `Component.literal("@")` default, and NBT stores it under
  `CustomName`. `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/minecart/MinecartCommandBlock.java:32`
  embeds that `BaseCommandBlock` and persists it via `commandBlock.save(output)` (line 68). Pumpkin's
  `crates/pumpkin-core/src/entity/vehicle/minecart/command_block.rs:44,112-118,233-240,268-272`
  `CommandBlockMinecart.custom_name` plus `get_name()` defaulting to `@` mirror vanilla; dropping the copy
  would diverge, so no change was made.

- Gap 5 (NBT component version, source doc lines 46-49): VERIFIED (matches vanilla), no code change.
  `.vendor/vanilla-decompiled/net/minecraft/world/level/BaseCommandBlock.java:50-63` serializes `CustomName`
  and `LastOutput` through the version-independent `ComponentSerialization.CODEC`. Pumpkin's
  `command_block.rs:230-251` `write_nbt` uses `TextComponent::to_nbt_tag()`, which selects
  `JavaMinecraftVersion::V_26_3` in `crates/pumpkin-util/src/text/mod.rs:1695-1699`. That version is exactly
  the single served protocol: `crates/pumpkin-data/src/generated/packet.rs:3-6` defines
  `CURRENT_MC_VERSION == LOWEST_SUPPORTED_MC_VERSION == JavaMinecraftVersion::V_26_3`. Since Pumpkin natively
  serves only that version (older clients are handled by plugins), the emitted component NBT matches vanilla
  for the served protocol. The original "hardcoded V_26_3 at command_block.rs:245-263" concern is also gone:
  the entity no longer passes a version at all. No generated `pumpkin-data` field is missing, and threading a
  target version through Entity NBT saving is unnecessary for parity (and would be a cross-crate refactor),
  so this row is VERIFIED rather than BLOCKED-DATA.

- Gap 6 (gamerule-off test, source doc lines 50-53): VERIFIED (matches vanilla), no new test needed.
  `crates/pumpkin-core/src/entity/vehicle/minecart.rs` `tests::command_block_minecart_perform_command_gamerule`
  sets `command_blocks_work = false`, asserts `success_count == 0` and empty output for a normal command, then
  asserts the case-insensitive Searge path still fires. This matches
  `.vendor/vanilla-decompiled/net/minecraft/world/level/BaseCommandBlock.java:95-105,134-140`: game-time early
  return, `equalsIgnoreCase` Searge check before the gamerule guard with no `lastExecution` update on that
  path, `successCount` reset before the guard, `lastExecution` updated at the end.
  `command_block.rs::perform_command` (lines 149-224) matches this ordering, and
  `crates/pumpkin-core/src/net/java/play/set_command_minecart.rs:34-39` gates `on_updated` behind the gamerule,
  matching `.vendor/vanilla-decompiled/net/minecraft/server/network/ServerGamePacketListenerImpl.java:1092-1120`.

All rows have terminal dispositions: no STILL OPEN rows remain.
