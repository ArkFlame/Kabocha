# Parity rows for crates/pumpkin-core/src/data/datapack/dynamic_registry_loader.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-da-2842f9** [partial/OWN-OPEN] own audit: 8 open, 2 fixed, 0 verified, 1 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-da-2842f9.md) docs/tracker/taudit-crates-pumpkin-core-src-da-2842f9.md
- **own/taudit-crates-pumpkin-core-src-da-4f0dec** [partial/OWN-OPEN] own audit: 12 open, 12 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-da-4f0dec.md) docs/tracker/taudit-crates-pumpkin-core-src-da-4f0dec.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)


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
