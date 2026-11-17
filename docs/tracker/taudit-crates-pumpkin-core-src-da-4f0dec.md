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
- Invalid input is now skipped with a warning instead of stored or mangled: `load_registry_dir_recursive`
  rejects ids that are not valid resource locations, unreadable or unparsable JSON, and non-object JSON
  roots (vanilla `FileToIdConverter` + registry codec behaviour); the `json_to_registry_nbt_bytes` doc
  now states a non-object root never reaches it. Covered by the `invalid_registry_files_are_skipped` test.
- Deleted the no-op `reg_entry.entry_id.clone_from(&entry.entry_id)` in
  `merge_dynamic_registry_entries` (the struct literal already sets that field).

## Remaining gaps

1. **JSON is streamed to NBT, bypassing the vanilla codec** (`json_to_nbt_tag:229-268`). Vanilla decodes
   each element with its registry codec and re-encodes through `NbtOps`, so field types are normalized
   (`Codec.LONG` -> `LongTag`) and unknown fields are dropped only if the codec does. Here every JSON
   integer becomes `Int` unless it exceeds `i32`, so a datapack that writes a long-typed field
   (e.g. structure/placement seeds, `worldgen/noise_settings` long values) sends `IntTag` where vanilla
   sends `LongTag`. Most client codecs read numbers generically, so this is latent, not yet observed
   broken. Fix: per-registry codecs, or a value-range hint table. Size L.
2. **Non-vanilla directory aliases** (`DYNAMIC_REGISTRIES:11-225`). Every registry lists a plural
   directory alias (`wolf_variants`, `enchantments`, ...) in addition to the vanilla path, and
   `worldgen/carver` / `worldgen/configured_carver` (likewise `feature`/`configured_feature`) scan the
   same two directories into two registries. Vanilla reads only the singular registry path
   (`Registries.elementsDirPath`). Harmless superset for vanilla packs, but it means a pack's
   `enchantments/` folder is silently accepted here and ignored by vanilla. Intentional leniency; do not
   remove without a migration note. Size S.
3. **Auto-discovery of unknown data folders** (`dynamic_registry_loader.rs:367-406`). Any directory under
   `data/<ns>/` that is not in `EXCLUDED_REGISTRY_DIRS` becomes a registry keyed `<ns>:<dir>`. Vanilla
   ignores unknown folders entirely; here an unrelated folder (e.g. a pack's `data/ns/config/`) turns
   into a registry that can be offered for sync. `EXCLUDED_REGISTRY_DIRS:315-340` also lists
   `"datapacks"`, which is not a vanilla `data/<ns>/` layout folder and is dead weight. Fix: keep the
   allow-list for `minecraft:*` and only auto-discover namespaces that ship a registry manifest.
   Size M.

## Second pass (same audit)

- Fixed: unknown `worldgen/<sub>` folders discovered by auto-discovery were keyed
  `<pack-namespace>:worldgen/<sub>` (`dynamic_registry_loader.rs:422`), so a pack's
  `data/mypack/worldgen/<new_registry>/` landed in a registry id that no vanilla consumer or sync
  roster can ever reference. Vanilla worldgen registry keys are the fixed `minecraft:` ids from
  `Registries` (`Registries.BIOME` -> `minecraft:worldgen/biome`, etc.), independent of the data-pack
  namespace; only the *element* ids carry the pack namespace. Now keyed `minecraft:worldgen/<sub>`,
  covered by `unknown_worldgen_dirs_use_minecraft_registry_key`. Known vanilla dirs were already
  correct through `DYNAMIC_REGISTRIES`.
- Still open (from the list above): JSON->NBT numeric typing (L), plural directory aliases (S,
  intentional), auto-discovery of non-worldgen folders (M).
- New small notes, not fixed:
  - `DYNAMIC_REGISTRIES:126-164` scans the `worldgen/carver`/`worldgen/feature` directories into
    *both* the built-in type registry id and the `configured_*` one. Vanilla only loads
    `worldgen/configured_carver` / `worldgen/configured_feature` (the `Registries.CARVER`/`FEATURE`
    type registries are builtin, codec-registered in code, no data pack loader). Today harmless: those
    two ids are absent from `pumpkin_data::registry::Registry::get_synced` and nothing reads them, so
    the duplicate registry is server-only memory, and the entry count is not inflated (measured per
    directory). Suggested fix when touching the table: drop the two builtin ids, keep the directory
    aliases under `configured_*`. Size S.
  - `DYNAMIC_REGISTRIES` also loads `item_modifier`, `slot_source`, `trade_set`, `villager_trade`,
    `template_pool` and `processor_list`, which `datapack/mod.rs:2268-2334` parses again with their
    dedicated loaders (same files, two parse passes and two copies). Behaviour matches vanilla (those
    ids are real datapack registries); only the duplicated I/O remains. Size S.
  - `EXCLUDED_REGISTRY_DIRS:352-377` still lists `"datapacks"`, which is never a `data/<ns>/` folder.
    Dead entry, no behaviour impact. Size S.

## Gap status
- Fixed in this pass section (unknown `worldgen/<sub>` keyed correctly): FIXED per note.
- 1. JSON streamed to NBT bypassing codec (`json_to_nbt_tag:229-268`): STILL OPEN (L).
- 2. Plural directory aliases (`DYNAMIC_REGISTRIES:11-225`): STILL OPEN (S).
- 3. Auto-discovery of unknown folders (`dynamic_registry_loader.rs:367-406`): STILL OPEN (S).


## Result

Verified against `.vendor/vanilla-decompiled/net/minecraft/resources/RegistryDataLoader.java`,
`core/registries/Registries.java` and `FileToIdConverter.java`.

- Gap 3 (auto-discovery of unknown folders, S): FIXED. `load_dynamic_registries_from_ns` no
  longer turns arbitrary `data/<ns>/<dir>` folders into `<ns>:<dir>` registries; only worldgen
  subfolders are auto-discovered (already keyed `minecraft:worldgen/<sub>`). Vanilla loads only
  the fixed `RegistryDataLoader` set and ignores unknown folders. Removed the now-unused
  `EXCLUDED_REGISTRY_DIRS` const (also drops the dead `datapacks` entry noted in pass 2).
  Test: `unknown_non_worldgen_folders_are_ignored`.
- New small note (duplicate builtin carver/feature registries, S): FIXED. Removed the
  `minecraft:worldgen/configured_carver` and `minecraft:worldgen/configured_feature` entries from
  `DYNAMIC_REGISTRIES`. Decompiled vanilla loads `Registries.CARVER` -> `worldgen/carver` and
  `Registries.FEATURE` -> `worldgen/feature` into those canonical ids; `configured_*` registry ids
  do not exist in this version (the type registries are `worldgen/carver_type`/`feature_type`,
  builtin with no datapack loader). The legacy `configured_*` directories remain aliases under the
  canonical ids, so old packs still load into the right registry. Test:
  `configured_carver_entries_load_into_canonical_registry`.
- Gap 1 (JSON streamed to NBT bypassing codecs, L): STILL OPEN - correct long-vs-int typing needs
  each registry's element codec (schema lives in generated data / other crates); out of scope for
  this file.
- Gap 2 (plural directory aliases, S): STILL OPEN - documented intentional leniency; removing it
  is a behaviour change for non-vanilla packs that needs a migration note, and it is not a
  vanilla-correctness bug for valid packs.
- New small note (duplicated I/O for item_modifier/slot_source/etc.): STILL OPEN - requires changes
  to `datapack/mod.rs`, a file outside this unit.

Files touched: crates/pumpkin-core/src/data/datapack/dynamic_registry_loader.rs (rustfmt applied).
Checks not run here (orchestrator compiles/nextests the batch).
