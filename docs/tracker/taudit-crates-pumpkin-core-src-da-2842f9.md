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
