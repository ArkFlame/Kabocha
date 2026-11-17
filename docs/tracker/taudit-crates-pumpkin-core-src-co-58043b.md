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
