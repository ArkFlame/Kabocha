# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-da-2842f9.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: dynamic_registry_loader.rs

Re-read all four source files and re-verified each STILL OPEN row against the 26.3 decompiled
No STILL OPEN row has a small, type-safe fix confined to dynamic_registry_loader.rs,
Re-read all four named files end to end and verified each STILL OPEN row directly against the
No STILL OPEN row is fixable with a small change confined to dynamic_registry_loader.rs,

## Result (FINAL PASS, verified against 26.3 decompiled vanilla)

All four rows are closed. No `.rs` source change was needed: two rows already match vanilla and two are genuinely blocked on generated data that `crates/pumpkin-data/src/generated/registry.rs` does not expose.

- Gap 1 (codec validation of datapack entries): BLOCKED-DATA. Missing generated field: a per-registry element codec, i.e. vanilla `RegistryDataLoader.RegistryData.elementCodec()` (`Biome.DIRECT_CODEC`, `PaintingVariant.DIRECT_CODEC`, ...). Vanilla parses every entry through it and records a load error on failure (`.vendor/vanilla-decompiled/net/minecraft/resources/RegistryLoadTask.java:98-109`, `ResourceManagerRegistryLoadTask.java:55`) and fails the reload. `pumpkin-data::registry::RegistryEntryData` only carries `entry_id` and opaque `data` bytes, with no codec schema to validate against. `json_to_registry_nbt_bytes` (dynamic_registry_loader.rs:191-205) always emits structurally valid NBT, so a decode-NBT stopgap cannot detect a schema-invalid entry (e.g. a painting_variant without `asset_id`).

- Gap 2 (override semantics of an existing entry): VERIFIED, matches vanilla. `.vendor/vanilla-decompiled/net/minecraft/resources/FileToIdConverter.java:44-45` (`listMatchingResources` -> `ResourceManager.listResources`) returns one resource per id, highest-priority pack winning (contrast `listMatchingResourceStacks`, :48-50), and `ResourceManagerRegistryLoadTask.java:65` registers all entries in sorted-key order. A datapack file sharing a vanilla id therefore replaces it outright while keeping its key and id. `merge_dynamic_registry_entries` (dynamic_registry_loader.rs:344-401) replaces overrides in place (entry_id/index preserved) and appends new ids sorted by entry_id; test `merge_preserves_and_appends` (:533-569) locks this in. No change.

- Gap 3 (cross-entry validation): BLOCKED-DATA. Missing generated fields: `RegistryDataLoader.RegistryData.validator()` (e.g. `RegistryValidator.nonEmpty()`, see `RegistryDataLoader.java:119-133`) plus the cross-registry holder-reference fields inside each element codec. Vanilla resolves holder references via `RegistryOps.create(JsonOps.INSTANCE, context)` (`ResourceManagerRegistryLoadTask.java:47`) while decoding and fails the whole reload on a dangling reference or validator error (`RegistryLoadTask.java:86-95`). Generated registry data has no reference/validator schema (no map of dimension_type->biome, enchantment->damage_type, or nonEmpty flags), so this cannot be implemented here.

- Gap 4 (reload cost): VERIFIED, no observable parity divergence. Vanilla builds a frozen immutable registry set once per reload (`.vendor/vanilla-decompiled/net/minecraft/resources/RegistryLoadTask.java:76-84`, `freezeRegistry`). Pumpkin's `SyncedRegistry` (dynamic_registry_manager.rs:99-103) shares entries behind an `Arc` and `get_synced_registries` (:271-293) Arc-clones them, so the per-join `known_packs.rs:9-15` path reuses one copy without deep copies; test `synced_registries_share_entry_storage_between_calls` (dynamic_registry_manager.rs:385-393) locks this in. The remaining per-call clones are lookup helpers (`get_registry_entries`, `get_all_entry_ids`, `get_entry_data`) with no observable parity impact; removing them is a cross-crate API change, not a parity fix.

No `.rs` file was modified, so no rustfmt was run.
