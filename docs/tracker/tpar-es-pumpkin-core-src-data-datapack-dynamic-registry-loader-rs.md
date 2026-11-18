# Parity rows for crates/pumpkin-core/src/data/datapack/dynamic_registry_loader.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-da-2842f9** [partial/OWN-OPEN] own audit: 8 open, 2 fixed, 0 verified, 1 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-da-2842f9.md) docs/tracker/taudit-crates-pumpkin-core-src-da-2842f9.md
- **own/taudit-crates-pumpkin-core-src-da-4f0dec** [partial/OWN-OPEN] own audit: 12 open, 12 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-da-4f0dec.md) docs/tracker/taudit-crates-pumpkin-core-src-da-4f0dec.md

## Result
FINAL PASS (all rows terminal). Re-verified against decompiled vanilla at
.vendor/vanilla-decompiled/net/minecraft/ (26.3 tree). No .rs source edits this pass:
every remaining open row is either already fixed/verified in source or needs
generated codec/validator data that pumpkin-data does not carry.

- own/taudit-crates-pumpkin-core-src-da-2842f9 (dynamic_registry_manager.rs):
  - Gap 1 (codec validation of datapack entries): BLOCKED-DATA. Exact missing fields: the
    per-registry element codecs paired in RegistryDataLoader.WORLD_REGISTRIES (e.g.
    DimensionType.DIRECT_CODEC, Biome.DIRECT_CODEC, PaintingVariant.DIRECT_CODEC) at
    .vendor/vanilla-decompiled/net/minecraft/resources/RegistryDataLoader.java:98-197.
    pumpkin-data RegistryEntryData (crates/pumpkin-data/src/generated/registry.rs:13-16)
    carries only entry_id and opaque data: Option<Box<[u8]>>, with no codec schema, so a
    schema-invalid entry cannot be rejected server side.
  - Gap 2 (override semantics): VERIFIED, matches vanilla. ResourceManagerRegistryLoadTask.java
    :44,65 lists one resource per id via FileToIdConverter.java:44-46 (highest-priority pack
    wins) and registers in sorted-key order (RegistryLoadTask.java:59-67), so a datapack file
    sharing a vanilla id replaces it in place. merge_dynamic_registry_entries
    (dynamic_registry_loader.rs:344-401) replaces in place keeping entry_id/index and appends
    only new ids sorted by entry_id; merge_preserves_and_appends locks it in. No code change.
  - Gap 3 (cross-entry validation): BLOCKED-DATA. Exact missing fields: the cross-registry
    reference map and the RegistryValidator.nonEmpty() metadata attached to
    RegistryDataLoader.RegistryData (e.g. WOLF_VARIANT at
    .vendor/vanilla-decompiled/net/minecraft/resources/RegistryDataLoader.java:119), resolved
    via RegistryOps.create at ResourceManagerRegistryLoadTask.java:47 and enforced by
    RegistryLoadTask.validateRegistry (:86-95). No such reference/validator schema exists in
    generated pumpkin-data, so a dangling reference cannot be rejected server side.
  - Gap 4 (reload cost): VERIFIED, no observable parity divergence. get_synced_registries
    (dynamic_registry_manager.rs:271-293) Arc-shares SyncedRegistry.entries (:95-103) so
    registry-data sends reuse entry storage, matching the single frozen registry set per
    reload (.vendor/vanilla-decompiled/net/minecraft/core/RegistrySynchronization.java). Test
    synced_registries_share_entry_storage_between_calls (:385-393) locks this in. Remaining
    clones are lookup helpers whose call sites include data/datapack/mod.rs (outside this
    unit); removing them is a cross-file API change with zero observable benefit, deferred.
  - Gap 5 (synced registry order is asset order, S): VERIFIED, not observable. registry_data
    packets are key addressed; the client builds its registry access from all received
    registries regardless of packet order (.vendor/vanilla-decompiled/net/minecraft/resources/
    NetworkRegistryLoadTask.java and core/RegistrySynchronization.java). No code change.

- own/taudit-crates-pumpkin-core-src-da-4f0dec (dynamic_registry_loader.rs): all rows already
  terminal in that doc; none STILL OPEN. Final statuses:
  - Gap 1 (JSON -> NBT numeric typing, L): VERIFIED, matches observable vanilla behaviour.
    NbtOps has no typed numeric getters: getNumberValue = input.asNumber()
    (.vendor/vanilla-decompiled/net/minecraft/nbt/NbtOps.java:69-72), so an IntTag, LongTag or
    DoubleTag carrying the same value decodes to the identical codec value on the client
    (resources/NetworkRegistryLoadTask.java:47,60). The LONG_FIELDS = ["seed"] hint and
    long_typed_seed_fields_encode_as_long_tags test remain as best-effort wire fidelity.
  - Gap 2 (plural directory aliases, S): VERIFIED. DYNAMIC_REGISTRIES lists only singular
    paths, matching Registries.elementsDirPath and FileToIdConverter.java:11-30; tests
    plural_directory_aliases_are_not_loaded and instance_registry_uses_singular_directory_only.
  - Gap 3 (auto-discovery of unknown folders, S): VERIFIED. Only worldgen/<sub> folders are
    discovered, keyed minecraft:worldgen/<sub>; other data/<ns>/<dir> folders are ignored,
    matching the fixed loader set at resources/RegistryDataLoader.java:98-150. Tests
    unknown_non_worldgen_folders_are_ignored and unknown_worldgen_dirs_use_minecraft_registry_key.
  - Note (duplicate builtin carver/feature registries, S): VERIFIED. Only canonical
    worldgen/carver / worldgen/feature ids exist (RegistryDataLoader.java:102-103).
  - Note (duplicated I/O for item_modifier/slot_source/etc., S): VERIFIED. Those ids are
    loaded by dedicated modules mirroring vanilla RELOADABLE_REGISTRIES
    (resources/RegistryDataLoader.java:154-163).

Files touched: this doc only. No rustfmt (no .rs edits). No cargo/git run by worker.


## Result (third pass, final closure)

No .rs source change was needed: every remaining STILL OPEN row is either already
fixed/verified in source or needs generated codec/validator data that pumpkin-data does
not carry. Re-verified against the 26.3 decompiled tree at
.vendor/vanilla-decompiled/net/minecraft/.

- own/taudit-crates-pumpkin-core-src-da-2842f9 (dynamic_registry_manager.rs audit):
  - Gap 1 (codec validation of datapack entries): BLOCKED-DATA. Exact missing field: the
    per-registry element codecs paired in RegistryDataLoader.WORLD_REGISTRIES (e.g.
    DimensionType.DIRECT_CODEC, Biome.DIRECT_CODEC, PaintingVariant.DIRECT_CODEC) at
    .vendor/vanilla-decompiled/net/minecraft/resources/RegistryDataLoader.java:98-197.
    Pumpkin stores opaque NBT in RegistryEntryData.data (Option<Box<[u8]>>) with no schema,
    and json_to_registry_nbt_bytes always emits structurally valid NBT, so rejecting a
    schema-invalid entry needs codec metadata that exists only as generated vanilla data.
  - Gap 2 (override semantics): VERIFIED, matches vanilla.
    .vendor/vanilla-decompiled/net/minecraft/resources/ResourceManagerRegistryLoadTask.java:44,65
    and FileToIdConverter.java:44-46: listMatchingResources yields one resource per id
    (highest-priority pack wins) and entries register in sorted-key order, so a datapack file
    sharing a vanilla id replaces it in place. merge_dynamic_registry_entries already replaces
    in place keeping entry_id/index and appends only new ids sorted by entry_id;
    merge_preserves_and_appends locks it in. No code change.
  - Gap 3 (cross-entry validation): BLOCKED-DATA. Exact missing fields: the cross-registry
    reference map and the RegistryValidator.nonEmpty() metadata attached to
    RegistryDataLoader.RegistryData (e.g. WOLF_VARIANT at
    .vendor/vanilla-decompiled/net/minecraft/resources/RegistryDataLoader.java:119), resolved
    via RegistryOps.create at ResourceManagerRegistryLoadTask.java:47 and enforced by
    RegistryLoadTask.validateRegistry. Pumpkin has no such reference/validator schema in
    generated data, so a dangling reference cannot be rejected server side.
  - Gap 4 (reload cost): VERIFIED, no observable parity divergence. The per-join hot path
    get_synced_registries (dynamic_registry_manager.rs:271-293) Arc-clones
    SyncedRegistry.entries so registry-data sends reuse entry storage, matching vanilla's
    single frozen registry set per reload (.vendor/vanilla-decompiled/net/minecraft/core/
    RegistrySynchronization.java). The remaining clones are lookup helpers
    (get_registry_entries/get_all_entry_ids/get_entry_data) whose call sites span
    data/datapack/mod.rs and command/argument_types/resource_key.rs; removing them is a
    cross-crate API change with zero observable benefit, deferred.

- own/taudit-crates-pumpkin-core-src-da-4f0dec (dynamic_registry_loader.rs audit): all rows
  already end FIXED/VERIFIED in that doc; none STILL OPEN. Its sole former open row
  (JSON->NBT numeric typing) is VERIFIED as non-observable: NbtOps has no typed numeric
  getters (getNumberValue = input.asNumber(),
  .vendor/vanilla-decompiled/net/minecraft/nbt/NbtOps.java:70-71), so an IntTag, LongTag or
  DoubleTag carrying the same value decodes to the identical codec value on the client
  (NetworkRegistryLoadTask.java:47,60). The LONG_FIELDS = ["seed"] hint and
  long_typed_seed_fields_encode_as_long_tags test remain as best-effort wire fidelity.

Files touched: this doc only. No rustfmt (no .rs edits). No cargo/git run by worker.
