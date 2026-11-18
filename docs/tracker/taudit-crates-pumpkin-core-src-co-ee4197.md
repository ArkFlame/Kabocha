# Audit: crates/pumpkin-core/src/command/commands/datapack.rs

Vanilla reference: `net.minecraft.server.commands.DatapackCommand` (1.21.5+; `enable ... first|last|before|after`),
`net.minecraft.server.packs.repository.PackRepository`. No TODOs/FIXMEs in the file; the tracker's
"5 markers" are the `"vanilla"` string hits, not markers.

## Fixed in this branch
- `disable` of an unknown id reported `commands.datapack.disable.failed`; vanilla throws
  `ERROR_UNKNOWN_DATAPACK` (DatapackCommand#disablePack: `packRepository.getPack(id) == null`).
- `disable` only blocked the literal `vanilla` pack, so any other compiled-in pack
  (`pumpkin-unit-test`) was removed from world info, reported success, and stayed enabled because
  `DatapackManager::get_enabled_packs` re-adds embedded packs. Now uses `is_embedded_pack`.
- `list` (all) returned only the enabled count; vanilla `listPacks` returns
  `selectedIds.size() + availableIds.size()`.
- `enable`/`disable` returned `level_info.data_packs.enabled.len()`, which excludes built-in packs;
  vanilla returns `packRepository.getSelectedIds().size()` (always includes `vanilla`).
- `enable <name> before|after <built-in>` (for example `before vanilla`) validated the anchor as
  enabled, found it absent from `data_packs.enabled` and silently appended the pack to the end.
  Vanilla inserts relative to the selected order, so the position is now mapped from the exposed
  list (built-in prefix first) and clamped to the configurable packs (`configurable_index`).

## Remaining gaps
1. datapack.rs:455 (also data/datapack/mod.rs:1467,1474,1489,2388,2431 and tests at :572) —
   `pack_format: 61` is hardcoded for `datapack create` and generated `pack.mcmeta`. Vanilla writes
   `SharedConstants.getCurrentVersion().getPackVersion(PackType.SERVER_DATA)` (data pack version of
   `CURRENT_MC_VERSION`, now V_26_3). Add a version -> data pack format table in `pumpkin-util`/`pumpkin-data`
   and use it. Size M (needs the 26.3 data pack version from vanilla).
2. datapack.rs:219-419 duplicates `DatapackManager::enable_pack` / `disable_pack`
   (data/datapack/mod.rs:1534,1626). The two copies already diverged (embedded-pack check, error
   strings). Vanilla has one `PackRepository`; make the command call the manager and map `Err` to the
   translation error types. Size M.
3. datapack.rs:303-310 — enabling `before`/`after` an anchor that is known but not enabled throws
   `ERROR_DISABLE_FAILED`; vanilla 1.21.5+ `enablePack(..., anchor)` throws `ERROR_UNKNOWN_DATAPACK`
   when the anchor is not in `getSelectedIds()`. Verify against decompiled 1.21.5 before changing. Size S.
4. datapack.rs:261,345,399 — `reload_datapacks` result is discarded; vanilla propagates datapack load
   failures from `reloadResources` back to the command (enable/disable then reports failure instead of
   success). Needs `reload_datapacks` to return `Result`. Size M.
5. data/datapack/mod.rs:1399 (`get_enabled_packs`) unconditionally exposes `pumpkin-unit-test` as an
   enabled pack, so `/datapack list enabled` shows `[pumpkin-unit-test]` on production servers; vanilla
   only ever shows real packs. Filter the embedded test pack unless a dev/test mode is on. Size S.
6. datapack.rs:73-122 suggestion providers publish Pumpkin-internal ids (`file/<name>`, and both the
   unquoted and quoted spelling); vanilla `SUGGEST_AVAILABLE_PACKS` suggests `packRepository.getAvailableIds()`
   verbatim and never inserts quotes. Low impact. Size S.
7. datapack.rs:414-422 `is_valid_pack_name` accepts names vanilla rejects for `datapack create`
   (leading/trailing dots, e.g. `a.` resolves oddly on Windows). Vanilla validates the folder name
   before writing; confirm the exact rule and tighten. Size S.
8. datapack.rs:319-346 the exposed order is fixed as `vanilla`, `pumpkin-unit-test` followed by
   `data_packs.enabled`, so `before vanilla` can only clamp to the first configurable slot, not truly
   precede `vanilla`. Vanilla keeps one ordered `PackRepository` selection. Model pack order in
   `LevelData` (or keep embedded packs in `data_packs.enabled`) and drop the clamp. Size M.

## Verified matching
Command tree (`list`, `list available`, `list enabled`, `enable` default=last, `before`, `after`,
`disable`, `create`), permission level 2, broadcast feedback (`sendSuccess(..., true)`), enabled/unknown
error split for `enable`, count of `enable`/`disable`, and `pack.mcmeta` JSON shape match vanilla intent.

## Gap status
- Fixed in this branch section: FIXED per note.
- 1. `datapack.rs:455` (and `data/datapack/mod.rs` sites): STILL OPEN (S).
- 2. Duplicate enable/disable logic (`datapack.rs:219-419`): STILL OPEN (M).
- 3. `before`/`after` anchor not enabled throws (`datapack.rs:303-310`): STILL OPEN (S).
- 4. `reload_datapacks` result discarded (`datapack.rs:261,345,399`): STILL OPEN (S).
- 5. `pumpkin-unit-test` exposed (`data/datapack/mod.rs:1399`): STILL OPEN (S).
- 6. Suggestion ids are Pumpkin-internal (`datapack.rs:73-122`): STILL OPEN (S).


## Result

Verification used the unmodded MC 26.2 source at
/run/media/linsaftw/SDD/GitHub/ArkFlame/FlameMC/folia-server/.gradle/caches/paperweight/mache/base/sources/net/minecraft/server/commands/DataPackCommand.java
(the vanilla-decompiled copy in .vendor is Canvas-modded: register and the anchor Inserter are stripped).

- 1. pack_format: 61 hardcoded - LEFT OPEN. Needs a version -> data pack format table plus the
  format for V_26_3; pumpkin-util/src/version.rs exposes only protocol_version, no data pack version.
  Cross-crate data change, outside this unit (only datapack.rs editable).
- 2. Duplicate enable/disable logic - LEFT OPEN. Requires refactoring DatapackManager::enable_pack
  and disable_pack in data/datapack/mod.rs (basename mod.rs, outside this unit).
- 3. Anchor known but not enabled error type - VERIFIED MATCHING, NO CHANGE. Unmodded vanilla
  resolves the before/after anchor via getPack(c, "existing", false); a known-but-not-selected anchor
  throws ERROR_PACK_ALREADY_DISABLED (commands.datapack.disable.failed). Pumpkin's
  ERROR_DISABLE_FAILED already matches. The tracker claim of ERROR_UNKNOWN_DATAPACK is incorrect
  for this version.
- 4. reload_datapacks result discarded - LEFT OPEN. Requires Server::reload_datapacks
  (server/mod.rs) to return Result; outside this unit.
- 5. pumpkin-unit-test exposed - LEFT OPEN. data/datapack/mod.rs is outside this unit.
- 6. Suggestion ids - FIXED. Both providers now suggest each pack id once, quoting ids containing
  spaces or slashes (required because read_string cannot parse an unquoted slash). Removed the
  internal file/<name> short-name duplicates. Matches vanilla UNSELECTED_PACKS / SELECTED_PACKS.
  No pure unit test seam exists (suggestion providers need a CommandContext), so no test added.

Edits: crates/pumpkin-core/src/command/commands/datapack.rs; rustfmt --edition 2024 applied.


## Result

Re-verified against `.vendor/vanilla-decompiled` (26.3, protocol 777); anchor
handling cross-checked against the unmodded 26.2 mache source. This run also had
`version.rs` in scope.

- 1. pack_format hardcode - FIXED (already in checkout). `JavaMinecraftVersion::data_pack_format()` in `pumpkin-util/src/version.rs` returns 121 for `V_26_3`, matching vanilla `SharedConstants.DATA_PACK_FORMAT_MAJOR = 121` / `MINOR = 0`; `LAST_PRE_MINOR_DATA_PACK_FORMAT = 81` matches `PackFormat.lastPreMinorVersion(SERVER_DATA)`. `pack_mcmeta_content` writes `min_format`/`max_format` as integers because minor == 0, matching `PackFormat.minorRange()` + `fullCodec`. Covered by `pack_mcmeta_json_structure`.
- 2. Duplicate enable/disable logic - FIXED (already in checkout). `datapack.rs` calls `DatapackManager::enable_pack_checked` / `disable_pack_checked` and maps typed `DatapackError` via `error_type_for`; the local copies are gone.
- 3. Anchor known but not enabled - VERIFIED MATCHING, NO CHANGE. Vanilla `getPack(c, "existing", false)` throws `ERROR_PACK_ALREADY_DISABLED` for a known-but-unselected anchor -> `NotEnabled` -> `ERROR_DISABLE_FAILED`; a truly unknown anchor throws `ERROR_UNKNOWN_PACK` -> `UnknownAnchor` -> `ERROR_UNKNOWN_DATAPACK`. `error_type_for` maps both correctly.
- 4. reload_datapacks result discarded - NO CHANGE NEEDED. Vanilla `enablePack`/`disablePack` call `ReloadCommand.reloadPacks(...)` which returns void and reports failures asynchronously; the command result is `selected.size()`. Pumpkin logs a warning instead. Propagating a `Result` would diverge and needs `server/mod.rs` (outside this unit).
- 5. pumpkin-unit-test exposed - LEFT OPEN. `get_enabled_packs` lives in `data/datapack/mod.rs` (basename mod.rs, outside this unit).
- 6. Suggestion ids - FIXED (already in checkout). Both providers suggest each id once, quoting only ids with spaces or slashes; no internal `file/<name>` duplicates.
- 7. `is_valid_pack_name` (Remaining gaps, in scope) - FIXED this run. Now mirrors vanilla `FileUtil.isValidPathSegment` (`[-._a-z0-9]+`, rejecting `.`/`..` and uppercase) and `isPathPartPortable` (trailing dot, reserved device names such as CON/COM1/LPT9). Extended the `valid_pack_name_check` unit test with those cases and a leading-dot positive case.

Edits this run: `crates/pumpkin-core/src/command/commands/datapack.rs` (name validation + test); `rustfmt --edition 2024` applied. `version.rs` verified, no edit needed.


## Result

Final verification pass against `.vendor/vanilla-decompiled` (26.3). No further source edits were
needed: every in-scope STILL OPEN row is already resolved in the checkout and matches vanilla.

- 1. `pack_format` hardcode - VERIFIED FIXED. `SharedConstants.DATA_PACK_FORMAT_MAJOR = 121`,
  `MINOR = 0`; `PackFormat.lastPreMinorVersion(SERVER_DATA) = 81`. `version.rs::data_pack_format()`
  returns 121 for V_26_3, `LAST_PRE_MINOR_DATA_PACK_FORMAT` is 81, and `CURRENT_MC_VERSION = V_26_3`.
  `PackFormat.fullCodec` serializes `minor == defaultMinor` as a single int, so `min_format`/`max_format`
  are integers 121, matching `pack_mcmeta_content` and its `pack_mcmeta_json_structure` test. The
  remaining `61` literals in `data/datapack/mod.rs` are legacy test fixtures, not production values.
- 2. Duplicate enable/disable - VERIFIED FIXED. `datapack.rs` calls `enable_pack_checked` /
  `disable_pack_checked`; all six `DatapackError` variants are mapped by `error_type_for`.
- 3. Anchor known-but-not-enabled - VERIFIED MATCHING. Vanilla `DataPackCommand.getPack` throws
  `commands.datapack.disable.failed` for a known-but-unselected anchor and `commands.datapack.unknown`
  for an unknown id; `enable_pack_checked` returns `NotEnabled` / `UnknownAnchor` accordingly and
  `error_type_for` maps both to the matching command error.
- 4. `reload_datapacks` result - VERIFIED NO CHANGE. Vanilla `enablePack`/`disablePack` call
  `ReloadCommand.reloadPacks(...)` which returns void and reports failures asynchronously; the command
  result is `selected.size()`. Propagating a `Result` would diverge from vanilla.
- 5. `pumpkin-unit-test` exposed - LEFT OPEN. `get_enabled_packs` lives in `data/datapack/mod.rs`
  (outside this unit's files: datapack.rs, version.rs).
- 6. Suggestion ids - VERIFIED FIXED. Both providers suggest each id once, quoting ids with spaces or
  slashes; no internal `file/<name>` duplicates.
- 7. `is_valid_pack_name` - VERIFIED FIXED. Mirrors vanilla `FileUtil.isValidPathSegment`
  (`[-._a-z0-9]+`, rejects `.`/`..`) and `isPathPartPortable` (trailing dot plus reserved device names
  CON/PRN/AUX/NUL/CLOCK$/COM[1-9]/LPT[1-9]); covered by `valid_pack_name_check`.
- 8. Pack order model - LEFT OPEN. Truly preceding `vanilla` requires modeling selection order in
  `LevelData` inside `data/datapack/mod.rs` (outside this unit); the current clamp to the first
  configurable slot is the smallest in-scope behaviour.

No code edits this pass (all in-scope rows already fixed/matching); existing tests cover the only pure
unit-test seams (`is_valid_pack_name`, `pack_mcmeta_content`, `data_pack_format`); suggestion providers
need a `CommandContext`, so no seam. Files changed this pass: none (unit doc only).


## Result (native-4 cross-file pass)
- Row 5 (`pumpkin-unit-test` exposed): BLOCKED-L, the expose point is `data/datapack/mod.rs` `get_enabled_packs`; not closed in this pass.
