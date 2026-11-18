# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-command-commands-datapack-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Rows 1-7 verified against the 26.3 vanilla sources; row 8 is a whole missing subsystem.

## Result

Final pass. All eight rows from the linked `tpar` / `taudit` trackers are closed below against
`.vendor/vanilla-decompiled/net/minecraft/` (26.3, protocol 777). Every in-scope source fix is
already present in the checkout and verified against vanilla; no source edits were needed this pass.
The unmodded 26.2 `DataPackCommand` mache source was used only to confirm the anchor error type and
the `Inserter` order (the `.vendor` copy is Canvas-stripped for register and the Inserter).

- 1. `pack_format` hardcode (`datapack.rs::pack_mcmeta_content`): FIXED / VERIFIED.
  `datapack.rs` calls `CURRENT_MC_VERSION.data_pack_format()`; `pumpkin-util/src/version.rs::data_pack_format()`
  returns 121 for `V_26_3` and writes integer `min_format`/`max_format` once the format exceeds
  `LAST_PRE_MINOR_DATA_PACK_FORMAT` (81). Vanilla
  `.vendor/vanilla-decompiled/net/minecraft/server/commands/DataPackCommand.java::createPack` writes
  `SharedConstants.getCurrentVersion().packVersion(PackType.SERVER_DATA).minorRange()`;
  `.vendor/vanilla-decompiled/net/minecraft/SharedConstants.java:31-33` has
  `DATA_PACK_FORMAT_MAJOR = 121`, `DATA_PACK_FORMAT_MINOR = 0`. Covered by `pack_mcmeta_json_structure`.
- 2. Duplicate enable/disable logic (`datapack.rs`): FIXED / VERIFIED. The command executors call
  `DatapackManager::enable_pack_checked` / `disable_pack_checked` and map the typed `DatapackError`
  through `error_type_for` (all six variants: Unknown, UnknownAnchor, AlreadyEnabled, NotEnabled,
  Embedded, Incompatible). The local `enable_pack` / `disable_pack` are thin delegating wrappers.
  Vanilla has one `PackRepository`:
  `.vendor/vanilla-decompiled/net/minecraft/server/commands/DataPackCommand.java::enablePack/disablePack`.
- 3. Anchor known-but-not-enabled (`datapack.rs` before/after): VERIFIED MATCHING. Vanilla resolves the
  anchor with `getPack(context, "existing", false)` (enabling = false): a known-but-unselected anchor
  throws `ERROR_PACK_ALREADY_DISABLED` (`commands.datapack.disable.failed`) and an unknown id throws
  `ERROR_UNKNOWN_PACK` (`commands.datapack.unknown`)
  (`.vendor/vanilla-decompiled/net/minecraft/server/commands/DataPackCommand.java:236-260`; unmodded
  26.2 `:285-309`). `enable_pack_checked` returns `NotEnabled` / `UnknownAnchor` and `error_type_for`
  maps them to `ERROR_DISABLE_FAILED` / `ERROR_UNKNOWN_DATAPACK`. The tracker claim of
  `ERROR_UNKNOWN_DATAPACK` for a known-but-unselected anchor is incorrect for this version.
- 4. `reload_datapacks` result discarded (`datapack.rs` enable/disable): VERIFIED NO CHANGE. Vanilla
  `enablePack` / `disablePack` call `ReloadCommand.reloadPacks(...)` (void, failures reported
  asynchronously) and the command returns `selected.size()`
  (`.vendor/vanilla-decompiled/net/minecraft/server/commands/DataPackCommand.java:173-189`). Pumpkin
  logs a warning on reload failure but still returns the enabled count, matching vanilla; propagating
  a `Result` would diverge from vanilla.
- 5. `pumpkin-unit-test` exposed (`data/datapack/mod.rs`): FIXED / VERIFIED.
  `DatapackManager::is_hidden_test_pack` filters `pumpkin-unit-test` from both `compute_enabled_packs`
  and `get_all_known_packs` in release builds; debug builds keep it for gametests. Vanilla
  `PackRepository` holds only the core pack and real world datapacks. Covered by
  `only_vanilla_pack_is_required` and `enabled_pack_list_filters_hidden_test_pack_and_disabled_packs`.
- 6. Suggestion ids (`datapack.rs` providers): FIXED / VERIFIED. Each provider publishes each pack id
  once, quoting ids that contain spaces or slashes, matching vanilla
  `StringArgumentType.escapeIfRequired` over `getSelectedIds` / `getAvailablePacks`
  (`.vendor/vanilla-decompiled/net/minecraft/server/commands/DataPackCommand.java:80-96`). No internal
  `file/<name>` short-name duplicates remain.
- 7. `is_valid_pack_name` (`datapack.rs`): FIXED / VERIFIED. `is_valid_pack_name` plus
  `is_reserved_windows_name` mirror vanilla `FileUtil.isValidPathSegment`
  (`STRICT_PATH_SEGMENT_CHECK = [-._a-z0-9]+`, rejecting `.` / `..`) and `isPathPartPortable`
  (`RESERVED_WINDOWS_FILENAMES = .*\.|(?:COM|CLOCK\$|CON|PRN|AUX|NUL|COM[1-9]|LPT[1-9])(?:\..*)?`)
  (`.vendor/vanilla-decompiled/net/minecraft/util/FileUtil.java:20-23,155-161`). Covered by
  `valid_pack_name_check`.
- 8. Pack order model / literal `before vanilla` (`data/datapack/mod.rs`): BLOCKED-DATA. Missing
  subsystem: a vanilla `PackRepository`-style ordered selected-pack model stored in `LevelData` that
  includes the built-in `vanilla` / embedded packs, plus a rewrite of `load_all` /
  `load_bundled_datapacks`. Bundled/embedded packs are always loaded first, so a world pack in
  `data_packs.enabled` can never precede `vanilla` in the load stack; `configurable_index` clamps the
  insert to the configurable slice, so the literal `before vanilla` placement cannot be represented.
  This is a whole selection-order subsystem and a cross-cutting refactor across
  `data/datapack/mod.rs` and `server/mod.rs`, not a missing generated `pumpkin-data` field.

No source edits this pass: rows 1-7 were already resolved and verified in the checkout, and row 8 is a
whole missing subsystem. Pure unit-test seams are already covered (`is_valid_pack_name`,
`pack_mcmeta_content`, `data_pack_format`, `valid_pack_name_check`, `compute_enabled_packs`,
`configurable_index`); suggestion providers need a `CommandContext`, so no seam.
