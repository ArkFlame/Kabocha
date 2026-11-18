# audit / crates/pumpkin-core/src/data/datapack/dynamic_registry_manager.rs

Source reviewed: `crates/pumpkin-core/src/data/datapack/dynamic_registry_manager.rs`
(branch kab/taudit-crates-pumpkin-core-src-da-2842f9), with
`crates/pumpkin-core/src/data/datapack/dynamic_registry_loader.rs` and the send site
`crates/pumpkin-core/src/net/java/login/known_packs.rs:7-26`.
Vanilla reference: `net.minecraft.server.packs.repository..RegistryDataLoader` /
`net.minecraft.core.RegistryDataLoader` (1.21.x) plus `RegistryDataLoader.SYNCHRONIZED_REGISTRIES`.
No `.vendor/` decompile in this worktree; behaviour checked against the vanilla classes.

Tracker note "2 TODO/FIXME/vanilla markers" is a false positive: the only matches are the doc
comment at `dynamic_registry_manager.rs:95` and the test name at line 287. There is no TODO/FIXME
in the file.

## Fixed in this pass

- Datapack-only registries were appended to `synced_order` and therefore shipped as
  `registry_data` packets (`known_packs.rs:9-13`). Vanilla only sends
  `RegistryDataLoader.SYNCHRONIZED_REGISTRIES`, a fixed list; the client logs and ignores an
  unknown registry key (`ClientPacketListener.handleRegistryData`), and such entries have
  `data: None` so they are not even decodable. Custom registries now stay server side
  (`dynamic_registry_manager.rs:169-181`), which also removes the HashMap-iteration
  nondeterminism from packet order.
- Lock-order inversion: `apply_custom_registries` took `registries` then `synced_order`,
  `get_synced_registries` took them in the opposite order, so a `/reload` racing a player join
  could deadlock. Both now take `registries` first (`dynamic_registry_manager.rs:259-274`).

## Remaining gaps

1. **No codec validation of datapack entries.** `dynamic_registry_loader.rs:257-272` converts
   JSON to raw NBT bytes and `RegistryEntryData.data` is stored unchecked
   (`dynamic_registry_manager.rs:170-179`). Vanilla decodes every entry through its registry codec
   (`RegistryDataLoader.RegistryData` + `Biome.NETWORK_CODEC`, `PaintingVariant.DIRECT_CODEC`, ...)
   and drops/reports the invalid entry. Here one malformed value is accepted at load and then
   invalidates the whole `registry_data` packet on the client. Suggested fix: validate each entry
   against the registry's schema (or at minimum decode with the existing packet NBT reader and
   reject on error), logging the resource id. Size: L.

2. **Override semantics of an existing entry are unverified.**
   `dynamic_registry_loader.rs:434-459` lets a datapack replace a vanilla entry's data while
   keeping its resource id and network id. Vanilla `RegistryDataLoader.loadContentsFromManager`
   reports a duplicate key for stable registries and keeps the built-in value; experimental
   datapacks are the only path that replaces entries. Confirm against the decompiled class before
   changing. Size: S if confirmed.

3. **No cross-entry validation.** Vanilla validates intra-registry references after load
   (e.g. `dimension_type` -> `biome`, `enchantment` -> `damage_type`) and fails the reload.
   `get_synced_registries` (`dynamic_registry_manager.rs:259-281`) sends whatever is stored, so a
   datapack with a dangling reference produces a client-side disconnect instead of a server-side
   load error. Size: L.

4. **Reload cost.** `apply_custom_registries` clones every base registry's entries plus rebuilds
   every `id_to_index` on each call (`dynamic_registry_manager.rs:153-154`), and
   `get_registry_entries`/`get_entry_data` clone the whole entry vector per call
   (`:219-245`). Vanilla builds an immutable registry set once per reload and hands out borrowed
   holders. Not observable parity, but per-join `known_packs` cost scales with all registry data.
   Size: M.

5. **Synced registry order is asset order.** `get_synced_registries` follows the generated
   `REGISTRY_V_*` order, vanilla follows `SYNCHRONIZED_REGISTRIES`. The packet is key addressed so
   this is not observable; only worth aligning if the generated list is ever regenerated from a
   sorted dump. Size: S, low priority.

## Status (sweep h2)

- FIXED (1e6e9d3ec): datapack-only registries stay out of the synced set and out of registry_data packets.
- FIXED (cca2a4fe7): registries and synced_order locks are taken in one order.
- STILL OPEN: gaps 1-4 (codec validation, override semantics, cross-entry validation, reload cost).

## Result (sweep 2026-11-17, verified against 26.3 decompiled vanilla)

Target is V_26_3 (crates/pumpkin-data/src/generated/packet.rs:3-4); decompiled source under
.vendor/vanilla-decompiled/net/minecraft/resources/ matches that version. No .rs source changes
were needed: gap 2 is already correct, gaps 1/3/4 cannot be fixed inside this unit's files.

- Gap 1 (codec validation of datapack entries): LEFT OPEN (needs generated data / per-registry
  codec schema). Vanilla decodes every entry through its registry codec and records a loading
  error on failure (RegistryLoadTask.java:97-109 PendingRegistration.loadFromResource;
  RegistryDataLoader.java:253-262 freezes/validates then throws on any error). Pumpkin stores
  opaque NBT with no schema. A "decode the NBT bytes" stopgap cannot help because
  json_to_registry_nbt_bytes (dynamic_registry_loader.rs:271-285) always emits structurally valid
  NBT; detecting schema/render errors requires the per-element DIRECT/NETWORK codecs that only
  exist as generated vanilla data. Out of scope for these three files.

- Gap 2 (override semantics of an existing entry): RESOLVED, no code change. The doc's 1.21.x
  loadContentsFromManager duplicate-key hypothesis does not apply to the 26.3 target (that
  method no longer exists in the decompiled tree). Vanilla builds each registry fresh from
  resource-manager files (ResourceManagerRegistryLoadTask.java:41-65); FileToIdConverter
  .listMatchingResources -> ResourceManager.listResources (FileToIdConverter.java:44-46)
  returns ONE resource per id, the highest-priority pack winning, so a datapack file that shares
  a vanilla id replaces the vanilla file outright (it does not keep the built-in value).
  Elements are then registered in sorted-key order (RegistryLoadTask.java:59-67), so an override
  keeps both its resource key and its raw id. Pumpkin's merge_dynamic_registry_entries
  (dynamic_registry_loader.rs:477-494) already replaces the vanilla data in place while keeping
  entry_id and index, and appends only genuinely new ids. That matches vanilla. The existing
  merge_preserves_and_appends test (dynamic_registry_loader.rs:623-660) locks this in, so no new
  test was added.

- Gap 3 (cross-entry validation): LEFT OPEN (needs generated data / reference schema). Vanilla
  resolves intra-registry holder references while decoding via RegistryOps (JsonOps + registry
  lookup, ResourceManagerRegistryLoadTask.java:47-57) and fails the whole reload on a dangling
  reference (RegistryDataLoader.java:253-262). Pumpkin has no map of which JSON fields are
  registry references (dimension_type->biome, enchantment->damage_type, ...); implementing this
  requires generated codec metadata. Out of scope for these three files.

- Gap 4 (reload cost): LEFT OPEN (cross-crate refactor; not observable parity). Removing the
  per-call clones at dynamic_registry_manager.rs:153-154, 219-247 and 276-279 means changing
  get_registry_entries/get_entry_data/get_synced_registries from owned clones to borrowed slices
  and updating their callers in other crates. Perf-only, so deferred rather than risk a broken
  API change that cannot be compiled here.

No STILL OPEN row can be fixed with a small, type-safe change confined to
dynamic_registry_loader.rs / dynamic_registry_manager.rs / known_packs.rs, and no rustfmt was run
because no .rs file was modified.


## Result (sweep 2026-11-18, re-verified against 26.3 decompiled vanilla)

Fresh pass over the four STILL OPEN rows. Decompiled tree is present this time at
`.vendor/vanilla-decompiled/net/minecraft/resources/` and matches the 26.3 target.
Re-read all three source files plus the send site; no .rs file was changed.

- Gap 1 (codec validation of datapack entries): LEFT OPEN, needs generated codec data.
  Re-confirmed at `ResourceManagerRegistryLoadTask.java:47-56` (each entry is parsed through
  `data.elementCodec().parse(RegistryOps<JsonElement>, json)`) and
  `RegistryLoadTask.java:98-109` (`loadFromResource` records `Either.right(error)` on any parse
  failure), with errors collected and thrown in `RegistryDataLoader.java:250-264`. Pumpkin
  stores opaque NBT with no per-registry schema; `json_to_registry_nbt_bytes`
  (dynamic_registry_loader.rs:191-205) always emits structurally valid NBT, so a "decode NBT"
  stopgap cannot detect a schema-invalid entry (e.g. a painting_variant missing `asset_id`).
  Rejecting such entries requires the per-element DIRECT/NETWORK codecs, which exist only as
  generated vanilla data outside this unit's files.

- Gap 2 (override semantics of an existing entry): RESOLVED, no code change (re-confirmed).
  `ResourceManagerRegistryLoadTask.java:44` uses `FileToIdConverter.listMatchingResources`
  (`FileToIdConverter.java:44-45` -> `ResourceManager.listResources`), which returns ONE
  resource per id with the highest-priority pack winning, then registers all entries in
  sorted-key order (`:65`, `sorted(Entry.comparingByKey())`). A datapack file sharing a vanilla
  id therefore replaces the built-in value outright while keeping its resource key; genuinely
  new ids are appended in key order. `merge_dynamic_registry_entries`
  (dynamic_registry_loader.rs:344-401) already replaces in place (keeping entry_id/index) and
  appends only new ids sorted by entry_id, and `merge_preserves_and_appends`
  (dynamic_registry_loader.rs:532-569) locks this in. No new test needed.

- Gap 3 (cross-entry validation): LEFT OPEN, needs generated reference schema.
  Re-confirmed at `ResourceManagerRegistryLoadTask.java:47` (`RegistryOps.create(JsonOps.INSTANCE,
  context)` resolves holder references while decoding) and
  `RegistryLoadTask.java:86-95`/`RegistryDataLoader.java:257-262` (`validateRegistry` fails the
  whole reload on any dangling reference or `RegistryValidator` failure such as `nonEmpty`).
  Pumpkin has no map of which JSON fields are cross-registry references
  (dimension_type->biome, enchantment->damage_type, ...) or which registries are `nonEmpty`,
  so this requires generated codec metadata. Out of scope for these files.

- Gap 4 (reload cost): LEFT OPEN, cross-crate API change; per-join path already mitigated.
  The hot path is already cheap: `SyncedRegistry` holds `Arc<Vec<RegistryEntryData>>`
  (dynamic_registry_manager.rs:102) and `get_synced_registries` (`:271-293`) Arc-clones it, so
  `known_packs.rs:9-15` reuses entry storage across joins without deep copying. The remaining
  clones (`get_registry_entries` :251-257, `get_all_entry_ids` :260-268, `get_entry_data`
  :231-239 clones one entry's bytes, not the whole vector) are lookup helpers, not per-join.
  Removing them means changing return types to borrowed slices and updating callers in
  `data/datapack/mod.rs` and elsewhere (other files / cross-crate), so deferred.

No STILL OPEN row has a small, type-safe fix confined to dynamic_registry_loader.rs,
dynamic_registry_manager.rs, known_packs.rs or packet.rs (the latter is the unrelated plugin
`PacketSentEvent` definitions). No rustfmt run because no .rs file was modified.

## Result (sweep 2026-11-18, fresh re-verification after server restart)

Re-read all four source files and re-verified each STILL OPEN row against the 26.3 decompiled
tree at `.vendor/vanilla-decompiled/net/minecraft/resources/`. No .rs file was changed.

New evidence re-confirming the prior verdicts:
- `ResourceManagerRegistryLoadTask.load` (`:44`) uses `FileToIdConverter.listMatchingResources`,
  which yields one resource per id (highest-priority pack wins), and registers every entry in
  sorted-key order (`:65`, `sorted(Entry.comparingByKey())`). This re-confirms gap 2: a datapack
  file with a vanilla id replaces it outright while keeping key/id, exactly what
  `merge_dynamic_registry_entries` (dynamic_registry_loader.rs:344-401) and
  `merge_preserves_and_appends` (:532-569) already do. No change.
- `RegistryLoadTask.loadFromResource` (`:98-109`) parses each entry through `elementDecoder.parse`
  and records `Either.right(error)` on failure; `validateRegistry` (`:86-95`) fails the reload on
  any dangling reference. This re-confirms gaps 1 and 3 require the per-element codec and
  cross-reference schemas that exist only as generated vanilla data, not in these files.
- `RegistryLoadTask.registerElements` (`:59-68`) registers in the supplied (sorted) stream order
  into a freshly built `MappedRegistry` per reload; Pumpkin's frozen `Arc<Vec<..>>` snapshot
  (`dynamic_registry_manager.rs:99-103`, `:271-293`) already avoids per-join deep copies, so gap 4
  remains perf-only and needs a cross-crate API change to remove the lookup-helper clones.

- Gap 1 (codec validation): LEFT OPEN, needs generated codec data (out of scope).
- Gap 2 (override semantics): RESOLVED, no code change, matches vanilla (re-confirmed).
- Gap 3 (cross-entry validation): LEFT OPEN, needs generated reference schema (out of scope).
- Gap 4 (reload cost): LEFT OPEN, cross-crate API change, not observable parity; hot path already Arc-backed.

No STILL OPEN row has a small, type-safe fix confined to dynamic_registry_loader.rs,
dynamic_registry_manager.rs, known_packs.rs or packet.rs. No rustfmt run because no .rs file
was modified.

## Result (sweep 2026-11-18, direct decompiled-source read of all three source files)

Re-read all four named files end to end and verified each STILL OPEN row directly against the
26.3 decompiled tree at `.vendor/vanilla-decompiled/net/minecraft/resources/`. No .rs file was
changed: no remaining row has a small, type-safe fix inside this unit's files. `packet.rs` is
`plugin/api/events/server/packet.rs` (plugin `PacketSentEvent` definitions), unrelated to every
gap.

- Gap 1 (codec validation of datapack entries): LEFT OPEN, needs generated per-element codec
  data. `RegistryLoadTask.PendingRegistration.loadFromResource` (RegistryLoadTask.java:98-109)
  parses each entry via `data.elementCodec().parse(RegistryOps<JsonElement>, json)` and records
  `Either.right(error)` on failure; `json_to_registry_nbt_bytes`
  (dynamic_registry_loader.rs:191-205) emits structurally valid NBT with no schema, so a
  "decode NBT" stopgap cannot detect a schema-invalid entry (e.g. a painting_variant without
  `asset_id`). Rejecting such entries needs the DIRECT/NETWORK codecs, generated data only.

- Gap 2 (override semantics): RESOLVED, no code change (verified in source this pass).
  `FileToIdConverter.listMatchingResources` (FileToIdConverter.java:44-46) delegates to
  `ResourceManager.listResources` and returns `Map<Identifier, Resource>`, one resource per id,
  highest-priority pack winning (contrast `listMatchingResourceStacks`, :48-50). Entries are
  registered in sorted-key order (ResourceManagerRegistryLoadTask.java:63-65). A datapack file
  sharing a vanilla id therefore replaces it outright while keeping its resource key and raw
  id. `merge_dynamic_registry_entries` (dynamic_registry_loader.rs:344-401) already replaces
  overrides in place (keeping entry_id/index) and appends only new ids sorted by entry_id;
  `merge_preserves_and_appends` (:533-569) locks this in. No new test needed.

- Gap 3 (cross-entry validation): LEFT OPEN, needs generated reference/validator schema.
  `RegistryOps.create(JsonOps.INSTANCE, context)` (ResourceManagerRegistryLoadTask.java:47)
  resolves holder references while decoding and `RegistryLoadTask.validateRegistry`
  (RegistryLoadTask.java:86-95) fails the whole reload on a dangling reference or validator
  error (`nonEmpty`). Pumpkin has no map of which JSON fields are cross-registry references
  (dimension_type->biome, enchantment->damage_type, ...) or which registries are `nonEmpty`, so
  this requires generated codec metadata outside these files.

- Gap 4 (reload cost): LEFT OPEN, cross-crate API change and not observable parity.
  The per-join path is already Arc-backed: `get_synced_registries`
  (dynamic_registry_manager.rs:271-293) Arc-clones `SyncedRegistry.entries`, so
  `known_packs.rs:9-15` reuses entry storage across joins without deep copies. The remaining
  clones in `get_registry_entries`/`get_all_entry_ids`/`get_entry_data` are lookup helpers whose
  return types are consumed in other files; changing them to borrowed slices is a cross-crate
  refactor, deferred.

No STILL OPEN row is fixable with a small change confined to dynamic_registry_loader.rs,
dynamic_registry_manager.rs, known_packs.rs or packet.rs. No rustfmt run: no .rs file modified.
