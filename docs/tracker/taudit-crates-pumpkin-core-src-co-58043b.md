# Audit: crates/pumpkin-core/src/command/mod.rs

Coverage: AUDIT. Fixed: command-block yaw for vertical facings (Up/Down were 270, vanilla `CommandBlockEntity` falls through to `Vec2.ZERO`). Remaining, ordered by value.

## 1. Command block custom name ignored (M)
`crates/pumpkin-core/src/command/mod.rs:378` (`// TODO: when command blocks get custom names`)
Vanilla: `CommandBlockEntity#getName()` returns `getCustomName()` when present, else `Component.literal("@")`; the source name is `this.getName().getString()` (`CommandBlockEntity#createCommandSourceStack`).
Gap: `CommandBlockEntity` has no `CustomName` field/NBT at all (`crates/pumpkin-core/src/block/entities/command_block.rs:11` `// todo: CustomName, LastExecution, UpdateLastExecution`), so a `/data merge` CustomName is dropped on save and never used as the source name.
Fix: add `custom_name: StdMutex<Option<TextComponent>>`, read/write `CustomName` in `write_sync_nbt`/`from_nbt` (JSON via `TextComponent` serde, like other block entities), pass it to `build_command_block_actor`, and use it in `into_source`. Then drop the TODO.

## 2. `LastOutput` is plain text, not a text-component JSON (M)
`crates/pumpkin-core/src/command/mod.rs:151-164`
Vanilla stores the last output as a `Component` and serialises it to the `LastOutput` string NBT as JSON (`Component.Serializer`), with the `[HH:mm:ss]` prefix added around the component; 1.20.5+ clients parse that string as JSON.
Gap: we write `"[HH:MM:SS] <plain text>"`, which is not valid JSON, so the client GUI output and any `/data get block ... LastOutput` consumer see a malformed component.
Fix: keep a `TextComponent` (not `String`) in `CommandBlockEntity::last_output`, render with `to_json`/serde for NBT and `get_text()` for the plain fallback; derive the timestamp with the server's local zone rather than `now_utc()`.

## 3. Player source text name uses the display name (S)
`crates/pumpkin-core/src/command/mod.rs:360`
Vanilla `ServerPlayer#createCommandSourceStack()` passes `this.getName().getString()` as `textName` and `getDisplayName()` as `displayName`. We use `get_display_name().get_text()` for both, so selectors/`/say` style text that read the raw name pick up team prefix/suffix formatting once teams exist.
Fix: pass the plain profile name for `textName`.

## 4. Console source position is world spawn, vanilla uses the origin (S, verify)
`crates/pumpkin-core/src/command/mod.rs:339-349`, `408-417`
`MinecraftServer#createCommandSourceStack()` uses `new CommandSourceStack(CommandSource.NULL, Vec3.ZERO, Vec2.ZERO, overworld(), 4, ...)`. We put console/dummy at the overworld spawn block, so `~ ~ ~` from console resolves at spawn instead of 0,0,0.
Needs a decompiled check before changing (could be a deliberate Pumpkin choice); low user impact.

## 5. `world_or_first` falls back to "first world", not the overworld (S)
`crates/pumpkin-core/src/command/mod.rs:283-286`
Doc comment says "first (overworld) world" but `server.worlds.load().first()` is insertion order. It is currently unused, so no live divergence; if it is wired up, prefer `server.get_world_from_dimension(&Dimension::OVERWORLD)` and only then `first()`.

## 6. Command block mincart sources (bolt-on)
`crates/pumpkin-core/src/command/mod.rs:120-124` comment mentions command block minecarts, but there is no sender variant; `MinecartCommandBlock` (`BaseCommandBlock`) is not implemented. Tracked with the minecart work, not this file.

## Gap status
- 1. Command block custom name ignored (M): FIXED. In-unit `custom_name` field + CustomName NBT + actor wiring (command_block.rs, set_command_block.rs); the formerly out-of-unit `into_source` now uses `command_block_name()` (custom name, `@` fallback) in command/mod.rs.
- 2. `LastOutput` plain text not text-component JSON (M): FIXED. `last_output` is `Option<TextComponent>` written via `to_nbt_tag_for_version(V_26_3)`; all consumers updated.
- 3. Player source text name (command/mod.rs): FIXED. `player_source` passes the plain profile name for textName.

## Result

- Row 1 (custom name): PARTIAL fixed in this unit. Added `custom_name:
  StdMutex<Option<TextComponent>>` to `CommandBlockEntity`, initialised it in
  `new`/`from_nbt`, read/wrote the `CustomName` NBT key via `CustomNameImpl`
  (compound text-component, matching vanilla `ComponentSerialization.CODEC`),
  passed it into `build_command_block_actor` instead of `None`, dropped the
  CustomName todo, and added a round-trip unit test. Verified against
  `BaseCommandBlock.java#getName/save/load` and
  `CommandBlockEntity.java#createCommandSourceStack`. Remaining outside this
  unit: `command/mod.rs#into_source` must use the custom name as the source
  name instead of literal `@`, and the `CommandBlockEntity { .. }` literal in
  `net/java/play/set_command_block.rs` must initialise the new field or the
  crate will not compile.
- Row 2 (`LastOutput` as text-component JSON): OPEN. Changing `last_output`
  from `String` to `TextComponent` is a cross-crate refactor that also requires
  `command/mod.rs`, `block/blocks/command.rs`, and both
  `pumpkin-wasm-host-v0_1/v0_2/src/block_entity.rs` (all outside this unit).
  Verified with `BaseCommandBlock.java#save/load/sendSystemMessage`.
- Row 3 (player source text name): OPEN. Entirely in `command/mod.rs#player_source`,
  a file outside this unit. Verified against vanilla
  `ServerPlayer#createCommandSourceStack`.


## Result (final verification, CODER)

All in-unit code for the STILL OPEN rows is already present and re-verified against decompiled vanilla; no source edits were needed in this pass.

- Row 1 (custom name): IN-UNIT WORK COMPLETE (previously applied, re-verified). `CommandBlockEntity` has `custom_name: StdMutex<Option<TextComponent>>`; `write_sync_nbt`/`from_nbt` read/write `CustomName` via `CustomNameImpl` (equivalent to vanilla `ComponentSerialization.CODEC`); `net/java/play/set_command_block.rs` initialises it from the old entity; `bedrock_block_actor_data` passes it to `build_command_block_actor`. Verified against `BaseCommandBlock.java#save/load/getName`, `CommandBlockEntity.java#createCommandSourceStack`, and `ComponentSerialization.java#createCodec`. OUTSIDE this unit (left open): `command/mod.rs#into_source` still builds `TextComponent::text("@")` (lines 388-389 TODO) instead of the entity custom name with `@` fallback, so the source display name is wrong when CustomName is set. `mod.rs` is not in this unit's allowed basenames.
- Row 2 (LastOutput text component): FIXED and verified. `last_output` is `StdMutex<Option<TextComponent>>`; NBT uses `to_nbt_tag_for_version(V_26_3)`, which matches vanilla `ComponentSerialization.CODEC` (bare String NBT for collapsible plain text, Compound otherwise) per `ComponentSerialization.java#createCodec` `tryCollapseToString` branch. `command/mod.rs#send_message` appends a `[HH:mm:ss] ` child using `now_local()` with `now_utc()` fallback, matching `BaseCommandBlock.java#sendSystemMessage`. `block/blocks/command.rs` and both `pumpkin-wasm-host-v0_1/v0_2/src/block_entity.rs` already consume the new type; round-trip unit tests exist in `command_block.rs`.
- Row 3 (player source text name): FIXED and verified (previously applied, outside this unit, no action needed). `command/mod.rs#player_source` passes `player.get_name().get_text()` for textName and `get_display_name()` for the display name; `Entity#get_name` returns the plain profile name, matching vanilla `ServerPlayer#createCommandSourceStack`.


## Result (post-restart re-verification, CODER)

Re-read all four in-unit files and decompiled vanilla `BaseCommandBlock.java` / `CommandBlockEntity.java`; no source edits were needed, all fixes already present and correct.

- Row 1 (custom name): IN-UNIT COMPLETE. `block/entities/command_block.rs` holds `custom_name: StdMutex<Option<TextComponent>>`, saves/loads `CustomName` via `CustomNameImpl` (String/Compound codec, matching `ComponentSerialization.CODEC`), and feeds it to `build_command_block_actor`; `net/java/play/set_command_block.rs` initialises it from the old entity; the original todo is gone. LEFT OPEN (outside unit): `command/mod.rs#into_source` still uses literal `@` (lines 388-389 TODO) instead of the entity custom name; `mod.rs` is not in this unit's allowed basenames.
- Row 2 (LastOutput text component): FIXED. `last_output` is `StdMutex<Option<TextComponent>>` written via `to_nbt_tag_for_version(V_26_3)` (== `CURRENT_MC_VERSION`, bare String for collapsible text, Compound otherwise) and read via `TextComponent::from_nbt`; consumers in `block/blocks/command.rs` and both `pumpkin-wasm-host-v0_1/v0_2/src/block_entity.rs` use the new type; `command/mod.rs#send_message` builds the literal `[HH:mm:ss] ` prefix plus message child, matching `BaseCommandBlock#sendSystemMessage`. Round-trip unit tests exist.
- Row 3 (player source text name): FIXED outside this unit (`command/mod.rs#player_source`), no action needed.

Compilation not run (worker shell denied; orchestrator compiles once).


## Result (final CODER pass, all rows verified FIXED)

Re-verified all three rows against decompiled vanilla (`BaseCommandBlock.java#getName/save/load/sendSystemMessage`, `CommandBlockEntity.java#createCommandSourceStack`) and against the current source. No source edits were needed; every row is fully fixed in the checkout.

- Row 1 (custom name): FIXED. In unit: `block/entities/command_block.rs` holds `custom_name: StdMutex<Option<TextComponent>>`, saves/loads `CustomName` via `CustomNameImpl` (String/Compound codec == vanilla `ComponentSerialization.CODEC`), and feeds it to `build_command_block_actor`; `net/java/play/set_command_block.rs` initialises it from the old entity. Previously out-of-unit: `command/mod.rs#command_block_name` now returns the custom name with `@` fallback, used by both `into_source` (line 394) and `Display`, matching vanilla `BaseCommandBlock#getName` (`this.getName().getString()` as source name).
- Row 2 (`LastOutput` text component): FIXED. `last_output` is `StdMutex<Option<TextComponent>>` written via `to_nbt_tag_for_version(V_26_3)` (== `CURRENT_MC_VERSION`: bare String for collapsible plain text, Compound otherwise) and only when TrackOutput is set, read via `TextComponent::from_nbt`; matches vanilla `BaseCommandBlock#save/load`. `command/mod.rs#send_message` appends the `[HH:mm:ss] ` prefix child using local time (`now_local`, `now_utc` fallback), matching `sendSystemMessage`. Consumers in `block/blocks/command.rs` and both `pumpkin-wasm-host-v0_1/v0_2/src/block_entity.rs` use the new type. Round-trip unit tests exist in `command_block.rs`.
- Row 3 (player source text name): FIXED. `command/mod.rs#player_source` passes `player.get_name().get_text()` for textName and `get_display_name()` for the display name, matching vanilla `ServerPlayer#createCommandSourceStack`.

No remaining in-scope STILL OPEN rows; no new unit-test seam (all seams already covered by existing round-trip tests). Compilation not run (worker shell denied; orchestrator compiles once).


## Result (third pass VERIFIED, CODER)

Re-read all four in-unit files plus `command/mod.rs`, `net/java/play/set_command_block.rs`,
and decompiled vanilla. All three STILL OPEN rows are fully fixed; no source edits needed.

- Row 1 (custom name): VERIFIED FIXED. `CommandBlockEntity` holds
  `custom_name: StdMutex<Option<TextComponent>>`; `write_sync_nbt`/`from_nbt` read/write
  `CustomName` via `CustomNameImpl` (String/Compound codec), and
  `net/java/play/set_command_block.rs` initialises it from the old entity. `command/mod.rs#command_block_name`
  returns the custom name with an `@` fallback and is used by both `into_source` (text name + display
  name) and `Display`. Matches `.vendor/vanilla-decompiled/net/minecraft/world/level/BaseCommandBlock.java`
  (`getName`, `save`, `load`) and `.../world/level/block/entity/CommandBlockEntity.java`
  (`createCommandSourceStack` uses `this.getName()`).
- Row 2 (`LastOutput` text component): VERIFIED FIXED. `last_output` is
  `StdMutex<Option<TextComponent>>`, written via `to_nbt_tag_for_version(V_26_3)` only when TrackOutput
  is set, read via `TextComponent::from_nbt`; `command/mod.rs#send_message` appends the `[HH:mm:ss] `
  child with local time (`now_local`, `now_utc` fallback). Matches
  `.../world/level/BaseCommandBlock.java#save/load/sendSystemMessage` (`ComponentSerialization.CODEC`,
  `Component.literal("[" + TIME_FORMAT + "] ").append(message)`). Round-trip unit tests exist in
  `command_block.rs`.
- Row 3 (player source text name): VERIFIED FIXED. `command/mod.rs#player_source` passes
  `player.get_name().get_text()` as textName and `get_display_name()` as display name, matching
  vanilla `ServerPlayer#createCommandSourceStack` (`.vendor/.../server/level/ServerPlayer.java`).

No remaining STILL OPEN rows; no new unit-test seam (all seams covered by existing round-trip tests).
No source edits this pass; rustfmt not needed. Compilation not run (worker shell denied; orchestrator compiles once).


## Result (third pass, final, CODER)

All six rows end FIXED / VERIFIED / BLOCKED-DATA; none remain STILL OPEN. Source edits this pass were limited to `command/mod.rs#world_or_first` (row 5).

- Row 1 (custom name): VERIFIED FIXED. `CommandBlockEntity` holds `custom_name: StdMutex<Option<TextComponent>>`; `write_sync_nbt`/`from_nbt` read/write `CustomName` via `CustomNameImpl` (String/Compound codec == `ComponentSerialization.CODEC`); `net/java/play/set_command_block.rs` initialises it from the old entity; `command/mod.rs#command_block_name` returns the custom name with an `@` fallback and feeds both text name and display name. Matches `.vendor/vanilla-decompiled/net/minecraft/world/level/BaseCommandBlock.java#getName/save/load` and `.../world/level/block/entity/CommandBlockEntity.java#createCommandSourceStack`.
- Row 2 (`LastOutput` text component): VERIFIED FIXED. `last_output` is `StdMutex<Option<TextComponent>>` written via `to_nbt_tag_for_version(V_26_3)` only when TrackOutput is set, read via `TextComponent::from_nbt`; `command/mod.rs#send_message` prepends the `[HH:mm:ss] ` child using local time. Matches `.vendor/.../world/level/BaseCommandBlock.java#save/load/sendSystemMessage`.
- Row 3 (player source text name): VERIFIED FIXED. `command/mod.rs#player_source` passes `player.get_name().get_text()` as textName and `get_display_name()` as display name, matching `.vendor/.../server/level/ServerPlayer.java#createCommandSourceStack`.
- Row 4 (console source position): VERIFIED, no change needed. The doc's premise (`Vec3.ZERO`) does not hold in this vanilla version: `.vendor/vanilla-decompiled/net/minecraft/server/MinecraftServer.java:2595-2599` builds the console source with `Vec3.atLowerCornerOf(this.getRespawnData().pos())` and `Vec2.ZERO` (world spawn, zero rotation). Pumpkin's `get_world_and_spawn_point` already uses the world spawn point with `Vector2::new(0.0, 0.0)`, so it matches. Only nuance: vanilla picks the respawn dimension via `findRespawnDimension()` (custom `respawnDimension`); Pumpkin uses the overworld. Low-impact edge case, left as-is.
- Row 5 (`world_or_first`): FIXED this pass in `command/mod.rs`. It now prefers `server.get_world_from_dimension(&Dimension::OVERWORLD)` before falling back to `server.worlds.load().first()`. The helper is currently unused (no caller besides the `world()` comment), so this is a correctness improvement for when it is wired up; it aligns with vanilla console/dummy senders operating on the overworld.
- Row 6 (command block minecart source): VERIFIED FIXED. The subsystem is implemented in `crates/pumpkin-core/src/entity/vehicle/minecart/command_block.rs` (and `minecart.rs`): `CommandBlockMinecart` has CustomName/LastOutput NBT, builds a `CommandSender::CommandBlock` via a proxy `CommandBlockEntity` (or `Dummy` when TrackOutput is off), and constructs a `CommandSource` with the minecart position, rotation, custom name, and the minecart entity as source. Matches `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/minecart/MinecartCommandBlock.java:127-137` (`createCommandSourceStack`: source, position, rotation, level, name via `BaseCommandBlock`). No source edit needed in this unit.

No generated pumpkin-data fields or whole missing subsystems block any row. Rustfmt not needed (edits are a method body + doc comments); compilation not run (worker shell denied; orchestrator compiles once).


## Result (final pass, all rows verified FIXED, CODER)

Re-read all four in-unit files (`block/entities/command_block.rs`,
`command/mod.rs`, `net/java/play/set_command_block.rs`,
`block/blocks/command.rs`) plus decompiled vanilla. All six rows already end
FIXED / VERIFIED / BLOCKED-DATA; none remain STILL OPEN. No source edits were
needed this pass. `STILL OPEN until closed` does not appear in the doc.

- Row 1 (custom name): VERIFIED FIXED. `CommandBlockEntity` holds
  `custom_name: StdMutex<Option<TextComponent>>`; `write_sync_nbt`/`from_nbt`
  read/write `CustomName` via `CustomNameImpl` (String/Compound codec),
  `net/java/play/set_command_block.rs` initialises it from the old entity, and
  `command/mod.rs#command_block_name` returns the custom name with an `@`
  fallback and feeds both text name and display name. Matches
  `.vendor/vanilla-decompiled/net/minecraft/world/level/BaseCommandBlock.java`
  (`getName` L147-149, `save` L53, `load` L68) and
  `.../world/level/block/entity/CommandBlockEntity.java#createCommandSourceStack`
  L55-65 (`this.getName()`).
- Row 2 (`LastOutput` text component): VERIFIED FIXED. `last_output` is
  `StdMutex<Option<TextComponent>>` written via
  `to_nbt_tag_for_version(V_26_3)` only when TrackOutput is set, read via
  `TextComponent::from_nbt`; `command/mod.rs#send_message` prepends the
  `[HH:mm:ss] ` child using local time. Matches
  `.vendor/vanilla-decompiled/net/minecraft/world/level/BaseCommandBlock.java`
  (`save` L50-63, `load` L65-82,
  `sendSystemMessage` L215-221: `Component.literal("[" + TIME_FORMAT.format + "] ").append(message)`)
  and `.../network/chat/ComponentSerialization.java#createCodec` L138-147
  (`tryCollapseToString` yields a bare String NBT for collapsible text, else a
  Compound). Round-trip unit tests exist in `command_block.rs`.
- Row 3 (player source text name): VERIFIED FIXED. `command/mod.rs#player_source`
  passes `player.get_name().get_text()` as textName and `get_display_name()`
  as display name. Matches
  `.vendor/vanilla-decompiled/net/minecraft/server/level/ServerPlayer.java#createCommandSourceStack`
  L3206-3208 and `.../commands/CommandSourceStack.java` (`NamesProvider.FOR_ENTITY`
  L652-662: `textName` = `entity.getPlainTextName()`, `displayName` =
  `entity.getDisplayName()`).
- Row 4 (console source position): VERIFIED, no change needed. The doc premise
  (`Vec3.ZERO`) does not hold in this vanilla version:
  `.vendor/vanilla-decompiled/net/minecraft/server/MinecraftServer.java#createCommandSourceStack`
  L2595-2599 uses `Vec3.atLowerCornerOf(this.getRespawnData().pos())` (world
  spawn) and `Vec2.ZERO`. Pumpkin's `get_world_and_spawn_point` already uses
  the world spawn with `Vector2::new(0.0, 0.0)`, so it matches. Only nuance:
  vanilla picks the respawn dimension via `findRespawnDimension()`; Pumpkin
  uses the overworld. Low-impact edge case, left as-is.
- Row 5 (`world_or_first`): VERIFIED FIXED. `command/mod.rs#world_or_first`
  prefers `server.get_world_from_dimension(&Dimension::OVERWORLD)` before
  `server.worlds.load().first()`. Helper currently has no live caller, so this
  is a correctness improvement for when it is wired up; aligns with vanilla
  console/dummy senders operating on the overworld
  (`MinecraftServer.java#findRespawnDimension` L2602-2606).
- Row 6 (command block minecart source): VERIFIED FIXED. Implemented in
  `crates/pumpkin-core/src/entity/vehicle/minecart/command_block.rs` and
  `minecart.rs`: `CommandBlockMinecart` holds CustomName/LastOutput NBT,
  builds a `CommandSender::CommandBlock` via a proxy `CommandBlockEntity`
  (or `Dummy` when TrackOutput off), and constructs a `CommandSource` with
  minecart position/rotation/name and the minecart as source. Matches
  `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/minecart/MinecartCommandBlock.java#createCommandSourceStack`
  L127-137. No source edit needed in this unit.

No generated pumpkin-data fields or whole missing subsystems block any row
(so no BLOCKED-DATA). No new unit-test seam (all seams covered by existing
round-trip tests in `command_block.rs`). No source edits this pass; rustfmt not
needed; compilation not run (worker shell denied; orchestrator compiles once).
