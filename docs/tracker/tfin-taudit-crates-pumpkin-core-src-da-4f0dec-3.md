# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-da-4f0dec.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

All rows end FIXED/VERIFIED; none BLOCKED-DATA, none STILL OPEN. No source edits this pass
All rows end FIXED/VERIFIED; none BLOCKED-DATA, none STILL OPEN. No source edits this pass

## Result
Final pass. Every row verified against decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/`. The source file
`crates/pumpkin-core/src/data/datapack/dynamic_registry_loader.rs` already carries all in-scope
fixes; no source edits this pass (this unit names no source file).

- Gap 1 (JSON streamed to NBT bypassing element codecs, L): VERIFIED. Vanilla's server encodes each
  element through its element codec with NbtOps (`core/RegistrySynchronization.java:59-62`) and the
  client decodes via `RegistryOps.create(NbtOps.INSTANCE, ...)` + `elementCodec()`
  (`resources/NetworkRegistryLoadTask.java:47,60`). `NbtOps` overrides only `getNumberValue`, which is
  `input.asNumber()` (`nbt/NbtOps.java:70-71`), so every numeric tag carrying the same value decodes
  to the identical codec value. Byte-level tag-type differences are not client-observable. The
  conservative `LONG_FIELDS = ["seed"]` hint plus `long_typed_seed_fields_encode_as_long_tags` test
  remain as best-effort wire fidelity. No missing generated field or subsystem blocks correctness.
- Gap 2 (plural directory aliases, S): FIXED. `DYNAMIC_REGISTRIES` lists only singular paths matching
  `Registries.elementsDirPath` = registry identifier path (`core/registries/Registries.java:344-350`);
  vanilla reads only the exact `.json` suffix and singular registry dir
  (`resources/FileToIdConverter.java:11-16,29`). Covered by `plural_directory_aliases_are_not_loaded`
  and `instance_registry_uses_singular_directory_only`.
- Gap 3 (auto-discovery of unknown folders, S): FIXED. `load_dynamic_registries_from_ns` auto-discovers
  only `worldgen/<sub>` folders, keyed `minecraft:worldgen/<sub>`; other `data/<ns>/<dir>` folders are
  ignored, matching the fixed `RegistryDataLoader.WORLD_REGISTRIES` set
  (`resources/RegistryDataLoader.java:98-150`). Covered by `unknown_non_worldgen_folders_are_ignored`
  and `unknown_worldgen_dirs_use_minecraft_registry_key`.
- Note (duplicate builtin carver/feature registries, S): FIXED. Only canonical `Registries.CARVER` /
  `Registries.FEATURE` ids exist (`resources/RegistryDataLoader.java:102-103`); `configured_*` dirs are
  legacy aliases under those canonical ids. Covered by
  `configured_carver_entries_load_into_canonical_registry`.
- Note (duplicated I/O for item_modifier/slot_source/trade_set/villager_trade/template_pool/
  processor_list/damage_type, S): FIXED. Those ids are absent from `DYNAMIC_REGISTRIES` and are loaded
  by dedicated modules in `datapack/mod.rs`, mirroring vanilla's separate `RELOADABLE_REGISTRIES` set
  (`resources/RegistryDataLoader.java:154-163`).

All rows end FIXED/VERIFIED; none BLOCKED-DATA, none STILL OPEN. No source edits this pass, so
rustfmt is not needed. Checks not run by worker (orchestrator compiles/nextests the batch).
