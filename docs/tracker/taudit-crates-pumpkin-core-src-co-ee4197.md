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
