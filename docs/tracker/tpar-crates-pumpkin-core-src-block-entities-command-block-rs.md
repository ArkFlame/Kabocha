# Parity rows for crates/pumpkin-core/src/block/entities/command_block.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-co-58043b** [partial/OWN-OPEN] own audit: 2 open, 11 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-co-58043b.md) docs/tracker/taudit-crates-pumpkin-core-src-co-58043b.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)


## Result (third pass, CODER, VERIFIED)

Re-read the in-unit source (`block/entities/command_block.rs`), the cross-file consumers
(`command/mod.rs`, `net/java/play/set_command_block.rs`,
`entity/vehicle/minecart/command_block.rs`) and decompiled vanilla. The single tracked row
is the audit doc `taudit-crates-pumpkin-core-src-co-58043b.md`, whose six sub-rows all end
FIXED / VERIFIED. No source edits were needed; every fix is already present and type-consistent.

- own/taudit-crates-pumpkin-core-src-co-58043b: VERIFIED FIXED. All six sub-rows closed:
  - Row 1 (custom name): `CommandBlockEntity` holds `custom_name: StdMutex<Option<TextComponent>>`;
    `write_sync_nbt`/`from_nbt` read/write `CustomName` via `CustomNameImpl` (String/Compound codec),
    `net/java/play/set_command_block.rs` initialises it, and `command/mod.rs#command_block_name`
    returns the custom name with an `@` fallback used by `into_source` and `Display`. Matches
    `.vendor/vanilla-decompiled/net/minecraft/world/level/BaseCommandBlock.java#getName/save/load`
    (`getName` returns `customName` or `DEFAULT_NAME`, `save`/`load` use `ComponentSerialization.CODEC`)
    and `.../world/level/block/entity/CommandBlockEntity.java#createCommandSourceStack`.
  - Row 2 (`LastOutput` text component): `last_output: StdMutex<Option<TextComponent>>` written via
    `to_nbt_tag_for_version(V_26_3)` only when TrackOutput is set, read via `TextComponent::from_nbt`;
    `command/mod.rs#send_message` prepends the `[HH:mm:ss] ` child using local time. Matches
    `.../world/level/BaseCommandBlock.java#save/load/sendSystemMessage`
    (`Component.literal("[" + TIME_FORMAT + "] ").append(message)`, `ComponentSerialization.CODEC`).
    Round-trip unit tests exist in `command_block.rs`.
  - Row 3 (player source text name): `command/mod.rs#player_source` passes `player.get_name().get_text()`
    as textName and `get_display_name()` as display name, matching vanilla
    `.../server/level/ServerPlayer.java#createCommandSourceStack`.
  - Row 4 (console source position): VERIFIED, no change. This vanilla version uses the respawn
    spawn point (`.../server/MinecraftServer.java`, `Vec3.atLowerCornerOf(getRespawnData().pos())`),
    which Pumpkin's `get_world_and_spawn_point` already matches.
  - Row 5 (`world_or_first`): FIXED in `command/mod.rs`, now prefers
    `get_world_from_dimension(&Dimension::OVERWORLD)` before `worlds.load().first()`.
  - Row 6 (command block minecart): VERIFIED, subsystem implemented in
    `entity/vehicle/minecart/command_block.rs`, matching vanilla
    `.../world/entity/vehicle/minecart/MinecartCommandBlock.java#createCommandSourceStack`.

No generated pumpkin-data fields or whole missing subsystems block any row. All seams already
covered by existing round-trip unit tests. Compilation not run (orchestrator compiles once).
