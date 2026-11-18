# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-block-entities-command-block-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

All rows closed. No source edits were needed this final pass: every fix is already present in the checkout and re-verified against decompiled vanilla. Compilation not run (worker shell denied; orchestrator compiles once).

## Result

The single tracked item is the audit `own/taudit-crates-pumpkin-core-src-co-58043b`
(`docs/tracker/taudit-crates-pumpkin-core-src-co-58043b.md`), whose six sub-rows all end
FIXED / VERIFIED. No generated pumpkin-data fields or whole missing subsystems block any
row (so no BLOCKED-DATA). No new unit-test seam (all seams covered by the existing
round-trip tests in `block/entities/command_block.rs` and `command/mod.rs`).

- Row 1 (custom name): VERIFIED FIXED. `CommandBlockEntity` holds `custom_name:
  StdMutex<Option<TextComponent>>`; `write_sync_nbt`/`from_nbt` read/write `CustomName`
  via `CustomNameImpl` (String/Compound codec == vanilla `ComponentSerialization.CODEC`);
  `net/java/play/set_command_block.rs` initialises it from the old entity; and
  `command/mod.rs#command_block_name` returns the custom name with an `@` fallback, used
  by both `into_source` (text name + display name) and `Display`. Matches
  `.vendor/vanilla-decompiled/net/minecraft/world/level/BaseCommandBlock.java#getName/save/load`
  and `.../world/level/block/entity/CommandBlockEntity.java#createCommandSourceStack`.
- Row 2 (`LastOutput` as text-component JSON): VERIFIED FIXED. `last_output` is
  `StdMutex<Option<TextComponent>>`, written via `to_nbt_tag_for_version(V_26_3)` only when
  TrackOutput is set, read via `TextComponent::from_nbt`; `command/mod.rs#send_message`
  prepends the `[HH:mm:ss] ` child using local time. Matches
  `.vendor/vanilla-decompiled/net/minecraft/world/level/BaseCommandBlock.java#save/load/sendSystemMessage`
  and `.../network/chat/ComponentSerialization.java#createCodec`. Round-trip unit tests
  exist in `command_block.rs`.
- Row 3 (player source text name): VERIFIED FIXED. `command/mod.rs#player_source` passes
  `player.get_name().get_text()` as textName and `get_display_name()` as display name,
  matching `.vendor/vanilla-decompiled/net/minecraft/server/level/ServerPlayer.java#createCommandSourceStack`.
- Row 4 (console source position): VERIFIED, no change needed. This vanilla version uses
  the respawn spawn point (`.vendor/vanilla-decompiled/net/minecraft/server/MinecraftServer.java:2595-2599`,
  `Vec3.atLowerCornerOf(getRespawnData().pos())`), which Pumpkin's
  `get_world_and_spawn_point` already matches.
- Row 5 (`world_or_first`): VERIFIED FIXED. `command/mod.rs#world_or_first` prefers
  `server.get_world_from_dimension(&Dimension::OVERWORLD)` before
  `server.worlds.load().first()`.
- Row 6 (command block minecart source): VERIFIED FIXED. Implemented in
  `crates/pumpkin-core/src/entity/vehicle/minecart/command_block.rs` and `minecart.rs`,
  matching `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/minecart/MinecartCommandBlock.java#createCommandSourceStack`.
