# Final rows from docs/tracker/tpar-es-pumpkin-core-src-data-datapack-dynamic-registry-loader-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

terminal in that doc; none STILL OPEN. Final statuses:
No .rs source change was needed: every remaining STILL OPEN row is either already
already end FIXED/VERIFIED in that doc; none STILL OPEN. Its sole former open row

## Result
FINAL PASS (all rows terminal). This unit closes the two audits tracked by
docs/tracker/tpar-es-pumpkin-core-src-data-datapack-dynamic-registry-loader-rs.md. No .rs
source file is named in this unit (basenames: none): every in-scope source fix was already
landed in earlier passes. This pass re-verified each row directly against the decompiled 26.3
tree at .vendor/vanilla-decompiled/net/minecraft/. No source edit, no rustfmt, no cargo/git.

own/taudit-crates-pumpkin-core-src-da-2842f9 (dynamic_registry_manager.rs audit):
- Gap 1 (codec validation of datapack entries): BLOCKED-DATA. Missing generated per-element
  codec schemas, the DIRECT_CODEC values paired in RegistryDataLoader.WORLD_REGISTRIES
  (DimensionType.DIRECT_CODEC, Biome.DIRECT_CODEC, PaintingVariant.DIRECT_CODEC) at
  .vendor/vanilla-decompiled/net/minecraft/resources/RegistryDataLoader.java:98-197. Vanilla
  parses each entry via elementDecoder().parse(RegistryOps, json) and records a loading error
  on failure (resources/RegistryLoadTask.java:98-109). Pumpkin stores the body as opaque NBT
  (RegistryEntryData.data: Option<Box<[u8]>>) with no schema, so a schema-invalid entry cannot
  be rejected server side; no such generated field exists.
- Gap 2 (override semantics): VERIFIED, matches vanilla.
  .vendor/vanilla-decompiled/net/minecraft/resources/FileToIdConverter.java:44-46 returns one
  resource per id (highest-priority pack wins) via ResourceManager.listResources, and entries
  register in sorted-key order
  (.vendor/vanilla-decompiled/net/minecraft/resources/ResourceManagerRegistryLoadTask.java:44,63-65).
  merge_dynamic_registry_entries already replaces overrides in place (keeping entry_id/index)
  and appends only new ids sorted by entry_id; merge_preserves_and_appends locks it in.
- Gap 3 (cross-entry validation): BLOCKED-DATA. Missing generated cross-registry reference map
  and RegistryValidator.nonEmpty() metadata (WOLF_VARIANT at
  .vendor/vanilla-decompiled/net/minecraft/resources/RegistryDataLoader.java:119). Vanilla
  resolves holder references while decoding via RegistryOps.create
  (resources/ResourceManagerRegistryLoadTask.java:47) and fails the reload on a dangling
  reference through validateRegistry (resources/RegistryLoadTask.java:86-95). Pumpkin has no
  such reference/validator schema in generated data.
- Gap 4 (reload cost): VERIFIED, no observable parity divergence. The per-join path Arc-shares
  SyncedRegistry.entries (dynamic_registry_manager.rs:271-293), matching vanilla's single
  frozen registry set per reload
  (.vendor/vanilla-decompiled/net/minecraft/core/RegistrySynchronization.java). Remaining clones
  are lookup helpers with cross-file call sites, not a parity defect.
- Gap 5 (synced registry order is asset order, S): VERIFIED, not observable. registry_data
  packets are key addressed and the client builds its registry access from all received
  registries regardless of order
  (.vendor/vanilla-decompiled/net/minecraft/resources/NetworkRegistryLoadTask.java and
  core/RegistrySynchronization.java).

own/taudit-crates-pumpkin-core-src-da-4f0dec (dynamic_registry_loader.rs audit): all rows VERIFIED.
- Gap 1 (JSON -> NBT numeric typing, L): VERIFIED, non-observable. NbtOps has no typed numeric
  getters: getNumberValue = input.asNumber()
  (.vendor/vanilla-decompiled/net/minecraft/nbt/NbtOps.java:70-71), so an IntTag, LongTag or
  DoubleTag carrying the same value decodes to the identical codec value on the client
  (resources/NetworkRegistryLoadTask.java:47,60). LONG_FIELDS = ["seed"] plus
  long_typed_seed_fields_encode_as_long_tags remain best-effort wire fidelity.
- Gap 2 (plural directory aliases, S): VERIFIED. DYNAMIC_REGISTRIES lists only singular paths,
  matching the fixed loader set and exact .json suffix
  (resources/RegistryDataLoader.java:98-150, resources/FileToIdConverter.java:11-16,28-30).
- Gap 3 (auto-discovery of unknown folders, S): VERIFIED. Only worldgen/<sub> folders are
  discovered, keyed minecraft:worldgen/<sub>; other data/<ns>/<dir> folders are ignored
  (resources/RegistryDataLoader.java:98-150).
- Note (duplicate builtin carver/feature, S): VERIFIED. Only canonical worldgen/carver and
  worldgen/feature ids exist (resources/RegistryDataLoader.java:102-103).
- Note (duplicated I/O for item_modifier/slot_source/etc., S): VERIFIED. Those ids are loaded by
  dedicated modules mirroring vanilla RELOADABLE_REGISTRIES
  (resources/RegistryDataLoader.java:154-163).

No STILL OPEN row remains. The two BLOCKED-DATA rows require generated codec/validator fields
absent from pumpkin-data and cannot be resolved with a small type-safe source change. Worker
compile pitfalls: no code added, so no unwrap/expect/panic, import, helper, macro or struct-init
changes.
