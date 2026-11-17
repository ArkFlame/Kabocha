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

## Verified matching
Command tree (`list`, `list available`, `list enabled`, `enable` default=last, `before`, `after`,
`disable`, `create`), permission level 2, broadcast feedback (`sendSuccess(..., true)`), enabled/unknown
error split for `enable`, count of `enable`/`disable`, and `pack.mcmeta` JSON shape match vanilla intent.
