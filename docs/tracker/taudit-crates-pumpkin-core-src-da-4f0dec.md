# audit / crates/pumpkin-core/src/data/datapack/dynamic_registry_loader.rs (AUDIT)

Source reviewed: `crates/pumpkin-core/src/data/datapack/dynamic_registry_loader.rs` (branch
kab/taudit-crates-pumpkin-core-src-da-4f0dec), plus its caller `data/datapack/mod.rs:724,2264`.
Vanilla reference: `net.minecraft.resources.RegistryDataLoader`,
`net.minecraft.resources.RegistrySynchronization`, `net.minecraft.server.packs.resources.FileToIdConverter`
(1.21.x behaviour, no .vendor decompile in this worktree). No TODO/FIXME markers exist in the file;
the tracker note's "8 markers" are the word "vanilla" in doc comments/tests.

## Fixed in this pass

- JSON `null` was converted to `NbtTag::End`, which the compound/list writer emits as a type-0 entry.
  A reader stops at the first End, so any key after a null field (and the rest of the registry entry
  payload sent to the client) was lost, i.e. a single `"field": null` truncated the whole element.
  `json_to_nbt_tag` now returns `Option<NbtTag>` and drops null object keys / list elements, matching
  NBT having no null and vanilla codecs never encoding one (`json_to_nbt_tag`, `json_to_registry_nbt_bytes`).
- Files were matched with `eq_ignore_ascii_case("json")` but the suffix stripped with a case-sensitive
  `strip_suffix(".json")`, so `Foo.JSON` loaded under the id `ns:Foo.JSON` (invalid resource location).
  Vanilla `FileToIdConverter` only accepts the exact `.json` suffix; the loader now does the same
  (`load_registry_dir_recursive`).

## Remaining gaps

1. **Invalid JSON is silently skipped** (`dynamic_registry_loader.rs:280-282,299-310`). Vanilla
   `RegistryDataLoader.loadContentsFromManager` throws `JsonParseException`/codec error, which fails
   the pack and aborts world load. Pumpkin returns/ignores: a malformed registry file produces a
   server that starts with the vanilla entry only, with no log line. Suggested fix: `warn!` per file
   with path and serde error at minimum; full parity needs a pack-level error propagated by
   `Datapack::load_all` (`mod.rs:262`). Size S for the warn, M for failing the load.
2. **JSON is streamed to NBT, bypassing the vanilla codec** (`json_to_nbt_tag:229-268`). Vanilla decodes
   each element with its registry codec and re-encodes through `NbtOps`, so field types are normalized
   (`Codec.LONG` -> `LongTag`) and unknown fields are dropped only if the codec does. Here every JSON
   integer becomes `Int` unless it exceeds `i32`, so a datapack that writes a long-typed field
   (e.g. structure/placement seeds, `worldgen/noise_settings` long values) sends `IntTag` where vanilla
   sends `LongTag`. Most client codecs read numbers generically, so this is latent, not yet observed
   broken. Fix: per-registry codecs, or a value-range hint table. Size L.
3. **Entry ids are not validated as resource locations** (`dynamic_registry_loader.rs:292-297`). A file
   such as `data/ns/wolf_variant/My Dog.json` produces `ns:My Dog`, which vanilla rejects at load with a
   `ResourceLocationException`; Pumpkin stores it and writes it into the registry-sync payload, so the
   client may fail to parse the packet. Fix: validate namespace/path (`[a-z0-9_.-/]`) and skip + warn.
   Size S.
4. **Non-compound root is mangled** (`json_to_registry_nbt_bytes:257-271`). A registry file whose root is
   not a JSON object (array/string/number) is written through `NbtTag::serialize`, i.e. as a *named*
   root tag inside an "unnamed" payload. Vanilla fails such a file (codec expects a map). Fix: emit an
   empty compound or reject; currently only reachable with broken packs. Size S.
5. **Non-vanilla directory aliases** (`DYNAMIC_REGISTRIES:11-225`). Every registry lists a plural
   directory alias (`wolf_variants`, `enchantments`, ...) in addition to the vanilla path, and
   `worldgen/carver` / `worldgen/configured_carver` (likewise `feature`/`configured_feature`) scan the
   same two directories into two registries. Vanilla reads only the singular registry path
   (`Registries.elementsDirPath`). Harmless superset for vanilla packs, but it means a pack's
   `enchantments/` folder is silently accepted here and ignored by vanilla. Intentional leniency; do not
   remove without a migration note. Size S.
6. **Auto-discovery of unknown data folders** (`dynamic_registry_loader.rs:367-406`). Any directory under
   `data/<ns>/` that is not in `EXCLUDED_REGISTRY_DIRS` becomes a registry keyed `<ns>:<dir>`. Vanilla
   ignores unknown folders entirely; here an unrelated folder (e.g. a pack's `data/ns/config/`) turns
   into a registry that can be offered for sync. `EXCLUDED_REGISTRY_DIRS:315-340` also lists
   `"datapacks"`, which is not a vanilla `data/<ns>/` layout folder and is dead weight. Fix: keep the
   allow-list for `minecraft:*` and only auto-discover namespaces that ship a registry manifest.
   Size M.
7. **Merge ordering** (`merge_dynamic_registry_entries:461-472`). Custom-only entries are appended in
   lexicographic order; vanilla registry iteration order is `Reference2ObjectMap` (hash) order. The sync
   packet carries ids, so the client is unaffected; documented so nobody "fixes" it later. Also
   `dynamic_registry_loader.rs:449` is a no-op (`reg_entry.entry_id.clone_from(&entry.entry_id)` right
   after the struct was built with that same value); dead line, safe to delete. Size S (cosmetic).
