# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-da-2842f9.md

Files: dynamic_registry_loader.rs

## Result

- Gap 1 (codec validation of datapack entries): BLOCKED-DATA. Missing generated field: a per-registry element codec (vanilla `RegistryDataLoader.RegistryData.elementCodec()`, e.g. `Biome.DIRECT_CODEC`). Vanilla fails the reload on a decode error (`.vendor/vanilla-decompiled/net/minecraft/resources/RegistryLoadTask.java:98-109`). `pumpkin-data::registry::RegistryEntryData` carries only `entry_id` and opaque `data`.
- Gap 2 (override semantics): VERIFIED. `.vendor/vanilla-decompiled/net/minecraft/resources/FileToIdConverter.java:44-45` gives one resource per id with the highest-priority pack winning; `merge_dynamic_registry_entries` in dynamic_registry_loader.rs replaces overrides in place.
- Gap 3 (cross-entry validation): BLOCKED-DATA. Missing generated fields: `RegistryData.validator()` and the holder-reference schema inside element codecs (`RegistryDataLoader.java:119-133`).
- Gap 4 (reload cost): VERIFIED. Vanilla freezes one registry set per reload (`RegistryLoadTask.java:76-84`); `SyncedRegistry` shares entries behind an `Arc` (dynamic_registry_manager.rs:99-103).

No `.rs` file was modified in this pass.
